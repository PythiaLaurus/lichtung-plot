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
) -> Result<FitOutcome<Vec<PanelBox>>, Error> {
    let (rows, columns) = shape;
    if rows == 0 || columns == 0 || panels.len() != minima.len() {
        return Err(Error::Render(
            "facet Grid has invalid shape or intrinsic ownership".into(),
        ));
    }
    let count = rows
        .checked_mul(columns)
        .ok_or_else(|| Error::Render("facet Grid topology overflowed".into()))?;
    let resources = theme.prepared_font_environment()?.generation_resources();
    let gap = required(theme.composition.facet_gap)?;
    let em = required(f64::from(theme.facet_header_font_size))?;
    let mut column_insets = Vec::new();
    let mut row_insets = Vec::new();
    data.reserve_source(&mut column_insets, columns)?;
    data.reserve_source(&mut row_insets, rows)?;
    column_insets.resize(columns, [0_i64; 2]);
    row_insets.resize(rows, [0_i64; 2]);
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
    data.reserve_source(&mut owners, panels.len())?;
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
    data.reserve_source(&mut widths, panels.len())?;
    data.reserve_source(&mut heights, panels.len())?;
    for ((panel, minimum), address) in panels.iter().zip(minima).zip(&owners) {
        widths.push(GridRequirement {
            start: panel.column,
            span: 1,
            hard: required(minimum.0)?,
            preferred: required(minimum.0)?,
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
    if width <= 0 || height <= 0 {
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
    data.reserve_source(&mut column_specs, columns)?;
    data.reserve_source(&mut row_specs, rows)?;
    column_specs.resize(columns, column_base);
    row_specs.resize(rows, row_base);
    let columns = match allocate_grid_axis(
        &column_specs,
        &widths,
        checked(width)?,
        [0, 0],
        gap,
        Some(em),
        true,
        &resources,
        scope,
    )? {
        FitOutcome::Measured(value) => value.sizes,
        FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
    };
    let rows = match allocate_grid_axis(
        &row_specs,
        &heights,
        checked(height)?,
        [0, 0],
        gap,
        Some(em),
        true,
        &resources,
        scope,
    )? {
        FitOutcome::Measured(value) => value.sizes,
        FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
    };
    let mut x_positions = Vec::new();
    let mut y_positions = Vec::new();
    data.reserve_source(&mut x_positions, columns.len())?;
    data.reserve_source(&mut y_positions, rows.len())?;
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
    data.reserve_source(&mut cells, count)?;
    data.claim_source_visits(count)?;
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
    Ok(FitOutcome::Measured(cells))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asymmetric_intrinsics_and_gutters_preserve_sparse_column_positions() {
        let plot = crate::Plot::scatter(
            crate::NumericValues::new("x", [1.0, 2.0]),
            crate::NumericValues::new("y", [3.0, 4.0]),
        )
        .erase();
        let theme = crate::Theme::Publication.resolve();
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
        )
        .unwrap() else {
            panic!("asymmetric measured tracks must fit");
        };
        assert_eq!(cells.len(), 2);
        assert_eq!(cells[0].x0.to_bits(), 20.0_f64.to_bits());
        assert_eq!(cells[0].width().to_bits(), 149.0_f64.to_bits());
        assert_eq!(cells[1].x0.to_bits(), 191.0_f64.to_bits());
        assert_eq!(cells[1].width().to_bits(), 189.0_f64.to_bits());
        assert_eq!(cells[0].y0.to_bits(), cells[1].y0.to_bits());
    }
}
