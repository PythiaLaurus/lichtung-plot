//! Canvas Grid tracks use the same quantized allocator as facet and TextPanel
//! owners while keeping each Plot's theme and declared sparse coordinates.

use super::{text_panel::TextPanelIntrinsics, CanvasHeader, ResolvedCanvasThemes};
use crate::layout::canvas::PanelRequirements;
use crate::layout::fit::{FitOutcome, FitScope};
use crate::layout::kernel::{
    allocate_grid_axis, nearest_spatial_quanta, FitKind, GridRequirement, TrackSpec,
};
use crate::layout::{AddressRole, ComponentAddress, PanelBox};
use crate::{Error, Plot};

#[expect(
    clippy::cast_precision_loss,
    reason = "Canvas tracks are bounded by exact public u32 dimensions"
)]
fn pixels(quanta: i64) -> f64 {
    quanta as f64 / 1024.0
}

fn required(value: f64) -> Result<i64, Error> {
    nearest_spatial_quanta(crate::layout::kernel::required_length(value)?)
}

fn checked(value: i128) -> Result<i64, Error> {
    i64::try_from(value).map_err(|_| Error::Render("Canvas track arithmetic overflowed".into()))
}

#[expect(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    reason = "one Canvas Grid owns canonical sparse topology, root TextPanels, per-Plot minima, and measured gutters"
)]
pub(super) fn allocate(
    outer: PanelBox,
    shape: (usize, usize),
    plots: &[Option<Plot>],
    text_panels: &[Option<TextPanelIntrinsics>],
    plot_titles: &[Option<crate::decorations::title_block::plan::RootTitleBlockIntrinsics>],
    column_headers: &[CanvasHeader],
    gutters: &[(usize, usize, PanelRequirements)],
    themes: &ResolvedCanvasThemes,
    scope: &FitScope,
) -> Result<FitOutcome<Vec<PanelBox>>, Error> {
    let (rows, columns) = shape;
    let count = rows
        .checked_mul(columns)
        .ok_or_else(|| Error::Render("Canvas Grid topology overflowed".into()))?;
    if rows == 0
        || columns == 0
        || plots.len() != count
        || text_panels.len() != count
        || plot_titles.len() != count
    {
        return Err(Error::Render(
            "Canvas Grid has mismatched declared slots".into(),
        ));
    }
    let theme = themes.root();
    let resources = theme.prepared_font_environment()?.generation_resources();
    let gap = required(theme.composition.canvas_grid_gap)?;
    let em = required(f64::from(theme.composition_title_font_size))?;
    let mut column_insets = Vec::new();
    let mut row_insets = Vec::new();
    crate::layout::fit::reserve(theme, &mut column_insets, columns, "Canvas column gutters")?;
    crate::layout::fit::reserve(theme, &mut row_insets, rows, "Canvas row gutters")?;
    column_insets.resize(columns, [0_i64; 2]);
    row_insets.resize(rows, [0_i64; 2]);
    for &(row, column, requirement) in gutters {
        if row >= rows || column >= columns {
            return Err(Error::Render(
                "Canvas gutter lies outside its declared Grid".into(),
            ));
        }
        let col = &mut column_insets[column];
        col[0] = col[0].max(required(requirement.left)?);
        col[1] = col[1].max(required(requirement.right)?);
        let row = &mut row_insets[row];
        row[0] = row[0].max(required(requirement.bottom)?);
        row[1] = row[1].max(required(requirement.top)?);
    }
    let grid = ComponentAddress::root().child(AddressRole::CanvasGrid, 0)?;
    let mut owners = Vec::new();
    crate::layout::fit::reserve(theme, &mut owners, count, "Canvas cell owners")?;
    for index in 0..count {
        owners.push(
            grid.child(
                AddressRole::CanvasCell,
                u32::try_from(index)
                    .map_err(|_| Error::Render("Canvas cell address overflowed".into()))?,
            )?,
        );
    }
    let mut widths = Vec::new();
    let mut heights = Vec::new();
    let width_count = count
        .checked_add(column_headers.len())
        .ok_or_else(|| Error::Render("Canvas header requirement count overflowed".into()))?;
    crate::layout::fit::reserve(
        theme,
        &mut widths,
        width_count,
        "Canvas column requirements",
    )?;
    crate::layout::fit::reserve(theme, &mut heights, count, "Canvas row requirements")?;
    for index in 0..count {
        let minimum = if let Some(plot) = &plots[index] {
            let child_theme = themes
                .plot(index)
                .expect("configured Plot retains its theme");
            let mut hard = plot.intrinsic_body_minimum(child_theme)?;
            let title = plot_titles[index]
                .expect("configured Plot retains prepared text intrinsics")
                .widths();
            let present = [plot.title_text(), plot.subtitle_text(), plot.caption_text()]
                .map(|value| value.is_some_and(|text| !text.is_empty()));
            let text_insets = if present.into_iter().any(|value| value) {
                32.0
            } else {
                0.0
            };
            let spacing = &child_theme.composition.root_title_block;
            let top = if present[0] || present[1] {
                f64::from(u32::from(present[0])) * child_theme.typography.title.line_height()
                    + f64::from(u32::from(present[1]))
                        * child_theme.typography.subtitle.line_height()
                    + if present[0] && present[1] {
                        spacing.title_subtitle_gap
                    } else {
                        0.0
                    }
                    + spacing.title_block_to_content_clearance
            } else {
                0.0
            };
            let bottom = if present[2] {
                child_theme.typography.caption.line_height() + spacing.caption_to_content_clearance
            } else {
                0.0
            };
            hard.1 += top + bottom + if text_insets > 0.0 { 20.0 } else { 0.0 };
            hard.0 = hard.0.max(title.0 + text_insets);
            Some((hard, hard.0.max(title.1 + text_insets)))
        } else {
            text_panels[index].map(|text| (text.minimum, text.preferred_width))
        };
        let Some((hard, preferred_width)) = minimum else {
            continue;
        };
        widths.push(GridRequirement {
            start: index % columns,
            span: 1,
            hard: required(hard.0)?,
            preferred: required(preferred_width.max(hard.0))?,
            fill: true,
            address: &owners[index],
        });
        heights.push(GridRequirement {
            start: index / columns,
            span: 1,
            hard: required(hard.1)?,
            preferred: required(hard.1)?,
            fill: true,
            address: &owners[index],
        });
    }
    let mut header_owners = Vec::new();
    crate::layout::fit::reserve(
        theme,
        &mut header_owners,
        column_headers.len(),
        "Canvas header owners",
    )?;
    let appearance =
        theme.legacy_text_appearance(theme.composition_title_font_size, crate::TextWeight::Normal);
    for (index, header) in column_headers.iter().enumerate() {
        if header.span == 0
            || header
                .start
                .checked_add(header.span)
                .is_none_or(|end| end > columns)
        {
            return Err(Error::Render(
                "Canvas column header span is outside declared Grid tracks".into(),
            ));
        }
        header_owners.push(
            ComponentAddress::root().child(
                AddressRole::CanvasHeader,
                u32::try_from(index)
                    .map_err(|_| Error::Render("Canvas header address overflowed".into()))?,
            )?,
        );
    }
    for (header, owner) in column_headers.iter().zip(&header_owners) {
        let (hard, preferred) = crate::layout::fit::intrinsic_widths(
            theme,
            &header.text,
            &appearance,
            "Canvas column header intrinsic",
        )?;
        widths.push(GridRequirement {
            start: header.start,
            span: header.span,
            hard: required(hard)?,
            preferred: required(preferred.max(hard))?,
            fill: false,
            address: owner,
        });
    }
    let x0 = nearest_spatial_quanta(outer.x0)?;
    let x1 = nearest_spatial_quanta(outer.x1)?;
    let y0 = nearest_spatial_quanta(outer.y0)?;
    let y1 = nearest_spatial_quanta(outer.y1)?;
    let horizontal_gutters = column_insets
        .iter()
        .flat_map(|edges| edges.iter())
        .map(|&value| i128::from(value))
        .sum::<i128>();
    let vertical_gutters = row_insets
        .iter()
        .flat_map(|edges| edges.iter())
        .map(|&value| i128::from(value))
        .sum::<i128>();
    let width = i128::from(x1) - i128::from(x0) - horizontal_gutters;
    let height = i128::from(y1) - i128::from(y0) - vertical_gutters;
    if width <= 0 || height <= 0 {
        return Ok(FitOutcome::Miss(scope.failure(
            FitKind::TrackOrResidual,
            "Canvas measured gutters leave no cell tracks",
        )));
    }
    let base = TrackSpec {
        min: 0,
        preferred: 0,
        max: None,
        flexible: false,
    };
    let mut column_base = Vec::new();
    let mut row_base = Vec::new();
    crate::layout::fit::reserve(theme, &mut column_base, columns, "Canvas column tracks")?;
    crate::layout::fit::reserve(theme, &mut row_base, rows, "Canvas row tracks")?;
    column_base.resize(columns, base);
    row_base.resize(rows, base);
    let columns = match allocate_grid_axis(
        &column_base,
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
        &row_base,
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
    crate::layout::fit::reserve(theme, &mut x_positions, columns.len(), "Canvas x positions")?;
    crate::layout::fit::reserve(theme, &mut y_positions, rows.len(), "Canvas y positions")?;
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
    crate::layout::fit::reserve(theme, &mut cells, count, "Canvas assigned cells")?;
    resources
        .claim_visits(
            u128::try_from(count)
                .map_err(|_| Error::Render("Canvas cell visit count overflowed".into()))?,
        )
        .map_err(|error| Error::Render(format!("Canvas cell allocation admission: {error:?}")))?;
    for (row, &top) in y_positions.iter().enumerate() {
        for (&left, &width) in x_positions.iter().zip(&columns) {
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
    fn sparse_mixed_grid_keeps_declared_gaps_and_trailing_cells() {
        let plots: Vec<Option<Plot>> = (0..4).map(|_| None).collect();
        let plot_titles = [None; 4];
        let themes = ResolvedCanvasThemes::new(crate::Theme::Publication, &plots);
        let text = super::super::TextPanel::new("A", "B").unwrap();
        let mut intrinsics = [None; 4];
        intrinsics[0] = Some(TextPanelIntrinsics::prepare(&text, themes.root()).unwrap());
        let FitOutcome::Measured(cells) = allocate(
            PanelBox::new(0.0, 400.0, 0.0, 300.0),
            (2, 2),
            &plots,
            &intrinsics,
            &plot_titles,
            &[],
            &[],
            &themes,
            &FitScope::root(),
        )
        .unwrap() else {
            panic!("declared trailing Canvas cells must fit");
        };
        assert_eq!(cells.len(), 4);
        assert!((cells[1].x0 - cells[0].x1 - 12.0).abs() < 1.0 / 1024.0);
        assert!((cells[0].y0 - cells[2].y1 - 12.0).abs() < 1.0 / 1024.0);
        assert!(cells[3].width() > 0.0 && cells[3].height() > 0.0);
    }

    #[test]
    fn text_panels_with_asymmetric_intrinsics_get_unequal_fill_columns() {
        let plots: Vec<Option<Plot>> = [None, None].into();
        let themes = ResolvedCanvasThemes::new(crate::Theme::Publication, &plots);
        let long =
            super::super::TextPanel::new("MMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMM", "Body").unwrap();
        let short = super::super::TextPanel::new("Brief", "Body").unwrap();
        let intrinsics = [
            Some(TextPanelIntrinsics::prepare(&long, themes.root()).unwrap()),
            Some(TextPanelIntrinsics::prepare(&short, themes.root()).unwrap()),
        ];
        let FitOutcome::Measured(cells) = allocate(
            PanelBox::new(0.0, 700.0, 0.0, 320.0),
            (1, 2),
            &plots,
            &intrinsics,
            &[None; 2],
            &[],
            &[],
            &themes,
            &FitScope::root(),
        )
        .unwrap() else {
            panic!("both assigned TextPanel cells must fit");
        };
        assert!(cells[0].width() > cells[1].width());
        assert!((cells[1].x0 - cells[0].x1 - 12.0).abs() < 1.0 / 1024.0);
        assert_eq!(cells[0].y0.to_bits(), cells[1].y0.to_bits());
    }

    #[test]
    fn spanning_column_header_preserves_title_width_without_repacking_sparse_cells() {
        let plots: Vec<Option<Plot>> = [None, None, None].into();
        let themes = ResolvedCanvasThemes::new(crate::Theme::Publication, &plots);
        let header = CanvasHeader {
            start: 0,
            span: 2,
            text: "MMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMM".into(),
        };
        let appearance = themes.root().legacy_text_appearance(
            themes.root().composition_title_font_size,
            crate::TextWeight::Normal,
        );
        let (hard, _) = crate::layout::fit::intrinsic_widths(
            themes.root(),
            &header.text,
            &appearance,
            "header span fixture",
        )
        .unwrap();
        let FitOutcome::Measured(cells) = allocate(
            PanelBox::new(0.0, 700.0, 0.0, 320.0),
            (1, 3),
            &plots,
            &[None; 3],
            &[None; 3],
            &[header],
            &[],
            &themes,
            &FitScope::root(),
        )
        .unwrap() else {
            panic!("spanning header and trailing empty cell must fit");
        };
        assert_eq!(cells.len(), 3);
        assert!(cells[1].x1 - cells[0].x0 >= hard);
        assert!(cells[2].width() > 0.0);
        assert!((cells[2].x0 - cells[1].x1 - 12.0).abs() < 1.0 / 1024.0);
    }
}
