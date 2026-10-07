//! Canvas Grid tracks use the same quantized allocator as facet and TextPanel
//! owners while keeping each Plot's theme and declared sparse coordinates.

use super::{
    text_panel::TextPanelIntrinsics, CanvasHeader, CanvasHeaderPlan, ResolvedCanvasThemes,
};
use crate::layout::canvas::PanelRequirements;
use crate::layout::fit::{self, FitOutcome, FitScope};
use crate::layout::kernel::{
    allocate_grid_axis, nearest_spatial_quanta, FitKind, GridRequirement, TrackSpec,
};
use crate::layout::PhysicalTextAlignment;
use crate::layout::{AddressRole, ComponentAddress, PanelBox};
use crate::render::backend::{PlacedText, Point2D, TextAnchor};
use crate::{Error, Plot};

#[expect(
    clippy::cast_precision_loss,
    reason = "Canvas tracks are bounded by exact public u32 dimensions"
)]
fn pixels(quanta: i64) -> f64 {
    quanta as f64 / 1024.0
}

fn position(value: f64) -> Result<f64, Error> {
    Ok(pixels(nearest_spatial_quanta(value)?))
}

fn centered_x(left: f64, right: f64, plan: &crate::layout::ResolvedTextPlan) -> Result<f64, Error> {
    // Freeze the whole block origin with its parent's parity, then use the
    // renderer's canonical text half exactly once. Rounding the span midpoint
    // and the odd-width text half independently can leak one quantum of ink.
    let origin = position(left + (right - left - plan.width()) / 2.0)?;
    let half = crate::render::backend::nearest_even_text_half(plan.width_quanta)
        .ok_or_else(|| Error::Render("Canvas header text anchor overflowed".into()))?;
    Ok(origin + pixels(half))
}

fn required(value: f64) -> Result<i64, Error> {
    nearest_spatial_quanta(crate::layout::kernel::required_length(value)?)
}

fn checked(value: i128) -> Result<i64, Error> {
    i64::try_from(value).map_err(|_| Error::Render("Canvas track arithmetic overflowed".into()))
}

pub(super) struct CanvasGridAssignment {
    pub cells: Vec<PanelBox>,
    pub headers: Vec<CanvasHeaderPlan>,
    pub row_titles: Vec<CanvasHeaderPlan>,
    pub size: (f64, f64),
    pub content_insets: Vec<[i64; 4]>,
}

pub(super) struct ContentSizing<'a> {
    pub source: &'a super::Canvas,
    pub interiors: &'a [Option<crate::layout::interior::PreparedInteriorInputs>],
    pub round: &'a crate::layout::kernel::FeedbackRound,
    pub measurements: Option<&'a super::sizing::HeightMeasurements>,
    pub natural_width: bool,
    pub natural_height: bool,
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
    row_headers: &[CanvasHeader],
    row_titles: &[CanvasHeader],
    gutters: &[(usize, usize, PanelRequirements)],
    themes: &ResolvedCanvasThemes,
    scope: &FitScope,
    content: Option<&ContentSizing<'_>>,
) -> Result<FitOutcome<CanvasGridAssignment>, Error> {
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
    let gap = required(theme.composition_grid_gap()?)?;
    let em = required(f64::from(
        theme.typography.composition.composition_header.font_size(),
    ))?;
    let header_lengths = theme.header_lengths()?;
    let appearance = theme.typography.composition().composition_header();
    let mut row_header_plans = Vec::new();
    crate::layout::fit::reserve(
        theme,
        &mut row_header_plans,
        row_headers.len(),
        "Canvas row-header plans",
    )?;
    let mut row_header_width = 0.0_f64;
    for header in row_headers {
        if header.span == 0
            || header
                .start
                .checked_add(header.span)
                .is_none_or(|end| end > rows)
        {
            return Err(Error::Render(
                "Canvas row header span is outside declared Grid tracks".into(),
            ));
        }
        let plan = theme.measure_legacy_appearance_text(
            &header.text,
            appearance,
            PhysicalTextAlignment::Center,
            "Canvas row header intrinsic",
            false,
        )?;
        row_header_width = row_header_width.max(plan.width());
        row_header_plans.push(plan);
    }
    if !row_headers.is_empty() {
        row_header_width = crate::theme::composition::hard(
            &header_lengths
                .row_header_padding
                .add_pixels(row_header_width, &resources)?,
            &resources,
        )?;
    }
    let header_outer = outer;
    let outer = PanelBox::new(outer.x0 + row_header_width, outer.x1, outer.y0, outer.y1);
    if outer.width() <= 0.0 && !content.is_some_and(|value| value.natural_width) {
        return Ok(FitOutcome::Miss(scope.failure(
            FitKind::TrackOrResidual,
            "Canvas row headers leave no width for cell tracks",
        )));
    }
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
    let grid = ComponentAddress::root().child(
        AddressRole::CanvasGrid,
        0,
        &theme.prepared_font_environment()?.generation_resources(),
    )?;
    let mut owners = Vec::new();
    crate::layout::fit::reserve(theme, &mut owners, count, "Canvas cell owners")?;
    for index in 0..count {
        owners.push(
            grid.child(
                AddressRole::CanvasCell,
                u32::try_from(index)
                    .map_err(|_| Error::Render("Canvas cell address overflowed".into()))?,
                &theme.prepared_font_environment()?.generation_resources(),
            )?,
        );
    }
    let mut widths = Vec::new();
    let mut heights = Vec::new();
    let width_count = count
        .checked_add(column_headers.len())
        .and_then(|count| count.checked_add(row_titles.len()))
        .ok_or_else(|| Error::Render("Canvas header requirement count overflowed".into()))?;
    crate::layout::fit::reserve(
        theme,
        &mut widths,
        width_count,
        "Canvas column requirements",
    )?;
    let height_count = count
        .checked_add(row_headers.len())
        .ok_or_else(|| Error::Render("Canvas row-header requirement count overflowed".into()))?;
    crate::layout::fit::reserve(theme, &mut heights, height_count, "Canvas row requirements")?;
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
            hard.1 = child_theme.embedded_minimum_height(present, hard.1)?;
            hard.0 = hard.0.max(if present.into_iter().any(|value| value) {
                child_theme.root_outer_width(title.0)?
            } else {
                title.0
            });
            let preferred = if content.is_some() {
                let body = crate::layout::interior::sizing::body_preference(
                    plot,
                    child_theme,
                    plot.intrinsic_body_minimum(child_theme)?,
                )?;
                let mut width = body.0;
                if let Some(content) = content.filter(|content| {
                    content.source.legend_policy == crate::CanvasLegendPolicy::PerPlot
                }) {
                    if let Some(guide) = crate::layout::interior::sizing::guided_preference(
                        plot,
                        content.interiors[index]
                            .as_ref()
                            .expect("prepared Canvas guide scope"),
                        child_theme,
                    )? {
                        let size = guide.preference(child_theme)?;
                        width = if guide.horizontal() {
                            width.max(size.0)
                        } else {
                            width + size.0 + child_theme.legend_outside_gap()?
                        };
                    }
                }
                for (source, appearance) in
                    [plot.title_text(), plot.subtitle_text(), plot.caption_text()]
                        .into_iter()
                        .zip([
                            &child_theme.typography.root.title,
                            &child_theme.typography.root.subtitle,
                            &child_theme.typography.root.caption,
                        ])
                {
                    if let Some(source) = source {
                        width = width.max(child_theme.root_outer_width(
                            crate::layout::interior::sizing::text_width(
                                child_theme,
                                source,
                                appearance,
                            )?,
                        )?);
                    }
                }
                width
            } else if present.into_iter().any(|value| value) {
                child_theme.root_outer_width(title.1)?
            } else {
                title.1
            };
            Some((hard, hard.0.max(preferred)))
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
    let mut row_header_owners = Vec::new();
    crate::layout::fit::reserve(
        theme,
        &mut row_header_owners,
        row_headers.len(),
        "Canvas row-header owners",
    )?;
    for index in 0..row_headers.len() {
        let ordinal = column_headers
            .len()
            .checked_add(index)
            .and_then(|index| u32::try_from(index).ok())
            .ok_or_else(|| Error::Render("Canvas row-header address overflowed".into()))?;
        row_header_owners.push(ComponentAddress::root().child(
            AddressRole::CanvasHeader,
            ordinal,
            &theme.prepared_font_environment()?.generation_resources(),
        )?);
    }
    for ((header, plan), owner) in row_headers
        .iter()
        .zip(&row_header_plans)
        .zip(&row_header_owners)
    {
        let height = required(plan.height())?;
        heights.push(GridRequirement {
            start: header.start,
            span: header.span,
            hard: height,
            preferred: height,
            fill: false,
            address: owner,
        });
    }
    let mut header_owners = Vec::new();
    crate::layout::fit::reserve(
        theme,
        &mut header_owners,
        column_headers.len(),
        "Canvas header owners",
    )?;
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
                &theme.prepared_font_environment()?.generation_resources(),
            )?,
        );
    }
    for (header, owner) in column_headers.iter().zip(&header_owners) {
        let (hard, preferred) = crate::layout::fit::intrinsic_widths(
            theme,
            &header.text,
            appearance,
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
    let mut row_title_owners = Vec::new();
    crate::layout::fit::reserve(
        theme,
        &mut row_title_owners,
        row_titles.len(),
        "Canvas row-title owners",
    )?;
    for (index, title) in row_titles.iter().enumerate() {
        if title.span != 1 || title.start >= rows {
            return Err(Error::Render(
                "Canvas row title lies outside declared Grid tracks".into(),
            ));
        }
        let owner = ComponentAddress::root().child(
            AddressRole::CanvasRowTitle,
            u32::try_from(index)
                .map_err(|_| Error::Render("Canvas row-title address overflowed".into()))?,
            &theme.prepared_font_environment()?.generation_resources(),
        )?;
        row_title_owners.push(owner);
    }
    for (title, owner) in row_titles.iter().zip(&row_title_owners) {
        let (hard, preferred) =
            fit::intrinsic_widths(theme, &title.text, appearance, "Canvas row title intrinsic")?;
        let padding = &header_lengths.row_title_horizontal_padding;
        widths.push(GridRequirement {
            start: 0,
            span: columns,
            hard: required(crate::theme::composition::hard(
                &padding.add_pixels(hard, &resources)?,
                &resources,
            )?)?,
            preferred: required(crate::theme::composition::hard(
                &padding.add_pixels(preferred.max(hard), &resources)?,
                &resources,
            )?)?,
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
    if (width <= 0 && !content.is_some_and(|value| value.natural_width))
        || (height <= 0 && !content.is_some_and(|value| value.natural_height))
    {
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
        if content.is_some_and(|value| value.natural_width) {
            None
        } else {
            Some(checked(width)?)
        },
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
    crate::layout::fit::reserve(theme, &mut x_positions, columns.len(), "Canvas x positions")?;
    let mut cursor = i128::from(x0);
    for (index, &width) in columns.iter().enumerate() {
        cursor += i128::from(column_insets[index][0]);
        x_positions.push(checked(cursor)?);
        cursor += i128::from(width) + i128::from(column_insets[index][1]);
        if index + 1 < columns.len() {
            cursor += i128::from(gap);
        }
    }
    let grid_width = pixels(checked(
        cursor - i128::from(nearest_spatial_quanta(header_outer.x0)?),
    )?);
    let mut column_plans = Vec::new();
    crate::layout::fit::reserve(
        theme,
        &mut column_plans,
        column_headers.len(),
        "Canvas measured column headers",
    )?;
    let mut header_height = 0.0_f64;
    for (index, header) in column_headers.iter().enumerate() {
        let end = header.start + header.span - 1;
        let left = pixels(x_positions[header.start]);
        let right = pixels(checked(
            i128::from(x_positions[end]) + i128::from(columns[end]),
        )?);
        let plan = match fit::text(
            theme,
            &header.text,
            appearance,
            right - left,
            PhysicalTextAlignment::Center,
            "Canvas column header",
            &FitScope::new(&header_owners[index], 0),
        )? {
            FitOutcome::Measured(plan) => plan,
            FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
        };
        header_height = header_height.max(crate::theme::composition::hard(
            &header_lengths
                .column_header_padding
                .add_pixels(plan.height(), &resources)?,
            &resources,
        )?);
        column_plans.push((plan, left, right));
    }
    let mut row_title_plans = Vec::new();
    crate::layout::fit::reserve(
        theme,
        &mut row_title_plans,
        row_titles.len(),
        "Canvas measured row titles",
    )?;
    let title_left = pixels(x_positions[0]);
    let last = columns.len() - 1;
    let title_right = pixels(checked(
        i128::from(x_positions[last]) + i128::from(columns[last]),
    )?);
    let mut row_title_heights = Vec::new();
    crate::layout::fit::reserve(
        theme,
        &mut row_title_heights,
        rows,
        "Canvas row-title heights",
    )?;
    row_title_heights.resize(rows, 0_i64);
    for (index, title) in row_titles.iter().enumerate() {
        let plan = match fit::text(
            theme,
            &title.text,
            appearance,
            crate::theme::composition::capacity(
                title_right - title_left,
                &header_lengths.row_title_horizontal_padding,
                &resources,
            )?,
            PhysicalTextAlignment::Center,
            "Canvas row title",
            &FitScope::new(&row_title_owners[index], 0),
        )? {
            FitOutcome::Measured(plan) => plan,
            FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
        };
        let reserve = required(crate::theme::composition::hard(
            &header_lengths
                .row_title_vertical_padding
                .add_pixels(plan.height(), &resources)?,
            &resources,
        )?)?;
        row_title_heights[title.start] = reserve;
        row_title_plans.push(plan);
    }
    let title_gutters = row_title_heights
        .iter()
        .map(|&height| i128::from(height))
        .sum::<i128>();
    let mut axis_row_tops = Vec::new();
    crate::layout::fit::reserve(theme, &mut axis_row_tops, rows, "Canvas row axis tops")?;
    axis_row_tops.extend(row_insets.iter().map(|insets| insets[1]));
    for (insets, &title_height) in row_insets.iter_mut().zip(&row_title_heights) {
        insets[1] = checked(i128::from(insets[1]) + i128::from(title_height))?;
    }
    let header_quanta = required(header_height)?;
    if let Some(content) = content {
        for child in &mut heights {
            if child.address.leaf_role() != AddressRole::CanvasCell {
                continue;
            }
            resources.claim_visits(1).map_err(|error| {
                Error::Render(format!("Canvas content owner lookup: {error:?}"))
            })?;
            let index = usize::try_from(child.address.leaf_slot_index())
                .map_err(|_| Error::Render("Canvas content slot index overflowed".into()))?;
            let width = pixels(columns[index % columns.len()]);
            let height = if let Some(plot) = &plots[index] {
                super::sizing::cell_height(
                    plot,
                    themes.plot(index).expect("prepared Plot theme"),
                    width,
                    content.interiors[index]
                        .as_ref()
                        .expect("prepared Canvas interior"),
                    content.source.legend_policy == crate::CanvasLegendPolicy::PerPlot,
                    content.round,
                    child.address,
                    [
                        column_insets[index % columns.len()][0],
                        column_insets[index % columns.len()][1],
                        row_insets[index / columns.len()][0],
                        row_insets[index / columns.len()][1],
                    ],
                    content.measurements,
                )?
            } else if let Some(text) = &content.source.text_panels[index] {
                super::text_panel::natural_height(text, width, theme)?
            } else {
                0.0
            };
            child.preferred = required(height)?.max(child.hard);
        }
    }
    let mut y1 = checked(i128::from(y1) - i128::from(header_quanta))?;
    let height = height - i128::from(header_quanta) - title_gutters;
    if height <= 0 && !content.is_some_and(|value| value.natural_height) {
        return Ok(FitOutcome::Miss(scope.failure(
            FitKind::TrackOrResidual,
            "Canvas measured headers and row titles leave no row tracks",
        )));
    }
    let rows = match allocate_grid_axis(
        &row_base,
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
        &resources,
        scope,
    )? {
        FitOutcome::Measured(value) => value.sizes,
        FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
    };
    let grid_height = pixels(checked(
        rows.iter().map(|&value| i128::from(value)).sum::<i128>()
            + i128::from(gap)
                * i128::try_from(rows.len().saturating_sub(1))
                    .map_err(|_| Error::Render("Canvas row gap overflowed".into()))?
            + vertical_gutters
            + title_gutters
            + i128::from(header_quanta),
    )?);
    if content.is_some_and(|value| value.natural_height) {
        y1 = checked(
            i128::from(nearest_spatial_quanta(header_outer.y0 + grid_height)?)
                - i128::from(header_quanta),
        )?;
    }
    let mut y_positions = Vec::new();
    crate::layout::fit::reserve(theme, &mut y_positions, rows.len(), "Canvas y positions")?;
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
    let mut content_insets = Vec::new();
    if content.is_some() {
        crate::layout::fit::reserve(
            theme,
            &mut content_insets,
            count,
            "Canvas content dependency insets",
        )?;
    }
    crate::layout::fit::reserve(theme, &mut cells, count, "Canvas assigned cells")?;
    resources
        .claim_visits(
            u128::try_from(count)
                .map_err(|_| Error::Render("Canvas cell visit count overflowed".into()))?,
        )
        .map_err(|error| Error::Render(format!("Canvas cell allocation admission: {error:?}")))?;
    for (row, &top) in y_positions.iter().enumerate() {
        for (column, (&left, &width)) in x_positions.iter().zip(&columns).enumerate() {
            cells.push(PanelBox::new(
                pixels(left),
                pixels(checked(i128::from(left) + i128::from(width))?),
                pixels(checked(i128::from(top) - i128::from(rows[row]))?),
                pixels(top),
            ));
            if content.is_some() {
                content_insets.push([
                    column_insets[column][0],
                    column_insets[column][1],
                    row_insets[row][0],
                    row_insets[row][1],
                ]);
            }
        }
    }
    let mut headers = Vec::new();
    let header_count = column_headers
        .len()
        .checked_add(row_headers.len())
        .ok_or_else(|| Error::Render("Canvas header storage overflowed".into()))?;
    crate::render::backend::reserve_text_records(
        &mut headers,
        header_count,
        theme.prepared_font_environment()?,
    )?;
    for (plan, left, right) in column_plans {
        let x = centered_x(left, right, &plan)?;
        headers.push(CanvasHeaderPlan {
            text: PlacedText::middle(
                plan,
                Point2D::new(x, position(header_outer.y1 - pixels(header_quanta) / 2.0)?),
                TextAnchor::Middle,
                0.0,
            ),
        });
    }
    for (header, plan) in row_headers.iter().zip(row_header_plans) {
        let top = cells[header.start * columns.len()].y1;
        let bottom = cells[(header.start + header.span - 1) * columns.len()].y0;
        let x = centered_x(header_outer.x0, header_outer.x0 + row_header_width, &plan)?;
        headers.push(CanvasHeaderPlan {
            text: PlacedText::middle(
                plan,
                Point2D::new(x, position(f64::midpoint(bottom, top))?),
                TextAnchor::Middle,
                0.0,
            ),
        });
    }
    let mut measured_row_titles = Vec::new();
    crate::render::backend::reserve_text_records(
        &mut measured_row_titles,
        row_titles.len(),
        theme.prepared_font_environment()?,
    )?;
    for (title, plan) in row_titles.iter().zip(row_title_plans) {
        let row = title.start;
        let x = centered_x(title_left, title_right, &plan)?;
        measured_row_titles.push(CanvasHeaderPlan {
            text: PlacedText::middle(
                plan,
                Point2D::new(
                    x,
                    position(
                        cells[row * columns.len()].y1
                            + pixels(axis_row_tops[row])
                            + pixels(row_title_heights[row]) / 2.0,
                    )?,
                ),
                TextAnchor::Middle,
                0.0,
            ),
        });
    }
    Ok(FitOutcome::Measured(CanvasGridAssignment {
        cells,
        headers,
        row_titles: measured_row_titles,
        size: (grid_width, grid_height),
        content_insets,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_pixel_headers_freeze_subpixel_centers_inside_their_actual_unpadded_bands() {
        let plots = [None];
        let mut themes = ResolvedCanvasThemes::new(
            crate::Theme::Publication,
            &plots,
            &crate::font::canonical::OperationGenerationResources::new(),
        )
        .unwrap();
        themes.root.typography.composition.composition_header = themes
            .root
            .typography
            .composition
            .composition_header
            .clone()
            .with_font_size(1)
            .unwrap();
        let zero = crate::theme::RelativeSpacing::ratio(0, 1).unwrap();
        let headers = crate::theme::composition::HeaderSpacing::new(
            zero.clone(),
            zero.clone(),
            zero.clone(),
            zero,
        )
        .unwrap();
        themes.root.composition = themes
            .root
            .composition
            .clone()
            .with_headers(headers)
            .unwrap();
        let header = CanvasHeader {
            start: 0,
            span: 1,
            text: "0".into(),
        };
        for parity in [0, 1] {
            let left = f64::from(parity) / 1024.0;
            let outer = PanelBox::new(left, 4.0 + left, 0.0, 4.0);
            let FitOutcome::Measured(grid) = allocate(
                outer,
                (1, 1),
                &plots,
                &[None],
                &[None],
                std::slice::from_ref(&header),
                std::slice::from_ref(&header),
                std::slice::from_ref(&header),
                &[],
                &themes,
                &FitScope::root(),
                None,
            )
            .unwrap() else {
                panic!("positive one-pixel-role header bands must fit without a padding floor");
            };
            let cell = grid.cells[0];
            let column = &grid.headers[0].text;
            let row = &grid.headers[1].text;
            let title = &grid.row_titles[0].text;
            assert!(row.plan.width() > 0.0 && row.plan.width() < 1.0);
            let bands = [
                (
                    column,
                    PanelBox::new(cell.x0, cell.x1, outer.y1 - column.plan.height(), outer.y1),
                ),
                (row, PanelBox::new(outer.x0, cell.x0, cell.y0, cell.y1)),
                (
                    title,
                    PanelBox::new(cell.x0, cell.x1, cell.y1, cell.y1 + title.plan.height()),
                ),
            ];
            for (text, band) in bands {
                assert_eq!(text.plan.source, "0");
                assert_eq!(text.plan.style.font_size, 1);
                let mut ink =
                    crate::render::geometry::GeometrySnapshot::new(themes.root()).unwrap();
                ink.text(text.placement, &text.plan, None).unwrap();
                assert!(!ink.bounds().is_empty());
                assert!(
                    ink.bounds().iter().all(|bounds| bounds.left >= band.x0
                        && bounds.right <= band.x1
                        && bounds.bottom >= band.y0
                        && bounds.top <= band.y1),
                    "complete header ink must stay in its actual band: band={band:?}, ink={:?}",
                    ink.bounds()
                );
            }
        }
    }

    #[test]
    fn unicode_spanning_headers_retain_complete_rtl_sources_and_unclipped_ink() {
        let plots = [None, None, None, None];
        let themes = ResolvedCanvasThemes::new(
            crate::Theme::Publication,
            &plots,
            &crate::font::canonical::OperationGenerationResources::new(),
        )
        .unwrap();
        let source = "مرحبا بالعالم\nעברית ושמות";
        let header = CanvasHeader {
            start: 0,
            span: 2,
            text: source.into(),
        };
        let title = CanvasHeader {
            start: 0,
            span: 1,
            text: source.into(),
        };
        let outer = PanelBox::new(1.0 / 1024.0, 601.0 / 1024.0 + 600.0, 0.0, 400.0);
        let FitOutcome::Measured(grid) = allocate(
            outer,
            (2, 2),
            &plots,
            &[None; 4],
            &[None; 4],
            std::slice::from_ref(&header),
            std::slice::from_ref(&header),
            std::slice::from_ref(&title),
            &[],
            &themes,
            &FitScope::root(),
            None,
        )
        .unwrap() else {
            panic!("complete RTL header spans must fit");
        };
        for text in grid
            .headers
            .iter()
            .map(|header| &header.text)
            .chain(grid.row_titles.iter().map(|header| &header.text))
        {
            assert_eq!(text.plan.source, source);
            assert_eq!(text.plan.lines.len(), 2);
            let mut ink = crate::render::geometry::GeometrySnapshot::new(themes.root()).unwrap();
            ink.text(text.placement, &text.plan, None).unwrap();
            assert!(!ink.bounds().is_empty());
            assert!(ink.bounds().iter().all(|bounds| bounds.left >= outer.x0
                && bounds.right <= outer.x1
                && bounds.bottom >= outer.y0
                && bounds.top <= outer.y1));
        }
    }

    #[test]
    fn declared_empty_tracks_use_header_em_independently_of_body_and_text_panel_title() {
        let plots = [None, None];
        let mut themes = ResolvedCanvasThemes::new(
            crate::Theme::Publication,
            &plots,
            &crate::font::canonical::OperationGenerationResources::new(),
        )
        .unwrap();
        themes.root.composition = themes
            .root
            .composition
            .clone()
            .with_canvas_grid_gap(crate::theme::RelativeSpacing::ratio(0, 1).unwrap())
            .unwrap();
        let source = crate::Canvas::grid(1, 2).unwrap();
        for (header, body, title) in [(20, 17, 20), (20, 34, 40), (31, 34, 40)] {
            themes.root.typography.composition.composition_header = themes
                .root
                .typography
                .composition
                .composition_header
                .clone()
                .with_font_size(header)
                .unwrap();
            themes.root.typography.composition.body = themes
                .root
                .typography
                .composition
                .body
                .clone()
                .with_font_size(body)
                .unwrap();
            themes.root.typography.composition.text_panel_title = themes
                .root
                .typography
                .composition
                .text_panel_title
                .clone()
                .with_font_size(title)
                .unwrap();
            crate::layout::plan_exact_root(
                1,
                1,
                |_| Ok(()),
                |(), assigned, round| {
                    let FitOutcome::Measured(grid) = allocate(
                        PanelBox::new(0.0, 0.0, 0.0, 0.0),
                        (1, 2),
                        &plots,
                        &[None; 2],
                        &[None; 2],
                        &[],
                        &[],
                        &[],
                        &[],
                        &themes,
                        &FitScope::root(),
                        Some(&ContentSizing {
                            source: &source,
                            interiors: &[None, None],
                            round,
                            measurements: None,
                            natural_width: true,
                            natural_height: true,
                        }),
                    )?
                    else {
                        panic!("declared tracks must have a natural preference");
                    };
                    assert_eq!(grid.size.0.to_bits(), (2.0 * f64::from(header)).to_bits());
                    assert_eq!(grid.size.1.to_bits(), f64::from(header).to_bits());
                    assert_eq!(grid.cells.len(), 2);
                    Ok(crate::layout::MeasuredLayout::new(assigned, ()))
                },
                |(), _, (), _| Ok(()),
            )
            .unwrap();
        }
    }

    #[test]
    fn sparse_mixed_grid_keeps_declared_gaps_and_trailing_cells() {
        let plots: Vec<Option<Plot>> = (0..4).map(|_| None).collect();
        let plot_titles = [None; 4];
        let themes = ResolvedCanvasThemes::new(
            crate::Theme::Publication,
            &plots,
            &crate::font::canonical::OperationGenerationResources::new(),
        )
        .unwrap();
        let text = super::super::TextPanel::new("A", "B").unwrap();
        let mut intrinsics = [None; 4];
        intrinsics[0] = Some(TextPanelIntrinsics::prepare(&text, themes.root()).unwrap());
        let FitOutcome::Measured(grid) = allocate(
            PanelBox::new(0.0, 400.0, 0.0, 300.0),
            (2, 2),
            &plots,
            &intrinsics,
            &plot_titles,
            &[],
            &[],
            &[],
            &[],
            &themes,
            &FitScope::root(),
            None,
        )
        .unwrap() else {
            panic!("declared trailing Canvas cells must fit");
        };
        let cells = grid.cells;
        assert_eq!(cells.len(), 4);
        assert!((cells[1].x0 - cells[0].x1 - 12.0).abs() < 1.0 / 1024.0);
        assert!((cells[0].y0 - cells[2].y1 - 12.0).abs() < 1.0 / 1024.0);
        assert!(cells[3].width() > 0.0 && cells[3].height() > 0.0);
    }

    #[test]
    fn text_panels_with_asymmetric_intrinsics_get_unequal_fill_columns() {
        let plots: Vec<Option<Plot>> = [None, None].into();
        let themes = ResolvedCanvasThemes::new(
            crate::Theme::Publication,
            &plots,
            &crate::font::canonical::OperationGenerationResources::new(),
        )
        .unwrap();
        let long =
            super::super::TextPanel::new("MMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMM", "Body").unwrap();
        let short = super::super::TextPanel::new("Brief", "Body").unwrap();
        let intrinsics = [
            Some(TextPanelIntrinsics::prepare(&long, themes.root()).unwrap()),
            Some(TextPanelIntrinsics::prepare(&short, themes.root()).unwrap()),
        ];
        let FitOutcome::Measured(grid) = allocate(
            PanelBox::new(0.0, 700.0, 0.0, 320.0),
            (1, 2),
            &plots,
            &intrinsics,
            &[None; 2],
            &[],
            &[],
            &[],
            &[],
            &themes,
            &FitScope::root(),
            None,
        )
        .unwrap() else {
            panic!("both assigned TextPanel cells must fit");
        };
        let cells = grid.cells;
        assert!(cells[0].width() > cells[1].width());
        assert!((cells[1].x0 - cells[0].x1 - 12.0).abs() < 1.0 / 1024.0);
        assert_eq!(cells[0].y0.to_bits(), cells[1].y0.to_bits());
    }

    #[test]
    fn spanning_column_header_preserves_title_width_without_repacking_sparse_cells() {
        let plots: Vec<Option<Plot>> = [None, None, None].into();
        let themes = ResolvedCanvasThemes::new(
            crate::Theme::Publication,
            &plots,
            &crate::font::canonical::OperationGenerationResources::new(),
        )
        .unwrap();
        let header = CanvasHeader {
            start: 0,
            span: 2,
            text: "MMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMM".into(),
        };
        let appearance = &themes.root().typography.composition.composition_header;
        let (hard, _) = crate::layout::fit::intrinsic_widths(
            themes.root(),
            &header.text,
            appearance,
            "header span fixture",
        )
        .unwrap();
        let FitOutcome::Measured(grid) = allocate(
            PanelBox::new(0.0, 700.0, 0.0, 320.0),
            (1, 3),
            &plots,
            &[None; 3],
            &[None; 3],
            &[header],
            &[],
            &[],
            &[],
            &themes,
            &FitScope::root(),
            None,
        )
        .unwrap() else {
            panic!("spanning header and trailing empty cell must fit");
        };
        let cells = grid.cells;
        assert_eq!(cells.len(), 3);
        assert!(cells[1].x1 - cells[0].x0 >= hard);
        assert!(cells[2].width() > 0.0);
        assert!((cells[2].x0 - cells[1].x1 - 12.0).abs() < 1.0 / 1024.0);
    }

    #[test]
    fn spanning_row_header_owns_its_left_track_and_vertical_span() {
        let plots: Vec<Option<Plot>> = [None, None].into();
        let themes = ResolvedCanvasThemes::new(
            crate::Theme::Publication,
            &plots,
            &crate::font::canonical::OperationGenerationResources::new(),
        )
        .unwrap();
        let row = CanvasHeader {
            start: 0,
            span: 2,
            text: "Paired rows".into(),
        };
        let FitOutcome::Measured(grid) = allocate(
            PanelBox::new(0.0, 300.0, 0.0, 120.0),
            (2, 1),
            &plots,
            &[None; 2],
            &[None; 2],
            &[],
            &[row],
            &[],
            &[],
            &themes,
            &FitScope::root(),
            None,
        )
        .unwrap() else {
            panic!("the measured row header must fit its two declared rows");
        };
        assert_eq!(grid.headers.len(), 1);
        assert_eq!(grid.headers[0].text.plan.source, "Paired rows");
        assert!(grid.cells[0].x0 > 0.0);
        assert_eq!(grid.cells[0].x0.to_bits(), grid.cells[1].x0.to_bits());
        assert!(grid.headers[0].text.placement.position.y > grid.cells[1].y0);
        assert!(grid.headers[0].text.placement.position.y < grid.cells[0].y1);
    }

    #[test]
    fn wrapped_row_title_owns_a_top_track_above_axis_gutters() {
        let plots: Vec<Option<Plot>> = [None, None].into();
        let themes = ResolvedCanvasThemes::new(
            crate::Theme::Publication,
            &plots,
            &crate::font::canonical::OperationGenerationResources::new(),
        )
        .unwrap();
        let title = CanvasHeader {
            start: 0,
            span: 1,
            text: "Independent longitudinal reference measurement period".into(),
        };
        let FitOutcome::Measured(grid) = allocate(
            PanelBox::new(0.0, 220.0, 0.0, 320.0),
            (2, 1),
            &plots,
            &[None; 2],
            &[None; 2],
            &[],
            &[],
            &[title],
            &[(
                0,
                0,
                PanelRequirements {
                    top: 24.0,
                    ..PanelRequirements::default()
                },
            )],
            &themes,
            &FitScope::root(),
            None,
        )
        .unwrap() else {
            panic!("the wrapped row title must fit its assigned track");
        };
        assert_eq!(grid.row_titles.len(), 1);
        let heading = &grid.row_titles[0].text;
        assert!(heading.plan.lines.len() > 1);
        assert!(heading.placement.position.y > grid.cells[0].y1 + 24.0);
        assert!(heading.placement.position.y < 320.0);
        assert!(grid.cells[1].height() > 0.0);
    }
}
