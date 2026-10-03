//! Facet selector construction, row extraction, and anchor validation.

use crate::Error;

use crate::builder::Plot;
use crate::layout::PanelBox;
use crate::spec::PlotKind;

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
    pub(super) fn render_scope(&self, data: crate::data::PlotData<'_>) -> Result<String, Error> {
        let mut scope = String::new();
        data.reserve_source_string(&mut scope, 32)?;
        if self.cell.is_empty() {
            use std::fmt::Write;
            write!(scope, "facet-{}", self.panel_index).expect("reserved index");
        } else {
            scope.push_str("facet ");
            for (index, selector) in self.cell.iter().enumerate() {
                let length = selector
                    .variable
                    .len()
                    .checked_add(selector.level.len())
                    .and_then(|length| length.checked_add(3))
                    .ok_or_else(|| Error::Render("facet scope length overflow".into()))?;
                data.reserve_source_string(&mut scope, length)?;
                if index != 0 {
                    scope.push_str(", ");
                }
                scope.push_str(&selector.variable);
                scope.push('=');
                scope.push_str(&selector.level);
            }
        }
        Ok(scope)
    }
    pub(super) fn significance_selectors(&self) -> impl Iterator<Item = (&str, &str)> {
        self.cell
            .iter()
            .map(|selector| (selector.variable.as_str(), selector.level.as_str()))
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "one admitted selector owner preserves wrapped and crossed facet ordering"
)]
pub(super) fn facet_selectors(
    plot: &Plot,
    data: crate::data::PlotData<'_>,
) -> Result<FacetSelectors, Error> {
    let data = if let Some(rows) = plot.configured_source_rows()? {
        data.map_source_rows(rows)
    } else {
        data
    };
    let (row_column, column_column, wrap_column, wrap_columns) = plot.facet_columns();
    let mut variable_names = Vec::new();
    data.reserve_source(&mut variable_names, 3)?;
    for column in [row_column, column_column, wrap_column]
        .into_iter()
        .flatten()
    {
        variable_names.push(data.column(column)?.name());
    }
    plot.resolve_axis_label_domain("facet variables", variable_names.iter().copied(), data)?;
    let cells = if let Some(wrap) = wrap_column {
        let levels = data.source_levels(&crate::data::strict_strings(data, wrap, "facet")?)?;
        plot.resolve_axis_label_domain("facet levels", levels.iter().map(String::as_str), data)?;
        let columns = wrap_columns.unwrap_or(1);
        if columns == 0 {
            return Err(Error::Data(
                "wrapped facets require at least one column".into(),
            ));
        }
        let variable = data.column(wrap)?.name();
        data.collect_source(levels.into_iter().enumerate().map(|(index, level)| {
            Ok((
                index / columns,
                index % columns,
                data.collect_source(std::iter::once(Ok(FacetSelector {
                    column: wrap,
                    variable: data.copy_source_string(variable)?,
                    level,
                })))?,
            ))
        }))?
    } else {
        let row_levels = row_column.map_or_else(
            || data.collect_source(std::iter::once(Ok(String::new()))),
            |column| data.source_levels(&crate::data::strict_strings(data, column, "facet")?),
        )?;
        let column_levels = column_column.map_or_else(
            || data.collect_source(std::iter::once(Ok(String::new()))),
            |column| data.source_levels(&crate::data::strict_strings(data, column, "facet")?),
        )?;
        if row_column.is_some() {
            plot.resolve_axis_label_domain(
                "facet levels for \"__facet_row\"",
                row_levels.iter().map(String::as_str),
                data,
            )?;
        }
        if column_column.is_some() {
            plot.resolve_axis_label_domain(
                "facet levels for \"__facet_column\"",
                column_levels.iter().map(String::as_str),
                data,
            )?;
        }
        let mut cells = Vec::new();
        let count = row_levels
            .len()
            .checked_mul(column_levels.len())
            .ok_or_else(|| Error::Render("facet cell count overflow".into()))?;
        data.reserve_source(&mut cells, count)?;
        for (row_index, row_level) in row_levels.iter().enumerate() {
            for (column_index, column_level) in column_levels.iter().enumerate() {
                let mut cell = Vec::new();
                data.reserve_source(
                    &mut cell,
                    usize::from(row_column.is_some()) + usize::from(column_column.is_some()),
                )?;
                if let Some(column) = row_column {
                    cell.push(FacetSelector {
                        column,
                        variable: data.copy_source_string(data.column(column)?.name())?,
                        level: data.copy_source_string(row_level)?,
                    });
                }
                if let Some(column) = column_column {
                    cell.push(FacetSelector {
                        column,
                        variable: data.copy_source_string(data.column(column)?.name())?,
                        level: data.copy_source_string(column_level)?,
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
    data.claim_source_visits(cells.len())?;
    data.claim_source_visits(cells.len())?;
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

pub(super) fn base_plot(
    source: &Plot,
    kind: PlotKind,
    has_horizontal_axis_label: bool,
    has_vertical_axis_label: bool,
    resources: &crate::font::canonical::OperationGenerationResources,
) -> Result<Plot, Error> {
    let source_rows = source.configured_source_rows()?;
    let data = source_rows
        .map_or_else(|| source.data(), |rows| source.data().map_source_rows(rows))
        .with_axis_resources(resources);
    let mut base_plot = source.admitted_clone(data)?.without_facets();
    if has_horizontal_axis_label {
        base_plot = base_plot.x_label("");
    }
    if has_vertical_axis_label {
        base_plot = base_plot.y_label("");
    }
    let independent_collectives = matches!(
        kind,
        PlotKind::Density | PlotKind::Line | PlotKind::Interaction
    ) || kind == PlotKind::LinePath
        && source.line_path_style_spec() == crate::LinePathStyle::Hue;
    if independent_collectives {
        let channels = crate::data::observed_collectives(
            data,
            source.group_col(),
            source.group_name(),
            source.hue_col(),
            source.hue_name(),
            None,
            "facet collective channels",
        )?;
        base_plot = base_plot
            .with_global_group_levels(channels.group_levels)
            .with_global_hue_levels(channels.hue_levels)
            .with_global_collective_pairs(channels.identities);
    } else if let Some(group_column) = source.group_col() {
        let levels = stable_levels(crate::data::strict_strings(
            data,
            group_column,
            "facet graphical group",
        )?);
        base_plot = base_plot.with_global_group_levels(levels);
    }
    if kind == PlotKind::Trajectory {
        if let Some(x_column) = source.x_col() {
            let column = data.column(x_column).map_err(|error| {
                Error::Data(format!("trajectory: column {x_column:?}: {error}"))
            })?;
            if !column.is_numeric() {
                let levels = stable_levels(crate::data::strict_normalized_strings(
                    data,
                    x_column,
                    "trajectory",
                    "condition",
                )?);
                base_plot = base_plot.with_global_line_condition_levels(levels);
            }
        }
    }
    Ok(base_plot)
}

#[expect(
    clippy::needless_range_loop,
    reason = "row index addresses every selector column, not the outer column vector"
)]
pub(super) fn extract_panels(
    plot: &Plot,
    selectors: &FacetSelectors,
    provisional: Option<&[PanelBox]>,
    mapped_data: crate::data::PlotData<'_>,
) -> Result<Vec<FacetPanel>, Error> {
    let configured_source_rows = plot.configured_source_rows()?;
    let mapped_data = if let Some(rows) = configured_source_rows {
        mapped_data.map_source_rows(rows)
    } else {
        mapped_data
    };
    let mut panels = Vec::new();
    mapped_data.reserve_source(&mut panels, selectors.cells.len())?;
    for (panel_index, (row, column, cell)) in selectors.cells.iter().enumerate() {
        let columns_data = mapped_data
            .collect_source(cell.iter().map(|selector| {
                crate::data::strict_strings(mapped_data, selector.column, "facet")
            }))?;
        let mut source_indices = Vec::new();
        mapped_data.reserve_source(&mut source_indices, plot.row_count())?;
        for index in 0..plot.row_count() {
            let mut matches = true;
            for (selector_index, selector) in cell.iter().enumerate() {
                mapped_data.claim_source_visits(selector.level.len())?;
                if columns_data[selector_index][index] != selector.level {
                    matches = false;
                    break;
                }
            }
            if matches {
                source_indices.push(index);
            }
        }
        if source_indices.is_empty() {
            continue;
        }
        let source_rows = mapped_data.collect_source(source_indices.iter().map(|index| {
            Ok(configured_source_rows
                .as_ref()
                .map_or(*index, |rows| rows[*index]))
        }))?;
        let panel = provisional.map_or(PanelBox::new(0.0, 1.0, 0.0, 1.0), |boxes| {
            boxes[*row * selectors.columns + *column]
        });
        panels.push(FacetPanel {
            panel_index,
            row: *row,
            column: *column,
            cell: mapped_data.collect_source(cell.iter().map(|selector| {
                Ok(FacetSelector {
                    column: selector.column,
                    variable: mapped_data.copy_source_string(&selector.variable)?,
                    level: mapped_data.copy_source_string(&selector.level)?,
                })
            }))?,
            rows: source_indices,
            source_rows,
            panel,
        });
    }
    Ok(panels)
}

pub(super) fn validate_significance_facet_anchors(
    plot: &Plot,
    panels: &[FacetPanel],
    data: crate::data::PlotData<'_>,
) -> Result<(), Error> {
    let dimensions = panels.first().map_or(0, |panel| panel.cell.len());
    for annotation in plot.significance_annotations() {
        data.claim_source_visits(1)?;
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
        let mut anchored = false;
        for panel in panels {
            data.claim_source_visits(1)?;
            for selector in &panel.cell {
                data.claim_source_visits(selector.level.len())?;
            }
            if cell.matches(panel.cell.iter().map(|selector| selector.level.as_str())) {
                anchored = true;
                break;
            }
        }
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
