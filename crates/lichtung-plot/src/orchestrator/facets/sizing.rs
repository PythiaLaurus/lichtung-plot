//! Facets compose their owning Plot's request; cells have no standalone roots.

use super::{data, tracks};
use crate::builder::PlotDescription;
use crate::decorations::title_block::plan::{RootAlignments, RootTitleBlockPlan};
use crate::layout::canvas::EdgeSpaceRequirements;
use crate::layout::fit::{self, FitScope};
use crate::layout::interior::sizing::text_width;
use crate::layout::kernel::{root_dimension, FeedbackKind, FeedbackRound, RootSize};
use crate::layout::{AddressRole, ComponentAddress, PixelRect};
use crate::orchestrator::registration::RowSelection;
use crate::orchestrator::root::{GridSlot, PreparedFacetTopology, Root};
use crate::{Error, FacetCell};

#[expect(
    clippy::too_many_arguments,
    reason = "one owning facet Grid retains semantic inputs and consumed geometry"
)]
pub(super) fn grid_height(
    root: &Root<'_>,
    topology: &PreparedFacetTopology,
    panels: &[(GridSlot, FacetCell, RowSelection)],
    windows: [Option<(f64, f64)>; 2],
    shape: (usize, usize),
    minima: &[(f64, f64)],
    theme: &crate::text::TextRequestContext<'_, '_>,
    round: &FeedbackRound,
    width: f64,
) -> Result<fit::FitOutcome<f64>, Error> {
    theme.text.prepared_font_environment()?;
    let owner = ComponentAddress::root().child(AddressRole::FacetGrid, 0)?;
    let mut gutters = Vec::new();
    crate::data::source::reserve_source(&mut gutters, panels.len())?;
    for (index, (slot, _, _)) in panels.iter().enumerate() {
        let address = owner.child(
            AddressRole::FacetCell,
            u32::try_from(index)
                .map_err(|_| Error::Render("facet content address overflow".into()))?,
        )?;
        let [left, right, bottom, top] = round.insets(&address, FeedbackKind::AxisGutter);
        gutters.push((
            slot.0 / shape.1,
            slot.0 % shape.1,
            EdgeSpaceRequirements {
                left,
                right,
                bottom,
                top,
            },
        ));
    }
    Ok(
        match tracks::allocate(
            PixelRect::new(0.0, width, 0.0, 0.0),
            shape,
            panels,
            minima,
            &gutters,
            theme,
            &FitScope::root(),
            Some(&tracks::ContentSizing {
                root,
                topology,
                windows,
                round,
                natural_width: false,
                natural_height: true,
            }),
        )? {
            fit::FitOutcome::Measured(grid) => fit::FitOutcome::Measured(grid.size.1),
            fit::FitOutcome::Miss(reason) => fit::FitOutcome::Miss(reason),
        },
    )
}

#[expect(
    clippy::too_many_lines,
    reason = "one owning root composes its sparse facet grid, shared titles, guide and root text"
)]
pub(in crate::orchestrator) fn natural_size(
    source: &PlotDescription,
    root: &Root<'_>,
    topology: &PreparedFacetTopology,
    theme: &crate::text::TextRequestContext<'_, '_>,
    request: crate::layout::RootRequest,
    round: &FeedbackRound,
) -> Result<RootSize, Error> {
    theme.text.prepared_font_environment()?;

    let selectors = data::facet_selectors(source)?;
    let (_shell, panels) = data::extract_facet_rows(source, selectors)?;
    let labels = crate::orchestrator::registration::prepare_axis_labels(
        root.plot(&topology.shared_titles_from)?,
    )?;
    for registered in &root.plots {
        registered
            .axes
            .retain_axis_labels(labels.each_ref().map(Option::as_ref))?;
    }
    data::parent_orders(source, source.kind())?;
    let windows = [source.x_axis_spec(), source.y_axis_spec()]
        .map(|axis| axis.window.map(crate::layout::RawWindow::pair));
    let mut minima = Vec::new();
    crate::data::source::reserve_source(&mut minima, panels.len())?;
    let mut gutters = Vec::new();
    crate::data::source::reserve_source(&mut gutters, panels.len())?;
    let grid_owner = ComponentAddress::root().child(AddressRole::FacetGrid, 0)?;
    for (index, (slot, _, _)) in panels.iter().enumerate() {
        let id = topology.slots[slot.0]
            .plot_id
            .as_ref()
            .expect("facet occurrence");
        let registered = root.plot(id)?;
        minima.push(crate::layout::interior::intrinsic_body_minimum(
            registered,
            crate::builder::RootFrameDomainInput::Prepared,
            root.plot_intrinsic(id)?,
            &root.operation,
            theme,
        )?);
        let owner = grid_owner.child(
            AddressRole::FacetCell,
            u32::try_from(index)
                .map_err(|_| Error::Render("facet content address overflowed".into()))?,
        )?;
        let [left, right, bottom, top] = round.insets(&owner, FeedbackKind::AxisGutter);
        gutters.push((
            slot.0 / topology.columns,
            slot.0 % topology.columns,
            EdgeSpaceRequirements {
                left,
                right,
                bottom,
                top,
            },
        ));
    }
    let position = source.legend_position_spec();
    let outside =
        position == crate::LegendPosition::Auto || crate::decorations::plan::is_outside(position);
    let horizontal = matches!(
        position,
        crate::LegendPosition::OutsideTopLeft
            | crate::LegendPosition::OutsideTop
            | crate::LegendPosition::OutsideTopRight
            | crate::LegendPosition::OutsideBottomLeft
            | crate::LegendPosition::OutsideBottom
            | crate::LegendPosition::OutsideBottomRight
    );
    let guide_size = if let Some(guide) = topology.parent_guide.as_ref().filter(|_| outside) {
        crate::decorations::legend::Legend::natural_preference(&guide.descriptor, theme)?
    } else {
        (0.0, 0.0)
    };
    let guide_gap = if guide_size.0 > 0.0 {
        theme.legend_outside_gap()?
    } else {
        0.0
    };
    let allocate = |width: Option<f64>, gutters: &[(usize, usize, EdgeSpaceRequirements)]| {
        fit::measured(tracks::allocate(
            PixelRect::new(0.0, width.unwrap_or(0.0), 0.0, 0.0),
            (topology.rows, topology.columns),
            &panels,
            &minima,
            gutters,
            theme,
            &FitScope::root(),
            Some(&tracks::ContentSizing {
                root,
                topology,
                windows,
                round,
                natural_width: width.is_none(),
                natural_height: true,
            }),
        ))
        .map_err(fit::PlanError::into_error)
    };
    let requested_width = request.exact_width();
    let mut preferred = if requested_width.is_none() {
        // The vertical shared title genuinely consumes natural Grid height.
        // Without it, column selection has no child-height dependency.
        let natural = if labels[1].is_some() {
            allocate(None, &[])?.size
        } else {
            let width = fit::measured(tracks::natural_width(
                (topology.rows, topology.columns),
                &panels,
                &minima,
                theme,
                &FitScope::root(),
                &tracks::ContentSizing {
                    root,
                    topology,
                    windows,
                    round,
                    natural_width: true,
                    natural_height: true,
                },
            ))
            .map_err(fit::PlanError::into_error)?;
            (width, 0.0)
        };
        let axes = super::planning::shared_axis_requirements(
            labels[0].as_ref(),
            labels[1].as_ref(),
            theme,
            PixelRect::new(0.0, natural.0, 0.0, natural.1),
        )
        .map_err(fit::PlanError::into_error)?;
        let body = theme.root_outer_width(natural.0 + axes[0])?;
        if horizontal {
            body.max(theme.root_outer_width(guide_size.0)?)
        } else {
            body + guide_size.0 + guide_gap
        }
    } else {
        0.0
    };
    let sources = source.prose();
    if requested_width.is_none() {
        for (text, appearance) in sources
            .into_iter()
            .zip(RootTitleBlockPlan::appearances(theme, sources))
        {
            if let Some(text) = text {
                preferred = preferred.max(theme.root_outer_width(text_width(
                    theme,
                    text.as_str(),
                    appearance,
                )?)?);
            }
        }
    }
    let width = request.select_width(preferred)?;
    let root_gutters = round.insets(
        &ComponentAddress::root().child(AddressRole::RootBody, 0)?,
        FeedbackKind::AxisGutter,
    );
    let grid_gutters = round.insets(&grid_owner, FeedbackKind::AxisGutter);
    let grid = allocate(
        Some(theme.root_inner_width(f64::from(width))? - root_gutters[0] - grid_gutters[0]),
        &gutters,
    )?;
    let axes = super::planning::shared_axis_requirements(
        labels[0].as_ref(),
        labels[1].as_ref(),
        theme,
        PixelRect::new(0.0, grid.size.0, 0.0, grid.size.1),
    )
    .map_err(fit::PlanError::into_error)?;
    let prose = RootTitleBlockPlan::content_reserves(
        theme,
        sources,
        theme.root_inner_width(f64::from(width))?,
        RootAlignments::resolve(theme, source.prose_alignments(), [None; 2]),
        &FitScope::root(),
    )
    .map_err(fit::PlanError::into_error)?;
    let height = root_dimension(theme.root_outer_height(&prose.height_with_body(
        grid.size.1.max(guide_size.1) + root_gutters[2].max(axes[2]) + grid_gutters[2],
    )?)?)?;
    Ok(RootSize::new(width, height))
}
