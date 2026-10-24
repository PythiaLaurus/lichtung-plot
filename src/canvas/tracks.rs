//! Canvas Grid tracks use the same quantized allocator as facet and TextPanel
//! owners while keeping each Plot's theme and declared sparse coordinates.

use super::{
    text_panel::TextPanelIntrinsics, CanvasHeader, CanvasHeaderPlan, ResolvedCanvasThemes,
};
use crate::decorations::legend::discrete::scoped::GuideAllocation;
use crate::layout::canvas::PanelRequirements;
use crate::layout::fit::{self, FitOutcome, FitScope};
use crate::layout::kernel::{
    allocate_grid_axis, nearest_spatial_quanta, FitKind, GridRequirement, TrackSpec,
};
use crate::layout::PhysicalTextAlignment;
use crate::layout::{AddressRole, ComponentAddress, PanelBox};
use crate::render::backend::{PlacedText, Point2D, TextAnchor};
use crate::theme::ResolvedTheme;
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

fn centered_x(
    left: f64,
    right: f64,
    plan: &crate::layout::ResolvedTextPlan,
) -> Result<(f64, TextAnchor), Error> {
    // Freeze the whole block origin with its parent's parity, then use the
    // renderer's canonical text half exactly once. Rounding the span midpoint
    // and the odd-width text half independently can leak one quantum of ink.
    let origin = position(left + (right - left - plan.width()) / 2.0)?;
    let (offset, anchor) = match plan.style.alignment {
        PhysicalTextAlignment::Start => (0, TextAnchor::Start),
        PhysicalTextAlignment::Center => (
            crate::render::backend::nearest_even_text_half(plan.width_quanta)
                .ok_or_else(|| Error::Render("Canvas header text anchor overflowed".into()))?,
            TextAnchor::Middle,
        ),
        PhysicalTextAlignment::End => (plan.width_quanta, TextAnchor::End),
    };
    Ok((origin + pixels(offset), anchor))
}

fn place_header(
    plan: crate::layout::ResolvedTextPlan,
    region: PanelBox,
    position: Point2D,
    anchor: TextAnchor,
    scope: &FitScope,
) -> Result<FitOutcome<PlacedText>, Error> {
    let placement = crate::layout::text::placement::resolve(
        crate::render::backend::PlannedTextPlacement::middle(position, anchor, 0.0),
        &plan,
        Some(region),
    )?;
    Ok(
        match fit::outcome(crate::layout::text::placement::contain(
            plan, placement, region, scope,
        ))? {
            FitOutcome::Measured(plan) => FitOutcome::Measured(PlacedText { plan, placement }),
            FitOutcome::Miss(reason) => FitOutcome::Miss(reason),
        },
    )
}

fn required(value: f64) -> Result<i64, Error> {
    nearest_spatial_quanta(crate::layout::kernel::required_length(value)?)
}

fn centered_y(region: PanelBox, plan: &crate::layout::ResolvedTextPlan) -> Result<f64, Error> {
    let bottom = position(region.y0 + (region.height() - plan.height()) / 2.0)?;
    let half = crate::render::backend::nearest_even_text_half(plan.height_quanta)
        .ok_or_else(|| Error::Render("Canvas header half height overflow".into()))?;
    Ok(bottom + pixels(plan.height_quanta - half))
}

fn checked(value: i128) -> Result<i64, Error> {
    i64::try_from(value).map_err(|_| Error::Render("Canvas track arithmetic overflowed".into()))
}

fn interior_insets(insets: &[[i64; 2]], start: usize, span: usize, rows: bool) -> i128 {
    let trailing = usize::from(!rows);
    (start..start + span - 1)
        .map(|index| {
            i128::from(insets[index][trailing]) + i128::from(insets[index + 1][1 - trailing])
        })
        .sum()
}

fn plot_minimum(
    plot: &Plot,
    title: crate::decorations::title_block::plan::RootTitleBlockIntrinsics,
    theme: &ResolvedTheme,
) -> Result<(f64, f64), Error> {
    let mut hard = plot.intrinsic_body_minimum(theme)?;
    let sources = plot.prose();
    hard.1 = crate::decorations::title_block::plan::RootTitleBlockPlan::minimum_height(
        theme, sources, hard.1,
    )?;
    let title = title.widths().0;
    hard.0 = hard.0.max(if sources.iter().any(Option::is_some) {
        theme.root_outer_width(title)?
    } else {
        title
    });
    Ok(hard)
}

pub(super) fn minimum(
    placements: &super::placement::Placements,
    plots: &[Option<Plot>],
    text: &[Option<TextPanelIntrinsics>],
    titles: &[Option<crate::decorations::title_block::plan::RootTitleBlockIntrinsics>],
    themes: &ResolvedCanvasThemes,
    scope: &FitScope,
) -> Result<FitOutcome<(f64, f64)>, Error> {
    let (rows, columns) = placements.shape();
    let owners = crate::data::source::collect_source((0..placements.len()).map(|index| {
        ComponentAddress::root().child(
            AddressRole::CanvasCell,
            u32::try_from(index)
                .map_err(|_| Error::Render("Canvas minimum owner overflow".into()))?,
        )
    }))?;
    let mut requirements = [Vec::new(), Vec::new()];
    for (index, address) in owners.iter().enumerate() {
        let Some(footprint) = placements.footprint(index) else {
            continue;
        };
        let hard = if let Some(plot) = &plots[index] {
            plot_minimum(
                plot,
                titles[index].expect("Plot title intrinsics"),
                themes.plot(index).expect("Plot theme"),
            )?
        } else if let Some(text) = text[index] {
            text.minimum
        } else {
            continue;
        };
        for (axis, (start, span, hard)) in [
            (footprint.column, footprint.column_span, hard.0),
            (footprint.row, footprint.row_span, hard.1),
        ]
        .into_iter()
        .enumerate()
        {
            fit::reserve(&mut requirements[axis], 1, "Canvas spanning minimum")?;
            requirements[axis].push(GridRequirement {
                start,
                span,
                hard: required(hard)?,
                preferred: required(hard)?,
                fill: true,
                address,
            });
        }
    }
    let mut result = [0.0; 2];
    for (axis, count) in [columns, rows].into_iter().enumerate() {
        let base = crate::data::source::collect_source((0..count).map(|_| {
            Ok(TrackSpec {
                min: 0,
                preferred: 0,
                max: None,
                flexible: false,
            })
        }))?;
        match allocate_grid_axis(
            &base,
            &requirements[axis],
            None,
            [0, 0],
            required(themes.root().composition_grid_gap()?)?,
            None,
            true,
            scope,
        )? {
            FitOutcome::Measured(assignment) => {
                let sum = assignment
                    .sizes
                    .iter()
                    .map(|&value| i128::from(value))
                    .sum::<i128>()
                    + i128::from(required(themes.root().composition_grid_gap()?)?)
                        * i128::try_from(count.saturating_sub(1))
                            .map_err(|_| Error::Render("Canvas minimum gap overflow".into()))?;
                result[axis] = pixels(checked(sum)?);
            }
            FitOutcome::Miss(failure) => return Ok(FitOutcome::Miss(failure)),
        }
    }
    Ok(FitOutcome::Measured((result[0], result[1])))
}

pub(super) struct CanvasGridAssignment {
    pub owners: Vec<Option<PanelBox>>,
    pub tracks: DeclaredTracks,
    pub headers: Vec<CanvasHeaderPlan>,
    pub row_titles: Vec<CanvasHeaderPlan>,
    pub size: (f64, f64),
    pub content_insets: Vec<[i64; 4]>,
}

pub(super) struct DeclaredTracks {
    columns: Vec<(f64, f64)>,
    rows: Vec<(f64, f64)>,
}
impl DeclaredTracks {
    fn owner(&self, footprint: super::placement::Footprint) -> PanelBox {
        PanelBox::new(
            self.columns[footprint.column].0,
            self.columns[footprint.column + footprint.column_span - 1].1,
            self.rows[footprint.row + footprint.row_span - 1].0,
            self.rows[footprint.row].1,
        )
    }
    pub fn observe(&self, round: &mut crate::layout::kernel::FeedbackRound) -> Result<(), Error> {
        for edges in [&self.columns, &self.rows] {
            round.observe_count(edges.len())?;
            for &(lower, upper) in edges {
                round.observe_number(lower)?;
                round.observe_number(upper)?;
            }
        }
        Ok(())
    }
}

pub(super) struct ContentSizing<'a> {
    pub source: &'a super::Canvas,
    pub interiors: &'a [Option<crate::layout::interior::PreparedInteriorInputs>],
    pub round: &'a crate::layout::kernel::FeedbackRound,
    pub measurements: Option<&'a super::sizing::HeightMeasurements>,
    pub shared_allocation: Option<GuideAllocation>,
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
    placements: &super::placement::Placements,
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
    let (rows, columns) = placements.shape();
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

    let gap = required(theme.composition_grid_gap()?)?;
    let em = required(
        theme
            .typography
            .composition
            .composition_header
            .get_font_size(),
    )?;
    let header_role = theme.typography.get_composition().get_composition_header();
    let mut row_header_plans = Vec::new();
    crate::layout::fit::reserve(
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
        let appearance = header.text.appearance(header_role).require_single()?;
        let plan = theme.measure_appearance_text(
            header.text.as_str(),
            appearance,
            header
                .text
                .resolved_alignment(crate::TextAlignment::Center)
                .into(),
            "Canvas row header intrinsic",
        )?;
        row_header_width = row_header_width.max(crate::theme::composition::hard(
            &theme
                .header_lengths(appearance.theme.get_font_size())?
                .row_header_padding
                .add_pixels(plan.occupied_size().0)?,
        )?);
        row_header_plans.push(plan);
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
    crate::layout::fit::reserve(&mut column_insets, columns, "Canvas column gutters")?;
    crate::layout::fit::reserve(&mut row_insets, rows, "Canvas row gutters")?;
    column_insets.resize(columns, [0_i64; 2]);
    row_insets.resize(rows, [0_i64; 2]);
    if let Some(content) = content.filter(|value| value.natural_width) {
        // Natural selection uses the same external gutter columns as assigned
        // layout. Header/body requirements remain inner-track requirements.
        for (index, plot) in plots.iter().enumerate() {
            let Some(plot) = plot else { continue };
            let theme = themes.plot(index).expect("prepared Plot theme");
            let (left, right) =
                crate::layout::interior::sizing::horizontal_gutter_preference(plot, theme)?;
            let footprint = placements.footprint(index).expect("Plot owner");
            column_insets[footprint.column][0] =
                column_insets[footprint.column][0].max(required(left)?);
            let end = footprint.column + footprint.column_span - 1;
            column_insets[end][1] = column_insets[end][1].max(required(right)?);
        }
        for group in &content.source.x_range_groups {
            let aligned = group.iter().fold([0_i64; 2], |maximum, &index| {
                let footprint = placements.footprint(index).expect("range group owner");
                [
                    maximum[0].max(column_insets[footprint.column][0]),
                    maximum[1].max(column_insets[footprint.column + footprint.column_span - 1][1]),
                ]
            });
            for &index in group {
                let footprint = placements.footprint(index).expect("range group owner");
                column_insets[footprint.column][0] = aligned[0];
                column_insets[footprint.column + footprint.column_span - 1][1] = aligned[1];
            }
        }
    }
    for &(row, column, requirement) in gutters {
        if row >= rows || column >= columns {
            return Err(Error::Render(
                "Canvas gutter lies outside its declared Grid".into(),
            ));
        }
        let Some(footprint) = placements.footprint(row * columns + column) else {
            continue;
        };
        let end_column = column + footprint.column_span - 1;
        let end_row = row + footprint.row_span - 1;
        column_insets[column][0] = column_insets[column][0].max(required(requirement.left)?);
        column_insets[end_column][1] =
            column_insets[end_column][1].max(required(requirement.right)?);
        row_insets[end_row][0] = row_insets[end_row][0].max(required(requirement.bottom)?);
        row_insets[row][1] = row_insets[row][1].max(required(requirement.top)?);
    }
    let grid = ComponentAddress::root().child(AddressRole::CanvasGrid, 0)?;
    let mut owners = Vec::new();
    crate::layout::fit::reserve(&mut owners, count, "Canvas cell owners")?;
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
        .and_then(|count| count.checked_add(row_titles.len()))
        .ok_or_else(|| Error::Render("Canvas header requirement count overflowed".into()))?;
    crate::layout::fit::reserve(&mut widths, width_count, "Canvas column requirements")?;
    let height_count = count
        .checked_add(row_headers.len())
        .ok_or_else(|| Error::Render("Canvas row-header requirement count overflowed".into()))?;
    crate::layout::fit::reserve(&mut heights, height_count, "Canvas row requirements")?;
    for index in 0..count {
        let minimum = if let Some(plot) = &plots[index] {
            let child_theme = themes
                .plot(index)
                .expect("configured Plot retains its theme");
            let hard = plot_minimum(
                plot,
                plot_titles[index].expect("Plot text intrinsics"),
                child_theme,
            )?;
            let title = plot_titles[index]
                .expect("configured Plot retains prepared text intrinsics")
                .widths();
            let sources = plot.prose();
            let present = sources.map(|value| value.is_some());
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
                for (source, appearance) in sources.into_iter().zip(
                    crate::decorations::title_block::plan::RootTitleBlockPlan::appearances(
                        child_theme,
                        sources,
                    ),
                ) {
                    if let Some(source) = source {
                        width = width.max(child_theme.root_outer_width(
                            crate::layout::interior::sizing::text_width(
                                child_theme,
                                source.as_str(),
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
        let footprint = placements
            .footprint(index)
            .ok_or_else(|| Error::Render("Canvas intrinsic has no canonical owner".into()))?;
        let horizontal = interior_insets(
            &column_insets,
            footprint.column,
            footprint.column_span,
            false,
        );
        let vertical = interior_insets(&row_insets, footprint.row, footprint.row_span, true);
        widths.push(GridRequirement {
            start: footprint.column,
            span: footprint.column_span,
            hard: checked((i128::from(required(hard.0)?) - horizontal).max(0))?,
            preferred: checked(
                (i128::from(required(preferred_width.max(hard.0))?) - horizontal).max(0),
            )?,
            fill: true,
            address: &owners[index],
        });
        heights.push(GridRequirement {
            start: footprint.row,
            span: footprint.row_span,
            hard: checked((i128::from(required(hard.1)?) - vertical).max(0))?,
            preferred: checked((i128::from(required(hard.1)?) - vertical).max(0))?,
            fill: true,
            address: &owners[index],
        });
    }
    let mut row_header_owners = Vec::new();
    crate::layout::fit::reserve(
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
        row_header_owners.push(ComponentAddress::root().child(AddressRole::CanvasHeader, ordinal)?);
    }
    for ((header, plan), owner) in row_headers
        .iter()
        .zip(&row_header_plans)
        .zip(&row_header_owners)
    {
        let height = required(plan.occupied_size().1)?;
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
            )?,
        );
    }
    for (header, owner) in column_headers.iter().zip(&header_owners) {
        let (hard, preferred) = crate::layout::fit::intrinsic_widths(
            theme,
            header.text.as_str(),
            header.text.appearance(header_role),
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
        )?;
        row_title_owners.push(owner);
    }
    for (title, owner) in row_titles.iter().zip(&row_title_owners) {
        let appearance = title.text.appearance(header_role);
        let (hard, preferred) = fit::intrinsic_widths(
            theme,
            title.text.as_str(),
            appearance,
            "Canvas row title intrinsic",
        )?;
        let padding = theme
            .header_lengths(appearance.theme.get_font_size())?
            .row_title_horizontal_padding;
        widths.push(GridRequirement {
            start: 0,
            span: columns,
            hard: required(crate::theme::composition::hard(&padding.add_pixels(hard)?)?)?,
            preferred: required(crate::theme::composition::hard(
                &padding.add_pixels(preferred.max(hard))?,
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
    crate::layout::fit::reserve(&mut column_base, columns, "Canvas column tracks")?;
    crate::layout::fit::reserve(&mut row_base, rows, "Canvas row tracks")?;
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
        scope,
    )? {
        FitOutcome::Measured(value) => value.sizes,
        FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
    };
    let mut x_positions = Vec::new();
    crate::layout::fit::reserve(&mut x_positions, columns.len(), "Canvas x positions")?;
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
        let appearance = header.text.appearance(header_role);
        let plan = match fit::single_text(
            theme,
            header.text.as_str(),
            appearance,
            right - left,
            header
                .text
                .resolved_alignment(crate::TextAlignment::Center)
                .into(),
            "Canvas column header",
            &FitScope::new(&header_owners[index], 0),
        )? {
            FitOutcome::Measured(plan) => plan,
            FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
        };
        header_height = header_height.max(crate::theme::composition::hard(
            &theme
                .header_lengths(appearance.theme.get_font_size())?
                .column_header_padding
                .add_pixels(plan.occupied_size().1)?,
        )?);
        column_plans.push((plan, left, right));
    }
    let mut row_title_plans = Vec::new();
    crate::layout::fit::reserve(
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
    crate::layout::fit::reserve(&mut row_title_heights, rows, "Canvas row-title heights")?;
    row_title_heights.resize(rows, 0_i64);
    for (index, title) in row_titles.iter().enumerate() {
        let appearance = title.text.appearance(header_role);
        let header_lengths = theme.header_lengths(appearance.theme.get_font_size())?;
        let plan = match fit::single_text(
            theme,
            title.text.as_str(),
            appearance,
            crate::theme::composition::capacity(
                title_right - title_left,
                &header_lengths.row_title_horizontal_padding,
            )?,
            title
                .text
                .resolved_alignment(crate::TextAlignment::Center)
                .into(),
            "Canvas row title",
            &FitScope::new(&row_title_owners[index], 0),
        )? {
            FitOutcome::Measured(plan) => plan,
            FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
        };
        let reserve = required(crate::theme::composition::hard(
            &header_lengths
                .row_title_vertical_padding
                .add_pixels(plan.occupied_size().1)?,
        )?)?;
        row_title_heights[title.start] = reserve;
        row_title_plans.push(plan);
    }
    let title_gutters = row_title_heights
        .iter()
        .map(|&height| i128::from(height))
        .sum::<i128>();
    let mut axis_row_tops = Vec::new();
    crate::layout::fit::reserve(&mut axis_row_tops, rows, "Canvas row axis tops")?;
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

            let index = usize::try_from(child.address.leaf_slot_index())
                .map_err(|_| Error::Render("Canvas content slot index overflowed".into()))?;
            let footprint = placements.footprint(index).expect("height owner");
            let end_column = footprint.column + footprint.column_span - 1;
            let end_row = footprint.row + footprint.row_span - 1;
            let width = pixels(checked(
                i128::from(x_positions[end_column]) + i128::from(columns[end_column])
                    - i128::from(x_positions[footprint.column]),
            )?);
            let height = if let Some(plot) = &plots[index] {
                let dependencies = content.source.consumed_guide_dependencies(
                    index,
                    width,
                    content.shared_allocation,
                    content.round,
                )?;
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
                        column_insets[footprint.column][0],
                        column_insets[end_column][1],
                        row_insets[end_row][0],
                        row_insets[footprint.row][1],
                    ],
                    content.measurements,
                    &dependencies,
                )?
            } else if let Some(text) = content.source.placements.text(index) {
                super::text_panel::natural_height(text, width, theme)?
            } else {
                0.0
            };
            child.preferred = checked(
                (i128::from(required(height)?)
                    - interior_insets(&row_insets, footprint.row, footprint.row_span, true))
                .max(i128::from(child.hard)),
            )?;
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
    crate::layout::fit::reserve(&mut y_positions, rows.len(), "Canvas y positions")?;
    let mut cursor = i128::from(y1);
    for (index, &height) in rows.iter().enumerate() {
        cursor -= i128::from(row_insets[index][1]);
        y_positions.push(checked(cursor)?);
        cursor -= i128::from(height) + i128::from(row_insets[index][0]);
        if index + 1 < rows.len() {
            cursor -= i128::from(gap);
        }
    }
    let tracks = DeclaredTracks {
        columns: crate::data::source::collect_source(x_positions.iter().zip(&columns).map(
            |(&left, &width)| {
                Ok((
                    pixels(left),
                    pixels(checked(i128::from(left) + i128::from(width))?),
                ))
            },
        ))?,
        rows: crate::data::source::collect_source(y_positions.iter().zip(&rows).map(
            |(&top, &height)| {
                Ok((
                    pixels(checked(i128::from(top) - i128::from(height))?),
                    pixels(top),
                ))
            },
        ))?,
    };
    let mut assigned = Vec::new();
    let mut content_insets = Vec::new();
    if content.is_some() {
        crate::layout::fit::reserve(
            &mut content_insets,
            count,
            "Canvas content dependency insets",
        )?;
    }
    crate::layout::fit::reserve(&mut assigned, count, "Canvas assigned owners")?;

    for (row, _) in row_insets.iter().enumerate() {
        for (column, _) in column_insets.iter().enumerate() {
            let footprint = placements.footprint(row * columns.len() + column);
            assigned.push(footprint.map(|footprint| tracks.owner(footprint)));
            if content.is_some() {
                content_insets.push(footprint.map_or([0; 4], |footprint| {
                    [
                        column_insets[footprint.column][0],
                        column_insets[footprint.column + footprint.column_span - 1][1],
                        row_insets[footprint.row + footprint.row_span - 1][0],
                        row_insets[footprint.row][1],
                    ]
                }));
            }
        }
    }
    let mut headers = Vec::new();
    let header_count = column_headers
        .len()
        .checked_add(row_headers.len())
        .ok_or_else(|| Error::Render("Canvas header storage overflowed".into()))?;
    crate::render::backend::reserve_text_records(&mut headers, header_count)?;
    for (index, (header, (plan, left, right))) in
        column_headers.iter().zip(column_plans).enumerate()
    {
        let (x, anchor) = centered_x(left, right, &plan)?;
        let inset = theme
            .header_lengths(plan.style.font_size)?
            .column_header_padding
            .pixels_rounded(crate::layout::kernel::SpatialRounding::NearestEven)?
            / 2.0;
        let region = PanelBox::new(
            left,
            right,
            header_outer.y1 - pixels(header_quanta) + inset,
            header_outer.y1 - inset,
        );
        let y = centered_y(region, &plan)?;
        let text = match place_header(
            plan,
            region,
            Point2D::new(x, y),
            anchor,
            &FitScope::new(&header_owners[index], 0),
        )? {
            FitOutcome::Measured(text) => text,
            FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
        };
        headers.push(CanvasHeaderPlan {
            request: header.text.try_copy()?,
            text,
        });
    }
    for (index, (header, plan)) in row_headers.iter().zip(row_header_plans).enumerate() {
        let top = tracks.rows[header.start].1;
        let bottom = tracks.rows[header.start + header.span - 1].0;
        let (x, anchor) = centered_x(header_outer.x0, header_outer.x0 + row_header_width, &plan)?;
        let inset = theme
            .header_lengths(plan.style.font_size)?
            .row_header_padding
            .pixels_rounded(crate::layout::kernel::SpatialRounding::NearestEven)?
            / 2.0;
        let region = PanelBox::new(
            header_outer.x0 + inset,
            header_outer.x0 + row_header_width - inset,
            bottom,
            top,
        );
        let y = centered_y(region, &plan)?;
        let text = match place_header(
            plan,
            region,
            Point2D::new(x, y),
            anchor,
            &FitScope::new(&row_header_owners[index], 0),
        )? {
            FitOutcome::Measured(text) => text,
            FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
        };
        headers.push(CanvasHeaderPlan {
            request: header.text.try_copy()?,
            text,
        });
    }
    let mut measured_row_titles = Vec::new();
    crate::render::backend::reserve_text_records(&mut measured_row_titles, row_titles.len())?;
    for (index, (title, plan)) in row_titles.iter().zip(row_title_plans).enumerate() {
        let row = title.start;
        let (x, anchor) = centered_x(title_left, title_right, &plan)?;
        let spacing = theme.header_lengths(plan.style.font_size)?;
        let horizontal = spacing
            .row_title_horizontal_padding
            .pixels_rounded(crate::layout::kernel::SpatialRounding::NearestEven)?
            / 2.0;
        let vertical = spacing
            .row_title_vertical_padding
            .pixels_rounded(crate::layout::kernel::SpatialRounding::NearestEven)?
            / 2.0;
        let lower = tracks.rows[row].1 + pixels(axis_row_tops[row]);
        let region = PanelBox::new(
            title_left + horizontal,
            title_right - horizontal,
            lower + vertical,
            lower + pixels(row_title_heights[row]) - vertical,
        );
        let y = centered_y(region, &plan)?;
        let text = match place_header(
            plan,
            region,
            Point2D::new(x, y),
            anchor,
            &FitScope::new(&row_title_owners[index], 0),
        )? {
            FitOutcome::Measured(text) => text,
            FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
        };
        measured_row_titles.push(CanvasHeaderPlan {
            request: title.text.try_copy()?,
            text,
        });
    }
    Ok(FitOutcome::Measured(CanvasGridAssignment {
        owners: assigned,
        tracks,
        headers,
        row_titles: measured_row_titles,
        size: (grid_width, grid_height),
        content_insets,
    }))
}
