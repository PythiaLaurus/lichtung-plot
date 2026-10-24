//! Facet Grid assignments retain sparse coordinates and separate measured
//! decoration gutters from the aligned data-panel tracks.

use super::data::FacetPanel;
use crate::data::PlotData;
use crate::layout::canvas::PanelRequirements;
use crate::layout::fit::{FitOutcome, FitScope};
use crate::layout::kernel::{
    allocate_grid_axis, nearest_spatial_quanta, FitKind, GridRequirement, TrackSpec,
};
use crate::layout::{AddressRole, ComponentAddress, PanelBox};
use crate::theme::ResolvedTheme;
use crate::Error;

#[expect(
    clippy::cast_precision_loss,
    reason = "all facet tracks remain within the public u32 root"
)]
fn pixels(quanta: i64) -> f64 {
    quanta as f64 / 1024.0
}

fn required(value: f64) -> Result<i64, Error> {
    nearest_spatial_quanta(crate::layout::kernel::required_length(value)?)
}

fn checked(value: i128) -> Result<i64, Error> {
    i64::try_from(value).map_err(|_| Error::Render("facet track arithmetic overflowed".into()))
}

pub(super) struct ContentSizing<'a> {
    pub plot: &'a crate::Plot,
    pub inputs: &'a crate::layout::interior::PreparedInteriorInputs,
    pub round: &'a crate::layout::kernel::FeedbackRound,
    pub natural_width: bool,
    pub natural_height: bool,
}

pub(super) struct GridAssignment {
    pub cells: Vec<PanelBox>,
    pub size: (f64, f64),
}

#[expect(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    reason = "one measured facet Grid retains row-major sparse cells, gutters, and both axis vectors"
)]
pub(super) fn allocate(
    data: PlotData<'_>,
    outer: PanelBox,
    shape: (usize, usize),
    panels: &[FacetPanel],
    minima: &[(f64, f64)],
    gutters: &[(usize, usize, PanelRequirements)],
    theme: &ResolvedTheme,
    scope: &FitScope,
    content: Option<&ContentSizing<'_>>,
) -> Result<FitOutcome<GridAssignment>, Error> {
    let (rows, columns) = shape;
    if rows == 0 || columns == 0 || panels.len() != minima.len() {
        return Err(Error::Render(
            "facet Grid has invalid shape or intrinsic ownership".into(),
        ));
    }
    let count = rows
        .checked_mul(columns)
        .ok_or_else(|| Error::Render("facet Grid topology overflowed".into()))?;
    let resources = theme.prepared_font_environment()?.operation();
    let data = data.with_operation(&resources);
    let mut column_insets = Vec::new();
    let mut row_insets = Vec::new();
    crate::data::source::reserve_source(&mut column_insets, columns)?;
    crate::data::source::reserve_source(&mut row_insets, rows)?;
    column_insets.resize(columns, [0_i64; 2]);
    row_insets.resize(rows, [0_i64; 2]);
    let gap = required(theme.composition_facet_gap()?)?;
    let em = required(theme.typography.composition.facet_header.get_font_size())?;
    let strip_lengths = theme.facet_strip_lengths()?;
    if let Some(content) = content.filter(|content| content.natural_width) {
        // Initial root width owns the same external axis columns as assigned
        // layout. Feedback can refine those requirements, but cannot grow a
        // root whose natural selection omitted them entirely.
        for panel in panels {
            let plot = content
                .plot
                .clone()
                .with_render_scope(panel.render_scope()?);
            let (left, right) =
                crate::layout::interior::sizing::horizontal_gutter_preference(&plot, theme)?;
            let insets = column_insets.get_mut(panel.column).ok_or_else(|| {
                Error::Render("facet child is outside declared Grid topology".into())
            })?;
            insets[0] = insets[0].max(required(left)?);
            insets[1] = insets[1].max(required(right)?);
        }
    }
    for &(row, column, req) in gutters {
        if row >= rows || column >= columns {
            return Err(Error::Render(
                "facet gutter is outside declared Grid topology".into(),
            ));
        }
        let col = &mut column_insets[column];
        col[0] = col[0].max(required(req.left)?);
        col[1] = col[1].max(required(req.right)?);
        let row_track = &mut row_insets[row];
        row_track[0] = row_track[0].max(required(req.bottom)?);
        row_track[1] = row_track[1].max(required(req.top)?);
    }
    let grid = ComponentAddress::root().child(AddressRole::FacetGrid, 0)?;
    let mut owners = Vec::new();
    crate::data::source::reserve_source(&mut owners, panels.len())?;
    for panel in panels {
        if panel.row >= rows || panel.column >= columns {
            return Err(Error::Render(
                "facet child is outside declared Grid topology".into(),
            ));
        }
        owners.push(
            grid.child(
                AddressRole::FacetCell,
                u32::try_from(panel.panel_index)
                    .map_err(|_| Error::Render("facet cell address overflowed".into()))?,
            )?,
        );
    }
    let mut widths = Vec::new();
    let mut heights = Vec::new();
    crate::data::source::reserve_source(&mut widths, panels.len())?;
    crate::data::source::reserve_source(&mut heights, panels.len())?;
    for ((panel, minimum), address) in panels.iter().zip(minima).zip(&owners) {
        let preferred = if let Some(content) = content {
            let plot = content
                .plot
                .clone()
                .with_render_scope(panel.render_scope()?);
            let preference =
                crate::layout::interior::sizing::body_preference(&plot, theme, *minimum)?;
            let guided = if matches!(
                plot.kind(),
                crate::PlotKind::Heatmap | crate::PlotKind::Beeswarm
            ) {
                crate::layout::interior::sizing::guided_preference(&plot, content.inputs, theme)?
            } else {
                None
            };
            let preferred = if let Some(guide) = &guided {
                let size = guide.preference(theme)?;
                if guide.horizontal() {
                    preference.0.max(size.0)
                } else {
                    preference.0 + size.0 + theme.legend_outside_gap()?
                }
            } else {
                preference.0
            };
            let label = super::rendering::facet_label(&plot, &panel.cell, data)?;
            let appearance = &theme.typography.composition.facet_header;
            preferred.max(crate::theme::composition::hard(
                &strip_lengths.horizontal_inset.add_pixels(
                    crate::layout::interior::sizing::text_width(theme, &label, appearance)?,
                )?,
            )?)
        } else {
            minimum.0
        };
        widths.push(GridRequirement {
            start: panel.column,
            span: 1,
            hard: required(minimum.0)?,
            preferred: required(preferred.max(minimum.0))?,
            fill: true,
            address,
        });
        heights.push(GridRequirement {
            start: panel.row,
            span: 1,
            hard: required(minimum.1)?,
            preferred: required(minimum.1)?,
            fill: true,
            address,
        });
    }
    let x0 = nearest_spatial_quanta(outer.x0)?;
    let x1 = nearest_spatial_quanta(outer.x1)?;
    let y0 = nearest_spatial_quanta(outer.y0)?;
    let y1 = nearest_spatial_quanta(outer.y1)?;
    let horizontal_reserved = column_insets
        .iter()
        .flat_map(|edges| edges.iter())
        .map(|&value| i128::from(value))
        .sum::<i128>();
    let vertical_reserved = row_insets
        .iter()
        .flat_map(|edges| edges.iter())
        .map(|&value| i128::from(value))
        .sum::<i128>();
    let width = i128::from(x1) - i128::from(x0) - horizontal_reserved;
    let height = i128::from(y1) - i128::from(y0) - vertical_reserved;
    if (width <= 0 && !content.is_some_and(|value| value.natural_width))
        || (height <= 0 && !content.is_some_and(|value| value.natural_height))
    {
        return Ok(FitOutcome::Miss(scope.failure(
            FitKind::TrackOrResidual,
            "facet gutters leave no room for declared tracks",
        )));
    }
    let column_base = TrackSpec {
        min: 0,
        preferred: 0,
        max: None,
        flexible: false,
    };
    let row_base = TrackSpec {
        min: 0,
        preferred: 0,
        max: None,
        flexible: false,
    };
    let mut column_specs = Vec::new();
    let mut row_specs = Vec::new();
    crate::data::source::reserve_source(&mut column_specs, columns)?;
    crate::data::source::reserve_source(&mut row_specs, rows)?;
    column_specs.resize(columns, column_base);
    row_specs.resize(rows, row_base);
    let columns = match allocate_grid_axis(
        &column_specs,
        &widths,
        if content.is_some_and(|value| value.natural_width) {
            None
        } else {
            Some(checked(width)?)
        },
        [0, 0],
        gap,
        Some(em),
        true,
        scope,
    )? {
        FitOutcome::Measured(value) => value.sizes,
        FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
    };
    if let Some(content) = content {
        for (index, ((panel, child), minimum)) in
            panels.iter().zip(&mut heights).zip(minima).enumerate()
        {
            let mut plot = content
                .plot
                .clone()
                .with_render_scope(panel.render_scope()?);
            let address = ComponentAddress::root().child(
                AddressRole::FacetCell,
                u32::try_from(index)
                    .map_err(|_| Error::Render("facet content domain address overflowed".into()))?,
            )?;
            if let Some(domain) = content.round.feedback_domain(
                &address,
                crate::layout::kernel::FeedbackKind::AutomaticDomainX,
            ) {
                let domain = plot
                    .x_axis_spec()
                    .limits
                    .map_or(domain, |base| (base.0.min(domain.0), base.1.max(domain.1)));
                plot = plot.with_planned_x_limits(domain);
            }
            if let Some(domain) = content.round.feedback_domain(
                &address,
                crate::layout::kernel::FeedbackKind::AutomaticDomainY,
            ) {
                let domain = plot
                    .y_axis_spec()
                    .limits
                    .map_or(domain, |base| (base.0.min(domain.0), base.1.max(domain.1)));
                plot = plot.with_planned_y_limits(domain);
            }
            let preference =
                crate::layout::interior::sizing::body_preference(&plot, theme, *minimum)?;
            let width = pixels(columns[panel.column]);
            let guide = if matches!(
                plot.kind(),
                crate::PlotKind::Heatmap | crate::PlotKind::Beeswarm
            ) {
                crate::layout::interior::sizing::guided_preference(&plot, content.inputs, theme)?
            } else {
                None
            };
            let mut height = if guide.is_some() {
                crate::layout::interior::sizing::guided_body_height(
                    &plot,
                    theme,
                    width,
                    guide.as_ref(),
                    scope,
                )?
            } else {
                plot.natural_body_height(theme, width, preference.1)?
            };
            if row_insets[panel.row][1] == 0 {
                let label = super::rendering::facet_label(&plot, &panel.cell, data)?;
                let appearance = &theme.typography.composition.facet_header;
                let strip = crate::layout::fit::measured(crate::layout::fit::text(
                    theme,
                    &label,
                    appearance,
                    crate::theme::composition::capacity(width, &strip_lengths.horizontal_inset)?,
                    crate::layout::PhysicalTextAlignment::Center,
                    "natural facet strip",
                    scope,
                ))
                .map_err(crate::layout::fit::PlanError::into_error)?;
                height = crate::theme::composition::hard(
                    &strip_lengths
                        .content_clearance
                        .add_pixels(strip.height())?
                        .add_pixels(height)?,
                )?;
            }
            child.preferred = required(height)?.max(child.hard);
        }
    }
    let rows = match allocate_grid_axis(
        &row_specs,
        &heights,
        if content.is_some_and(|value| value.natural_height) {
            None
        } else {
            Some(checked(height)?)
        },
        [0, 0],
        gap,
        Some(em),
        true,
        scope,
    )? {
        FitOutcome::Measured(value) => value.sizes,
        FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
    };
    let grid_width = checked(
        columns.iter().map(|&value| i128::from(value)).sum::<i128>()
            + horizontal_reserved
            + i128::from(gap)
                * i128::try_from(columns.len().saturating_sub(1))
                    .map_err(|_| Error::Render("facet gap count overflowed".into()))?,
    )?;
    let grid_height = checked(
        rows.iter().map(|&value| i128::from(value)).sum::<i128>()
            + vertical_reserved
            + i128::from(gap)
                * i128::try_from(rows.len().saturating_sub(1))
                    .map_err(|_| Error::Render("facet gap count overflowed".into()))?,
    )?;
    let y1 = if content.is_some_and(|value| value.natural_height) {
        checked(i128::from(y0) + i128::from(grid_height))?
    } else {
        y1
    };
    let mut x_positions = Vec::new();
    let mut y_positions = Vec::new();
    crate::data::source::reserve_source(&mut x_positions, columns.len())?;
    crate::data::source::reserve_source(&mut y_positions, rows.len())?;
    let mut cursor = i128::from(x0);
    for (index, &width) in columns.iter().enumerate() {
        cursor += i128::from(column_insets[index][0]);
        x_positions.push(checked(cursor)?);
        cursor += i128::from(width) + i128::from(column_insets[index][1]);
        if index + 1 < columns.len() {
            cursor += i128::from(gap);
        }
    }
    let mut cursor = i128::from(y1);
    for (index, &height) in rows.iter().enumerate() {
        cursor -= i128::from(row_insets[index][1]);
        y_positions.push(checked(cursor)?);
        cursor -= i128::from(height) + i128::from(row_insets[index][0]);
        if index + 1 < rows.len() {
            cursor -= i128::from(gap);
        }
    }
    let mut cells = Vec::new();
    crate::data::source::reserve_source(&mut cells, count)?;

    for &top in &y_positions {
        for (&left, &width) in x_positions.iter().zip(&columns) {
            let row = cells.len() / columns.len();
            cells.push(PanelBox::new(
                pixels(left),
                pixels(checked(i128::from(left) + i128::from(width))?),
                pixels(checked(i128::from(top) - i128::from(rows[row]))?),
                pixels(top),
            ));
        }
    }
    Ok(FitOutcome::Measured(GridAssignment {
        cells,
        size: (pixels(grid_width), pixels(grid_height)),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn declared_empty_facet_tracks_use_only_facet_header_em_and_shrink_to_one_quantum() {
        let plot = crate::Plot::scatter(
            crate::NumericValues::new("x", [0.0, 1.0]),
            crate::NumericValues::new("y", [1.0, 0.0]),
        )
        .erase();
        let resources = crate::font::canonical::OperationContext::new();
        let inputs = crate::plots::shared_x::prepare_plot(&plot, &resources, None).unwrap();
        let mut theme = crate::Theme::publication().resolve().unwrap();
        theme.composition = theme
            .composition
            .clone()
            .facet_gap(crate::theme::RelativeSpacing::ratio(0, 1).unwrap())
            .unwrap();
        theme.typography.composition.body = theme
            .typography
            .composition
            .body
            .clone()
            .font_size(34.0)
            .unwrap();
        theme.typography.composition.composition_header = theme
            .typography
            .composition
            .composition_header
            .clone()
            .font_size(40.0)
            .unwrap();
        for em in [15, 30] {
            theme.typography.composition.facet_header = theme
                .typography
                .composition
                .facet_header
                .clone()
                .font_size(f64::from(em))
                .unwrap();
            crate::layout::plan_exact_root(
                1,
                1,
                |_| Ok(()),
                |(), assigned, round| {
                    let FitOutcome::Measured(grid) = allocate(
                        plot.data(),
                        PanelBox::new(0.0, 0.0, 0.0, 0.0),
                        (1, 2),
                        &[],
                        &[],
                        &[],
                        &theme,
                        &FitScope::root(),
                        Some(&ContentSizing {
                            plot: &plot,
                            inputs: &inputs,
                            round,
                            natural_width: true,
                            natural_height: true,
                        }),
                    )?
                    else {
                        panic!("declared facet tracks must retain their natural preference");
                    };
                    assert_eq!(grid.size.0.to_bits(), (2.0 * f64::from(em)).to_bits());
                    assert_eq!(grid.size.1.to_bits(), f64::from(em).to_bits());
                    let FitOutcome::Measured(exact) = allocate(
                        plot.data(),
                        PanelBox::new(0.0, 2.0 / 1024.0, 0.0, 1.0 / 1024.0),
                        (1, 2),
                        &[],
                        &[],
                        &[],
                        &theme,
                        &FitScope::root(),
                        None,
                    )?
                    else {
                        panic!("declared em is a preference, not a minimum");
                    };
                    assert!(exact
                        .cells
                        .iter()
                        .all(|cell| cell.width().to_bits() == (1.0_f64 / 1024.0).to_bits()));
                    Ok(crate::layout::MeasuredLayout::new(assigned, ()))
                },
                |(), _, (), _| Ok(()),
            )
            .unwrap();
        }
    }

    #[test]
    fn grid_allocation_preserves_incoming_scoped_rows_without_source_reads() {
        use crate::font::canonical::OperationContext;
        let plot = crate::Plot::scatter(
            crate::NumericValues::new("x", [0.0, 1.0, 2.0, 3.0]),
            crate::NumericValues::new("y", [4.0, 5.0, 6.0, 7.0]),
        )
        .erase();
        let rows = [1, 3];
        let ids = [7, 29];
        {
            let theme = crate::Theme::publication().resolve().unwrap();

            let foreign = OperationContext::new();
            let data = PlotData::with_rows(&plot, &rows, &ids).with_operation(&foreign);

            let reads = crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get);
            let result = allocate(
                data,
                PanelBox::new(0.0, 400.0, 0.0, 200.0),
                (2, 2),
                &[],
                &[],
                &[],
                &theme,
                &FitScope::root(),
                None,
            );
            let FitOutcome::Measured(grid) = result.unwrap() else {
                panic!("empty facet grid should fit")
            };
            assert_eq!(grid.cells.len(), 4);

            assert_eq!(data.height(), 2);
            assert_eq!([data.source_row(0), data.source_row(1)], ids);
            assert_eq!(
                crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get),
                reads
            );
        }
    }

    #[test]
    fn asymmetric_intrinsics_and_gutters_preserve_sparse_column_positions() {
        let plot = crate::Plot::scatter(
            crate::NumericValues::new("x", [1.0, 2.0]),
            crate::NumericValues::new("y", [3.0, 4.0]),
        )
        .erase();
        let theme = crate::Theme::publication().resolve().unwrap();
        let panels = [0, 1].map(|index| FacetPanel {
            panel_index: index,
            row: 0,
            column: index,
            cell: Vec::new(),
            rows: vec![index],
            source_rows: vec![index],
            panel: PanelBox::new(0.0, 1.0, 0.0, 1.0),
        });
        let gutters = [
            (
                0,
                0,
                PanelRequirements {
                    left: 20.0,
                    right: 5.0,
                    bottom: 0.0,
                    top: 0.0,
                },
            ),
            (
                0,
                1,
                PanelRequirements {
                    left: 5.0,
                    right: 20.0,
                    bottom: 0.0,
                    top: 0.0,
                },
            ),
        ];
        let FitOutcome::Measured(cells) = allocate(
            plot.data(),
            PanelBox::new(0.0, 400.0, 0.0, 200.0),
            (1, 2),
            &panels,
            &[(80.0, 60.0), (120.0, 60.0)],
            &gutters,
            &theme,
            &FitScope::root(),
            None,
        )
        .unwrap() else {
            panic!("asymmetric measured tracks must fit");
        };
        let cells = cells.cells;
        assert_eq!(cells.len(), 2);
        assert_eq!(cells[0].x0.to_bits(), 20.0_f64.to_bits());
        assert_eq!(cells[0].width().to_bits(), 149.0_f64.to_bits());
        assert_eq!(cells[1].x0.to_bits(), 191.0_f64.to_bits());
        assert_eq!(cells[1].width().to_bits(), 189.0_f64.to_bits());
        assert_eq!(cells[0].y0.to_bits(), cells[1].y0.to_bits());
    }
}
