//! Facet selector construction, row extraction, and anchor validation.

use crate::Error;

use crate::builder::{Plot, PlotDescription};
use crate::layout::PixelRect;
use crate::orchestrator::registration::{select_rows, RowSelection};
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
    pub(super) rows: RowSelection,
    pub(super) panel: PixelRect,
}

impl FacetPanel {
    pub(super) fn render_scope(&self) -> Result<String, Error> {
        let mut scope = String::new();
        crate::data::source::reserve_source_string(&mut scope, 32)?;
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
                crate::data::source::reserve_source_string(&mut scope, length)?;
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
}

#[expect(
    clippy::too_many_lines,
    reason = "one admitted selector owner preserves wrapped and crossed facet ordering"
)]
pub(super) fn facet_selectors(plot: &PlotDescription) -> Result<FacetSelectors, Error> {
    let rows = plot.configured_source_rows()?;
    let column = |key: &str| {
        plot.data_column(key)
            .ok_or_else(|| Error::Data(format!("unknown semantic value {key:?}")))
    };
    let strings = |key: &str| {
        let column = column(key)?;
        let crate::data::DataColumn::Category(values) = column else {
            return Err(Error::Data(format!(
                "facet: value {:?} must be categorical",
                column.name()
            )));
        };
        crate::data::strict_source_strings(values, rows, "facet")
    };
    let (row_column, column_column, wrap_column, wrap_columns) = plot.facet_columns();
    let mut variable_names = Vec::new();
    crate::data::source::reserve_source(&mut variable_names, 3)?;
    for column in [row_column, column_column, wrap_column]
        .into_iter()
        .flatten()
    {
        variable_names.push(
            plot.data_column(column)
                .ok_or_else(|| Error::Data(format!("unknown semantic value {column:?}")))?
                .name(),
        );
    }
    plot.resolve_axis_label_domain("facet variables", variable_names.iter().copied())?;
    let cells = if let Some(wrap) = wrap_column {
        let levels = crate::data::source::source_levels(&strings(wrap)?)?;
        plot.resolve_axis_label_domain("facet levels", levels.iter().map(String::as_str))?;
        let columns = wrap_columns.unwrap_or(1);
        if columns == 0 {
            return Err(Error::Data(
                "wrapped facets require at least one column".into(),
            ));
        }
        let variable = column(wrap)?.name();
        crate::data::source::collect_source(levels.into_iter().enumerate().map(
            |(index, level)| {
                Ok((
                    index / columns,
                    index % columns,
                    crate::data::source::collect_source(std::iter::once(Ok(FacetSelector {
                        column: wrap,
                        variable: crate::data::source::copy_source_string(variable)?,
                        level,
                    })))?,
                ))
            },
        ))?
    } else {
        let row_levels = row_column.map_or_else(
            || crate::data::source::collect_source(std::iter::once(Ok(String::new()))),
            |column| crate::data::source::source_levels(&strings(column)?),
        )?;
        let column_levels = column_column.map_or_else(
            || crate::data::source::collect_source(std::iter::once(Ok(String::new()))),
            |column| crate::data::source::source_levels(&strings(column)?),
        )?;
        if row_column.is_some() {
            plot.resolve_axis_label_domain(
                "facet levels for \"__facet_row\"",
                row_levels.iter().map(String::as_str),
            )?;
        }
        if column_column.is_some() {
            plot.resolve_axis_label_domain(
                "facet levels for \"__facet_column\"",
                column_levels.iter().map(String::as_str),
            )?;
        }
        let mut cells = Vec::new();
        let count = row_levels
            .len()
            .checked_mul(column_levels.len())
            .ok_or_else(|| Error::Render("facet cell count overflow".into()))?;
        crate::data::source::reserve_source(&mut cells, count)?;
        for (row_index, row_level) in row_levels.iter().enumerate() {
            for (column_index, column_level) in column_levels.iter().enumerate() {
                let mut cell = Vec::new();
                crate::data::source::reserve_source(
                    &mut cell,
                    usize::from(row_column.is_some()) + usize::from(column_column.is_some()),
                )?;
                if let Some(column) = row_column {
                    cell.push(FacetSelector {
                        column,
                        variable: crate::data::source::copy_source_string(
                            plot.data_column(column)
                                .ok_or_else(|| {
                                    Error::Data(format!("unknown semantic value {column:?}"))
                                })?
                                .name(),
                        )?,
                        level: crate::data::source::copy_source_string(row_level)?,
                    });
                }
                if let Some(column) = column_column {
                    cell.push(FacetSelector {
                        column,
                        variable: crate::data::source::copy_source_string(
                            plot.data_column(column)
                                .ok_or_else(|| {
                                    Error::Data(format!("unknown semantic value {column:?}"))
                                })?
                                .name(),
                        )?,
                        level: crate::data::source::copy_source_string(column_level)?,
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

pub(super) fn base_plot(
    source: &Plot,
    kind: PlotKind,
) -> Result<(Plot, Option<Vec<crate::data::CollectivePair>>), Error> {
    let source_rows = source.description().configured_source_rows()?;
    let mut base_plot = source.clone().without_facets();
    let mut collectives = None;
    let independent_collectives = matches!(
        kind,
        PlotKind::Density | PlotKind::Line | PlotKind::Interaction
    ) || kind == PlotKind::LinePath
        && source.description().line_path_style_spec() == crate::LinePathStyle::Hue;
    if independent_collectives {
        let channels = crate::data::observed_source_collectives(
            source.description(),
            "facet collective channels",
        )?;
        base_plot = base_plot
            .with_global_group_levels(channels.group_levels)
            .with_global_hue_levels(channels.hue_levels);
        collectives = Some(channels.identities);
    } else if let Some(group_column) = source.description().group_col() {
        let column = source
            .description()
            .data_column(group_column)
            .ok_or_else(|| Error::Data(format!("unknown semantic value {group_column:?}")))?;
        let crate::data::DataColumn::Category(values) = column else {
            return Err(Error::Data(format!(
                "facet graphical group: value {:?} must be categorical",
                column.name()
            )));
        };
        let levels = stable_levels(crate::data::strict_source_strings(
            values,
            source_rows,
            "facet graphical group",
        )?);
        base_plot = base_plot.with_global_group_levels(levels);
    }
    if kind == PlotKind::Trajectory {
        if let Some(x_column) = source.description().x_col() {
            let column = source
                .description()
                .data_column(x_column)
                .ok_or_else(|| Error::Data(format!("unknown semantic value {x_column:?}")))
                .map_err(|error| {
                    Error::Data(format!("trajectory: column {x_column:?}: {error}"))
                })?;
            if !column.is_numeric() {
                let crate::data::DataColumn::Category(values) = column else {
                    return Err(Error::Data(format!(
                        "trajectory: value {:?} must be categorical",
                        column.name()
                    )));
                };
                let _ = stable_levels(crate::data::strict_source_normalized_strings(
                    values,
                    source_rows,
                    "trajectory",
                    "condition",
                )?);
            }
        }
    }
    Ok((base_plot, collectives))
}

#[expect(
    clippy::needless_range_loop,
    reason = "row index addresses every selector column, not the outer column vector"
)]
pub(super) fn extract_panels(
    plot: &PlotDescription,
    selectors: &FacetSelectors,
    provisional: Option<&[PixelRect]>,
) -> Result<Vec<FacetPanel>, Error> {
    let configured_source_rows = plot.configured_source_rows()?;
    let mut panels = Vec::new();
    crate::data::source::reserve_source(&mut panels, selectors.cells.len())?;
    for (panel_index, (row, column, cell)) in selectors.cells.iter().enumerate() {
        let columns_data = crate::data::source::collect_source(cell.iter().map(|selector| {
            let column = plot.data_column(selector.column).ok_or_else(|| {
                Error::Data(format!("unknown semantic value {:?}", selector.column))
            })?;
            let crate::data::DataColumn::Category(values) = column else {
                return Err(Error::Data(format!(
                    "facet: value {:?} must be categorical",
                    column.name()
                )));
            };
            crate::data::strict_source_strings(values, configured_source_rows, "facet")
        }))?;
        let mut source_indices = Vec::new();
        crate::data::source::reserve_source(&mut source_indices, plot.row_count())?;
        for index in 0..plot.row_count() {
            let mut matches = true;
            for (selector_index, selector) in cell.iter().enumerate() {
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
        let rows = select_rows(plot, source_indices)?;
        let panel = provisional.map_or(PixelRect::new(0.0, 1.0, 0.0, 1.0), |boxes| {
            boxes[*row * selectors.columns + *column]
        });
        panels.push(FacetPanel {
            panel_index,
            row: *row,
            column: *column,
            cell: crate::data::source::collect_source(cell.iter().map(|selector| {
                Ok(FacetSelector {
                    column: selector.column,
                    variable: crate::data::source::copy_source_string(&selector.variable)?,
                    level: crate::data::source::copy_source_string(&selector.level)?,
                })
            }))?,
            rows,
            panel,
        });
    }
    Ok(panels)
}

pub(super) fn validate_significance_facet_anchors(
    plot: &PlotDescription,
    panels: &[FacetPanel],
) -> Result<(), Error> {
    let dimensions = panels.first().map_or(0, |panel| panel.cell.len());
    for annotation in plot.significance_annotations() {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_facet_extraction_preserves_sparse_slots_and_selected_diagnostic_rows() {
        let source = Plot::scatter(
            crate::NumericValues::new("x", [0.0, 1.0, 2.0, 3.0]),
            crate::NumericValues::new("y", [3.0, 2.0, 1.0, 0.0]),
        )
        .source_rows(crate::SourceRows::new([91, 700, 9001, 32]).unwrap())
        .facet(
            crate::FacetSpec::grid(
                crate::CategoryValues::new("row", ["B", "A", "B", "A"]).unwrap(),
                crate::CategoryValues::new("column", ["C", "D", "C", "D"]).unwrap(),
            )
            .unwrap(),
        );
        let selectors = facet_selectors(source.description()).unwrap();
        assert_eq!(
            (selectors.rows, selectors.columns, selectors.cells.len()),
            (2, 2, 4)
        );
        let panels = extract_panels(source.description(), &selectors, None).unwrap();
        assert_eq!(panels.len(), 2);
        assert_eq!(
            (panels[0].panel_index, panels[0].row, panels[0].column),
            (0, 0, 0)
        );
        assert_eq!(
            (panels[1].panel_index, panels[1].row, panels[1].column),
            (3, 1, 1)
        );
        assert_eq!(
            panels[0].rows.selected(),
            Some((&[0, 2][..], &[91, 9001][..]))
        );
        assert_eq!(
            panels[1].rows.selected(),
            Some((&[1, 3][..], &[700, 32][..]))
        );
        assert_eq!(panels[0].render_scope().unwrap(), "facet row=B, column=C");
    }

    #[test]
    fn source_facet_selector_retains_mapped_strict_read_errors() {
        let source = Plot::scatter(
            crate::NumericValues::new("x", [0.0, 1.0]),
            crate::NumericValues::new("y", [1.0, 0.0]),
        )
        .source_rows(crate::SourceRows::new([90, 7001]).unwrap())
        .facet(crate::FacetSpec::column(
            crate::CategoryValues::nullable("facet", [Some("first"), None]).unwrap(),
        ));
        let error = facet_selectors(source.description()).err().unwrap();
        assert_eq!(
            error.to_string(),
            "data error: facet: value \"facet\" contains null at source row 7001"
        );
    }
}
