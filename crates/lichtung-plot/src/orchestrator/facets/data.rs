//! Facet selector construction, row extraction, and anchor validation.

use crate::Error;

use crate::builder::PlotDescription;
use crate::orchestrator::registration::{select_rows, RowSelection};
use crate::orchestrator::root::{
    DeclaredFacetSelector, FacetSelectors, FacetSlot, FacetTopologyShell, GridSlot,
};
use crate::spec::PlotKind;
use crate::FacetCell;

pub(super) fn fields<'a>(
    source: &'a PlotDescription,
    cell: &'a FacetCell,
) -> Vec<(&'static str, &'a crate::CategoryValues, &'a str)> {
    let (row, column, wrap, _) = source.facet_columns();
    let levels = if let Some((row, column)) = cell.grid_levels() {
        vec![row, column]
    } else {
        vec![cell.single_level().expect("single facet selector")]
    };
    [row, column, wrap]
        .into_iter()
        .flatten()
        .zip(levels)
        .map(|(key, level)| {
            let crate::data::DataColumn::Category(values) =
                source.data_column(key).expect("validated facet column")
            else {
                unreachable!("categorical facet")
            };
            (key, values, level)
        })
        .collect()
}

pub(super) fn render_scope(source: &PlotDescription, cell: &FacetCell) -> Result<String, Error> {
    let mut scope = String::new();
    crate::data::source::reserve_source_string(&mut scope, 32)?;
    scope.push_str("facet ");
    for (index, (_, values, level)) in fields(source, cell).into_iter().enumerate() {
        let variable = values.name();
        let length = variable
            .len()
            .checked_add(level.len())
            .and_then(|length| length.checked_add(3))
            .ok_or_else(|| Error::Render("facet scope length overflow".into()))?;
        crate::data::source::reserve_source_string(&mut scope, length)?;
        if index != 0 {
            scope.push_str(", ");
        }
        scope.push_str(variable);
        scope.push('=');
        scope.push_str(level);
    }
    Ok(scope)
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
        crate::data::source::collect_source(levels.into_iter().enumerate().map(
            |(index, level)| {
                Ok((
                    index / columns,
                    index % columns,
                    FacetCell::single(level),
                    DeclaredFacetSelector::Single { level: index },
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
                let (cell, selector) = match (row_column, column_column) {
                    (Some(_), Some(_)) => (
                        FacetCell::grid(
                            crate::data::source::copy_source_string(row_level)?,
                            crate::data::source::copy_source_string(column_level)?,
                        ),
                        DeclaredFacetSelector::Grid {
                            row_level: row_index,
                            column_level: column_index,
                        },
                    ),
                    (Some(_), None) => (
                        FacetCell::single(crate::data::source::copy_source_string(row_level)?),
                        DeclaredFacetSelector::Single { level: row_index },
                    ),
                    (None, Some(_)) => (
                        FacetCell::single(crate::data::source::copy_source_string(column_level)?),
                        DeclaredFacetSelector::Single {
                            level: column_index,
                        },
                    ),
                    (None, None) => unreachable!("faceted source"),
                };
                cells.push((row_index, column_index, cell, selector));
            }
        }
        cells
    };
    if cells.is_empty() {
        return Err(Error::Data(
            "faceting requires at least one observed level".into(),
        ));
    }

    let rows = cells.iter().map(|(row, _, _, _)| *row).max().unwrap_or(0) + 1;
    let columns = cells
        .iter()
        .map(|(_, column, _, _)| *column)
        .max()
        .unwrap_or(0)
        + 1;
    let count = rows
        .checked_mul(columns)
        .ok_or_else(|| Error::Render("facet cell count overflow".into()))?;
    let mut slots = Vec::new();
    crate::data::source::reserve_source(&mut slots, count)?;
    slots.resize_with(count, || FacetSlot {
        selector: None,
        plot_id: None,
    });
    let cells = crate::data::source::collect_source(cells.into_iter().map(
        |(row, column, cell, selector)| {
            let slot = GridSlot(row * columns + column);
            slots[slot.0].selector = Some(selector);
            Ok((slot, cell))
        },
    ))?;
    Ok(FacetSelectors {
        shell: FacetTopologyShell {
            rows,
            columns,
            slots,
        },
        cells,
    })
}

#[expect(
    clippy::type_complexity,
    reason = "the original parent scans return their owning group and pair results directly for publication at the guide boundary"
)]
pub(super) fn parent_orders(
    source: &PlotDescription,
    kind: PlotKind,
) -> Result<
    (
        Option<Vec<String>>,
        Option<Vec<crate::data::CollectivePair>>,
        Option<Vec<String>>,
    ),
    Error,
> {
    let source_rows = source.configured_source_rows()?;
    let mut collectives = None;
    let mut groups = None;
    let mut hues = None;
    let independent_collectives = matches!(
        kind,
        PlotKind::Density | PlotKind::Line | PlotKind::Interaction
    ) || kind == PlotKind::LinePath
        && source.line_path_style_spec() == crate::LinePathStyle::Hue;
    if independent_collectives {
        let channels =
            crate::data::observed_source_collectives(source, "facet collective channels")?;
        groups = Some(channels.group_levels);
        hues = Some(channels.hue_levels);
        collectives = Some(channels.identities);
    } else if let Some(group_column) = source.group_col() {
        let column = source
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
        groups = Some(levels);
    }
    if kind == PlotKind::Trajectory {
        if let Some(x_column) = source.x_col() {
            let column = source
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
    Ok((groups, collectives, hues))
}

#[expect(
    clippy::needless_range_loop,
    clippy::type_complexity,
    reason = "the fixed shell and cell/row tuples preserve ownership while each row index addresses every selector column"
)]
pub(super) fn extract_facet_rows(
    plot: &PlotDescription,
    selectors: FacetSelectors,
) -> Result<(FacetTopologyShell, Vec<(GridSlot, FacetCell, RowSelection)>), Error> {
    let configured_source_rows = plot.configured_source_rows()?;
    let mut panels = Vec::new();
    crate::data::source::reserve_source(&mut panels, selectors.cells.len())?;
    for (slot, cell) in selectors.cells {
        let fields = fields(plot, &cell);
        let columns_data =
            crate::data::source::collect_source(fields.iter().map(|(key, _, _)| {
                let column = plot
                    .data_column(key)
                    .ok_or_else(|| Error::Data(format!("unknown semantic value {key:?}")))?;
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
            for (selector_index, (_, _, level)) in fields.iter().enumerate() {
                if columns_data[selector_index][index] != *level {
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
        panels.push((slot, cell, rows));
    }
    Ok((selectors.shell, panels))
}

pub(super) fn validate_significance_facet_anchors(
    plot: &PlotDescription,
    panels: &[(GridSlot, FacetCell, RowSelection)],
) -> Result<(), Error> {
    let dimensions = panels.first().map_or(0, |(_, cell, _)| cell.dimensions());
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
        for (_, panel, _) in panels {
            if cell == panel {
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
    use crate::Plot;

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
            (
                selectors.shell.rows,
                selectors.shell.columns,
                selectors.cells.len()
            ),
            (2, 2, 4)
        );
        let (shell, panels) = extract_facet_rows(source.description(), selectors).unwrap();
        assert_eq!(panels.len(), 2);
        assert_eq!(
            (
                panels[0].0 .0,
                panels[0].0 .0 / shell.columns,
                panels[0].0 .0 % shell.columns
            ),
            (0, 0, 0)
        );
        assert_eq!(
            (
                panels[1].0 .0,
                panels[1].0 .0 / shell.columns,
                panels[1].0 .0 % shell.columns
            ),
            (3, 1, 1)
        );
        assert_eq!(panels[0].2.selected(), Some((&[0, 2][..], &[91, 9001][..])));
        assert_eq!(panels[1].2.selected(), Some((&[1, 3][..], &[700, 32][..])));
        assert_eq!(
            render_scope(source.description(), &panels[0].1).unwrap(),
            "facet row=B, column=C"
        );
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
