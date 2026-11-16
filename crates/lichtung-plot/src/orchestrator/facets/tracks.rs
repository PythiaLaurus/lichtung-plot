//! Facet Grid assignments retain sparse coordinates and separate measured
//! decoration gutters from the aligned data-panel tracks.

use crate::layout::canvas::EdgeSpaceRequirements;
use crate::layout::fit::{FitOutcome, FitScope};
use crate::layout::kernel::{
    allocate_grid_axis, nearest_spatial_quanta, FitKind, GridRequirement, TrackSpec,
};
use crate::layout::{AddressRole, ComponentAddress, PixelRect};
use crate::orchestrator::registration::RowSelection;
use crate::orchestrator::root::{GridSlot, PreparedFacetTopology, Root};
use crate::Error;
use crate::FacetCell;

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

pub(super) struct ContentSizing<'a, 'src> {
    pub root: &'a Root<'src>,
    pub topology: &'a PreparedFacetTopology,
    pub windows: [Option<(f64, f64)>; 2],
    pub round: &'a crate::layout::kernel::FeedbackRound,
    pub natural_width: bool,
    pub natural_height: bool,
}

pub(super) struct GridAssignment {
    pub cells: Vec<PixelRect>,
    pub size: (f64, f64),
}

#[expect(
    clippy::too_many_arguments,
    reason = "facet ownership and prepared summaries are shared by both measurement phases"
)]
pub(super) fn allocate(
    outer: PixelRect,
    shape: (usize, usize),
    panels: &[(GridSlot, FacetCell, RowSelection)],
    minima: &[(f64, f64)],
    gutters: &[(usize, usize, EdgeSpaceRequirements)],
    theme: &crate::text::TextRequestContext<'_, '_>,
    scope: &FitScope,
    content: Option<&ContentSizing<'_, '_>>,
) -> Result<FitOutcome<GridAssignment>, Error> {
    Ok(
        match measure(
            outer,
            shape,
            panels,
            minima,
            gutters,
            theme,
            scope,
            content,
            Phase::Complete,
        )? {
            FitOutcome::Measured(Measurement::Complete(grid)) => FitOutcome::Measured(grid),
            FitOutcome::Miss(failure) => FitOutcome::Miss(failure),
            FitOutcome::Measured(Measurement::Columns(_)) => {
                unreachable!("complete facet Grid phase")
            }
        },
    )
}

pub(super) fn natural_width(
    shape: (usize, usize),
    panels: &[(GridSlot, FacetCell, RowSelection)],
    minima: &[(f64, f64)],
    theme: &crate::text::TextRequestContext<'_, '_>,
    scope: &FitScope,
    content: &ContentSizing<'_, '_>,
) -> Result<FitOutcome<f64>, Error> {
    if !content.natural_width {
        return Err(Error::Render(
            "natural facet columns require a natural-width assignment".into(),
        ));
    }
    Ok(
        match measure(
            PixelRect::new(0.0, 0.0, 0.0, 0.0),
            shape,
            panels,
            minima,
            &[],
            theme,
            scope,
            Some(content),
            Phase::Columns,
        )? {
            FitOutcome::Measured(Measurement::Columns(width)) => FitOutcome::Measured(width),
            FitOutcome::Miss(failure) => FitOutcome::Miss(failure),
            FitOutcome::Measured(Measurement::Complete(_)) => {
                unreachable!("facet column preference phase")
            }
        },
    )
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Columns,
    Complete,
}
enum Measurement {
    Columns(f64),
    Complete(GridAssignment),
}

#[expect(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    reason = "one measured facet Grid retains row-major sparse cells, gutters, and both axis vectors"
)]
fn measure(
    outer: PixelRect,
    shape: (usize, usize),
    panels: &[(GridSlot, FacetCell, RowSelection)],
    minima: &[(f64, f64)],
    gutters: &[(usize, usize, EdgeSpaceRequirements)],
    theme: &crate::text::TextRequestContext<'_, '_>,
    scope: &FitScope,
    content: Option<&ContentSizing<'_, '_>>,
    phase: Phase,
) -> Result<FitOutcome<Measurement>, Error> {
    let (rows, columns) = shape;
    if rows == 0 || columns == 0 || panels.len() != minima.len() {
        return Err(Error::Render(
            "facet Grid has invalid shape or intrinsic ownership".into(),
        ));
    }
    let count = rows
        .checked_mul(columns)
        .ok_or_else(|| Error::Render("facet Grid topology overflowed".into()))?;
    theme.text.prepared_font_environment()?;
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
        for (slot, _, _) in panels {
            let id = content.topology.slots[slot.0]
                .plot_id
                .as_ref()
                .expect("facet producer");
            let registered = content.root.plot(id)?;
            let (left, right) = crate::layout::interior::sizing::horizontal_gutter_preference(
                registered.source,
                Some(registered),
                &content.root.operation,
                theme,
            )?;
            let insets = column_insets.get_mut(slot.0 % columns).ok_or_else(|| {
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
    for (slot, _, _) in panels {
        if slot.0 / columns >= rows {
            return Err(Error::Render(
                "facet child is outside declared Grid topology".into(),
            ));
        }
        owners.push(
            grid.child(
                AddressRole::FacetCell,
                u32::try_from(slot.0)
                    .map_err(|_| Error::Render("facet cell address overflowed".into()))?,
            )?,
        );
    }
    let mut widths = Vec::new();
    let mut heights = Vec::new();
    crate::data::source::reserve_source(&mut widths, panels.len())?;
    crate::data::source::reserve_source(&mut heights, panels.len())?;
    for (((slot, _, _), minimum), address) in panels.iter().zip(minima).zip(&owners) {
        let preferred = if let Some(content) = content {
            let id = content.topology.slots[slot.0]
                .plot_id
                .as_ref()
                .expect("facet producer");
            let registered = content.root.plot(id)?;
            let preference = crate::layout::interior::sizing::body_preference(
                registered.source,
                Some(registered),
                content.root.plot_intrinsic(id)?,
                &content.root.operation,
                theme,
                *minimum,
            )?;
            let guided = if matches!(
                registered.source.kind(),
                crate::PlotKind::Heatmap | crate::PlotKind::Beeswarm
            ) {
                crate::layout::interior::sizing::guided_preference(
                    registered.source,
                    content.root.plots.first(),
                )?
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
            preferred.max(crate::theme::composition::hard(
                &strip_lengths.horizontal_inset.add_pixels(
                    super::rendering::strip_widths(
                        registered.source,
                        registered
                            .scope
                            .numeric_owner
                            .facet
                            .as_ref()
                            .expect("prepared facet occurrence"),
                        theme,
                    )?
                    .1,
                )?,
            )?)
        } else {
            minimum.0
        };
        widths.push(GridRequirement {
            start: slot.0 % columns,
            span: 1,
            hard: required(minimum.0)?,
            preferred: required(preferred.max(minimum.0))?,
            fill: true,
            address,
        });
        heights.push(GridRequirement {
            start: slot.0 / columns,
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
        || (phase == Phase::Complete
            && height <= 0
            && !content.is_some_and(|value| value.natural_height))
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
    let grid_width = checked(
        columns.iter().map(|&value| i128::from(value)).sum::<i128>()
            + horizontal_reserved
            + i128::from(gap)
                * i128::try_from(columns.len().saturating_sub(1))
                    .map_err(|_| Error::Render("facet gap count overflowed".into()))?,
    )?;
    if phase == Phase::Columns {
        return Ok(FitOutcome::Measured(Measurement::Columns(pixels(
            grid_width,
        ))));
    }
    if let Some(content) = content {
        for (index, (((slot, _, _), child), minimum)) in
            panels.iter().zip(&mut heights).zip(minima).enumerate()
        {
            let id = content.topology.slots[slot.0]
                .plot_id
                .as_ref()
                .expect("facet producer");
            let registered = content.root.plot(id)?;
            let address = ComponentAddress::root().child(
                AddressRole::FacetCell,
                u32::try_from(index)
                    .map_err(|_| Error::Render("facet content domain address overflowed".into()))?,
            )?;
            let mut windows = content.windows;
            if let Some(domain) = content.round.feedback_domain(
                &address,
                crate::layout::kernel::FeedbackKind::AutomaticDomainX,
            ) {
                windows[0] = Some(
                    windows[0].map_or(domain, |base| (base.0.min(domain.0), base.1.max(domain.1))),
                );
            }
            if let Some(domain) = content.round.feedback_domain(
                &address,
                crate::layout::kernel::FeedbackKind::AutomaticDomainY,
            ) {
                windows[1] = Some(
                    windows[1].map_or(domain, |base| (base.0.min(domain.0), base.1.max(domain.1))),
                );
            }
            let preference = crate::layout::interior::sizing::body_preference(
                registered.source,
                Some(registered),
                content.root.plot_intrinsic(id)?,
                &content.root.operation,
                theme,
                *minimum,
            )?;
            let width = pixels(columns[slot.0 % columns.len()]);
            let guide = if matches!(
                registered.source.kind(),
                crate::PlotKind::Heatmap | crate::PlotKind::Beeswarm
            ) {
                crate::layout::interior::sizing::guided_preference(
                    registered.source,
                    content.root.plots.first(),
                )?
            } else {
                None
            };
            let mut height = if guide.is_some() {
                crate::layout::interior::sizing::guided_body_height(
                    registered.source,
                    Some(registered),
                    content.root.plot_intrinsic(id)?,
                    &content.root.operation,
                    theme,
                    width,
                    guide.as_ref(),
                    scope,
                    Some(std::array::from_fn(|side| {
                        windows[side].unwrap_or(registered.axes.ranges()[side])
                    })),
                )?
            } else {
                registered.source.natural_body_height(
                    Some(registered),
                    content.root.plot_intrinsic(id)?,
                    &content.root.operation,
                    theme,
                    width,
                    preference.1,
                    Some(std::array::from_fn(|side| {
                        windows[side].unwrap_or(registered.axes.ranges()[side])
                    })),
                )?
            };
            if row_insets[slot.0 / columns.len()][1] == 0 {
                let strips = super::rendering::strip_plans(
                    registered.source,
                    registered
                        .scope
                        .numeric_owner
                        .facet
                        .as_ref()
                        .expect("prepared facet occurrence"),
                    theme,
                    width,
                    scope,
                )
                .map_err(crate::layout::fit::PlanError::into_error)?;
                height = crate::theme::composition::hard(
                    &strip_lengths
                        .content_clearance
                        .add_pixels(
                            strips
                                .iter()
                                .map(crate::layout::ResolvedTextPlan::height)
                                .sum(),
                        )?
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
            cells.push(PixelRect::new(
                pixels(left),
                pixels(checked(i128::from(left) + i128::from(width))?),
                pixels(checked(i128::from(top) - i128::from(rows[row]))?),
                pixels(top),
            ));
        }
    }
    Ok(FitOutcome::Measured(Measurement::Complete(
        GridAssignment {
            cells,
            size: (pixels(grid_width), pixels(grid_height)),
        },
    )))
}
