//! Facets compose their owning Plot's request; cells have no standalone roots.

use super::{data, tracks, PreparedFacetGuide};
use crate::decorations::title_block::plan::{RootAlignments, RootTitleBlockPlan};
use crate::layout::canvas::PanelRequirements;
use crate::layout::fit::{self, FitScope};
use crate::layout::interior::sizing::{integral, text_width};
use crate::layout::kernel::{FeedbackKind, FeedbackRound, RootSize};
use crate::layout::{AddressRole, ComponentAddress, PanelBox};
use crate::theme::ResolvedTheme;
use crate::{Error, Plot};

#[expect(
    clippy::too_many_arguments,
    reason = "one owning facet Grid retains semantic inputs and consumed geometry"
)]
pub(super) fn grid_height(
    plot: &Plot,
    panels: &[data::FacetPanel],
    shape: (usize, usize),
    minima: &[(f64, f64)],
    theme: &ResolvedTheme,
    round: &FeedbackRound,
    width: f64,
    inputs: &crate::layout::interior::PreparedInteriorInputs,
) -> Result<fit::FitOutcome<f64>, Error> {
    let resources = theme.prepared_font_environment()?.operation();
    let data = plot.data().with_operation(&resources);
    let owner = ComponentAddress::root().child(AddressRole::FacetGrid, 0)?;
    let mut gutters = Vec::new();
    crate::data::source::reserve_source(&mut gutters, panels.len())?;
    for (index, panel) in panels.iter().enumerate() {
        let address = owner.child(
            AddressRole::FacetCell,
            u32::try_from(index)
                .map_err(|_| Error::Render("facet content address overflow".into()))?,
        )?;
        let [left, right, bottom, top] = round.insets(&address, FeedbackKind::AxisGutter);
        gutters.push((
            panel.row,
            panel.column,
            PanelRequirements {
                left,
                right,
                bottom,
                top,
            },
        ));
    }
    Ok(
        match tracks::allocate(
            data,
            PanelBox::new(0.0, width, 0.0, 0.0),
            shape,
            panels,
            minima,
            &gutters,
            theme,
            &FitScope::root(),
            Some(&tracks::ContentSizing {
                plot,
                inputs,
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
    source: &Plot,
    theme: &ResolvedTheme,
    request: RootSize,
    round: &FeedbackRound,
    guide: &PreparedFacetGuide,
    inputs: &crate::layout::interior::PreparedInteriorInputs,
) -> Result<RootSize, Error> {
    let resources = theme.prepared_font_environment()?.operation();

    let data = source.data().with_operation(&resources);
    let selectors = data::facet_selectors(source, data)?;
    let panels = data::extract_panels(source, &selectors, None, data)?;
    let labels = source.prepare_axis_labels(None)?;
    let base = data::base_plot(source, source.kind(), &resources)?;
    let mut minima = Vec::new();
    crate::data::source::reserve_source(&mut minima, panels.len())?;
    let mut gutters = Vec::new();
    crate::data::source::reserve_source(&mut gutters, panels.len())?;
    let grid_owner = ComponentAddress::root().child(AddressRole::FacetGrid, 0)?;
    for (index, panel) in panels.iter().enumerate() {
        let plot = base.clone().with_render_scope(panel.render_scope()?);
        minima.push(plot.intrinsic_body_minimum(theme)?);
        let owner = grid_owner.child(
            AddressRole::FacetCell,
            u32::try_from(index)
                .map_err(|_| Error::Render("facet content address overflowed".into()))?,
        )?;
        let [left, right, bottom, top] = round.insets(&owner, FeedbackKind::AxisGutter);
        gutters.push((
            panel.row,
            panel.column,
            PanelRequirements {
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
    let guide_size = if let Some(descriptor) = guide.descriptor.as_ref().filter(|_| outside) {
        crate::decorations::legend::Legend::natural_preference(descriptor, theme)?
    } else {
        (0.0, 0.0)
    };
    let guide_gap = if guide_size.0 > 0.0 {
        theme.legend_outside_gap()?
    } else {
        0.0
    };
    let allocate = |width: Option<f64>, gutters: &[(usize, usize, PanelRequirements)]| {
        fit::measured(tracks::allocate(
            data,
            PanelBox::new(0.0, width.unwrap_or(0.0), 0.0, 0.0),
            (selectors.rows, selectors.columns),
            &panels,
            &minima,
            gutters,
            theme,
            &FitScope::root(),
            Some(&tracks::ContentSizing {
                plot: &base,
                inputs,
                round,
                natural_width: width.is_none(),
                natural_height: true,
            }),
        ))
        .map_err(fit::PlanError::into_error)
    };
    let (requested_width, _) = request.requested_dimensions();
    let mut preferred = if requested_width.is_none() {
        let natural = allocate(None, &[])?;
        let axes = super::planning::shared_axis_requirements(
            labels[0].as_deref(),
            labels[1].as_deref(),
            theme,
            PanelBox::new(0.0, natural.size.0, 0.0, natural.size.1),
        )
        .map_err(fit::PlanError::into_error)?;
        let body = theme.root_outer_width(natural.size.0 + axes[0])?;
        if horizontal {
            body.max(theme.root_outer_width(guide_size.0)?)
        } else {
            body + guide_size.0 + guide_gap
        }
    } else {
        0.0
    };
    let sources = [
        source.title_text(),
        source.subtitle_text(),
        source.caption_text(),
    ];
    if requested_width.is_none() {
        for (text, appearance) in sources.into_iter().zip([
            &theme.typography.root.title,
            &theme.typography.root.subtitle,
            &theme.typography.root.caption,
        ]) {
            if let Some(text) = text {
                preferred =
                    preferred.max(theme.root_outer_width(text_width(theme, text, appearance)?)?);
            }
        }
    }
    let width = requested_width.map_or_else(|| integral(preferred), Ok)?;
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
        labels[0].as_deref(),
        labels[1].as_deref(),
        theme,
        PanelBox::new(0.0, grid.size.0, 0.0, grid.size.1),
    )
    .map_err(fit::PlanError::into_error)?;
    let prose = RootTitleBlockPlan::content_reserves(
        theme,
        sources,
        theme.root_inner_width(f64::from(width))?,
        RootAlignments::resolve(
            theme,
            [source.title_alignment, source.subtitle_alignment],
            [None; 2],
        ),
        &FitScope::root(),
    )
    .map_err(fit::PlanError::into_error)?;
    let height = integral(theme.root_outer_height(&prose.height_with_body(
        grid.size.1.max(guide_size.1) + root_gutters[2].max(axes[2]) + grid_gutters[2],
    )?)?)?;
    Ok(RootSize::new(width, height))
}
