//! Content sizing uses the rendering Grid's canonical track requirements.

use super::{Canvas, PreparedCanvasInputs};
use crate::decorations::legend::discrete::scoped::{
    ConsumedGuideDependency, GuideUse, ScopedGuideInputs,
};
use crate::decorations::title_block::plan::{RootAlignments, RootTitleBlockPlan};
use crate::layout::canvas::PanelRequirements;
use crate::layout::fit::{self, FitScope};
use crate::layout::interior::sizing::{body_preference, integral, text_width};
use crate::layout::kernel::{FeedbackKind, FeedbackRound, RootSize};
use crate::layout::{AddressRole, ComponentAddress, PanelBox};
use crate::theme::ResolvedTheme;
use crate::{Error, Plot};

#[cfg(test)]
mod tests;

/// Root-local measured consequences. Source data, labels, typography and guide
/// semantics are immutable under this owner; coordinated inputs below are not.
#[derive(Default)]
pub(super) struct HeightMeasurements(std::cell::RefCell<Vec<Option<HeightMeasurement>>>);

#[derive(Debug, Clone, PartialEq)]
struct HeightInputs {
    prose: [crate::text::TextSlot; 3],
    axis_titles: [crate::text::TextSlot; 2],
    width: i64,
    ranges: [[u64; 2]; 2],
    provenance: [crate::layout::RangeProvenance; 2],
    generation: [crate::TickGeneration; 2],
    category_labels: [crate::CategoryLabelSelection; 2],
    resolved_x: Option<[u64; 2]>,
    own_insets: [[i64; 4]; 2],
    parent_insets: [i64; 4],
    y_labels: bool,
    legend: crate::LegendPolicy,
    continuous_hidden: bool,
    numeric: crate::numeric::inputs::NumericInputs,
    alpha: crate::alpha::AlphaInputs,
    guides: ScopedGuideInputs,
    scope: Option<String>,
}

struct HeightMeasurement {
    inputs: HeightInputs,
    height: f64,
}

impl HeightInputs {
    fn matches_assignment(&self, other: &Self) -> bool {
        if self.width != other.width
            || self.prose != other.prose
            || self.axis_titles != other.axis_titles
            || self.ranges != other.ranges
            || self.provenance != other.provenance
            || self.generation != other.generation
            || self.category_labels != other.category_labels
            || self.resolved_x != other.resolved_x
            || self.own_insets != other.own_insets
            || self.parent_insets != other.parent_insets
            || self.y_labels != other.y_labels
            || self.legend != other.legend
            || self.continuous_hidden != other.continuous_hidden
            || self.numeric != other.numeric
            || self.alpha != other.alpha
            || self.guides != other.guides
        {
            return false;
        }
        match (&self.scope, &other.scope) {
            (None, None) => true,
            (Some(a), Some(b)) if a.len() == b.len() => {
                for (a, b) in a.bytes().zip(b.bytes()) {
                    if a != b {
                        return false;
                    }
                }
                true
            }
            _ => false,
        }
    }
    fn new(
        plot: &Plot,
        width: f64,
        parent_insets: [i64; 4],
        round: &FeedbackRound,
        owner: &ComponentAddress,
        per_plot: bool,
        dependencies: &[ConsumedGuideDependency],
    ) -> Result<Self, Error> {
        let axes = plot.prepared_axis_inputs().ok_or_else(|| {
            Error::Render("height measurement requires prepared cell axes".into())
        })?;
        let ranges = [
            plot.x_axis_spec().limits.unwrap_or(axes.ranges[0]),
            plot.y_axis_spec().limits.unwrap_or(axes.ranges[1]),
        ]
        .map(|range| [range.0.to_bits(), range.1.to_bits()]);
        let child =
            ComponentAddress::root().child(AddressRole::CanvasCell, owner.leaf_slot_index())?;
        let mut own_insets = [[0; 4]; 2];
        for (target, kind) in own_insets
            .iter_mut()
            .zip([FeedbackKind::AxisGutter, FeedbackKind::OwnedInk])
        {
            for (target, value) in target.iter_mut().zip(round.insets(&child, kind)) {
                *target = crate::layout::kernel::nearest_spatial_quanta(value)?;
            }
        }

        let use_kind = if plot.legend_policy_spec() == crate::LegendPolicy::Hidden {
            GuideUse::Hidden
        } else if per_plot {
            GuideUse::Local
        } else {
            GuideUse::SharedSuppressed
        };
        let local = if use_kind == GuideUse::Local {
            plot.prepared_discrete_guide()?
                .map(|descriptor| {
                    crate::decorations::legend::discrete::GuideSlot::new(
                        descriptor,
                        &child,
                        plot.legend_position_spec(),
                    )
                })
                .transpose()?
        } else {
            None
        };
        Ok(Self {
            prose: plot.copy_prose()?,
            axis_titles: plot.copy_axis_titles()?,
            width: crate::layout::kernel::nearest_spatial_quanta(width)?,
            ranges,
            provenance: [plot.x_axis_spec().provenance, plot.y_axis_spec().provenance],
            generation: [
                plot.x_axis_spec().generation.clone(),
                plot.y_axis_spec().generation.clone(),
            ],
            category_labels: [
                plot.x_axis_spec().category_labels.clone(),
                plot.y_axis_spec().category_labels.clone(),
            ],
            resolved_x: plot
                .resolved_facet_x_range()
                .map(|range| [range.0.to_bits(), range.1.to_bits()]),
            own_insets,
            parent_insets,
            y_labels: plot.y_tick_labels_visible(),
            legend: plot.legend_policy_spec(),
            continuous_hidden: plot.continuous_guide_suppressed(),
            numeric: crate::numeric::inputs::NumericInputs::new(plot)?,
            alpha: plot.alpha_inputs().clone(),
            guides: ScopedGuideInputs::new(plot, use_kind, local.as_ref(), dependencies)?,
            scope: plot
                .axis_scope()
                .map(crate::data::source::copy_source_string)
                .transpose()?,
        })
    }
}

impl HeightMeasurements {
    fn measured(
        &self,
        inputs: &HeightInputs,
        owner: &ComponentAddress,
        round: &FeedbackRound,
    ) -> Result<Option<f64>, Error> {
        let index = usize::try_from(owner.leaf_slot_index())
            .map_err(|_| Error::Render("content height slot overflowed".into()))?;
        let entries = self.0.borrow();
        let Some(Some(entry)) = entries.get(index) else {
            return Ok(None);
        };
        if entry.inputs.matches_assignment(inputs) && entry.inputs.guides.stable(round) {
            Ok(Some(entry.height))
        } else {
            Ok(None)
        }
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "record one exact width-dependent consequence with its declared guide dependencies"
    )]
    pub(super) fn record(
        &self,
        plot: &Plot,
        width: f64,
        height: f64,
        parent_insets: [i64; 4],
        round: &FeedbackRound,
        owner: &ComponentAddress,
        dependencies: &[ConsumedGuideDependency],
    ) -> Result<(), Error> {
        let inputs =
            HeightInputs::new(plot, width, parent_insets, round, owner, true, dependencies)?;
        let index = usize::try_from(owner.leaf_slot_index())
            .map_err(|_| Error::Render("content height slot overflowed".into()))?;
        let mut entries = self.0.borrow_mut();
        let needed = index
            .checked_add(1)
            .ok_or_else(|| Error::Render("content height slot count overflowed".into()))?;
        if entries.len() < needed {
            let additional = needed - entries.len();
            fit::reserve(
                &mut entries,
                additional,
                "content height measurement inputs",
            )?;
            entries.resize_with(needed, || None);
        }

        entries[index] = Some(HeightMeasurement {
            inputs,
            height: crate::layout::kernel::required_length(height)?,
        });
        Ok(())
    }
}

pub(super) fn measured_preference(
    plot: &Plot,
    plan: &crate::decorations::plan::PanelPlan,
    local_text_height: f64,
    round: &FeedbackRound,
    owner: &ComponentAddress,
    theme: &ResolvedTheme,
) -> Result<f64, Error> {
    let hard = plot.intrinsic_body_minimum(theme)?;
    let preference = body_preference(plot, theme, hard)?;
    let mut height = plot.natural_body_height(theme, plan.panel_box.width(), preference.1)?;
    if let Some(guide) = &plan.legend {
        let panel = plan.panel_box;
        if guide.bounds.bottom >= panel.y1 || guide.bounds.top <= panel.y0 {
            height += crate::layout::kernel::required_length(
                guide.legend.compute_box_height() + theme.legend_outside_gap()?,
            )?;
        } else if guide.bounds.left >= panel.x1 || guide.bounds.right <= panel.x0 {
            height = height.max(crate::layout::kernel::required_length(
                guide.legend.compute_box_height(),
            )?);
        }
    }
    let gutters = round.insets(owner, FeedbackKind::AxisGutter);
    let ink = round.insets(owner, FeedbackKind::OwnedInk);
    crate::layout::kernel::required_length(
        height + gutters[2] + gutters[3] + ink[2] + ink[3] + local_text_height,
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "one width-dependent cell call receives its prepared and coordinated identity"
)]
pub(super) fn cell_height(
    plot: &Plot,
    theme: &ResolvedTheme,
    width: f64,
    inputs: &crate::layout::interior::PreparedInteriorInputs,
    per_plot: bool,
    round: &FeedbackRound,
    owner: &ComponentAddress,
    parent_insets: [i64; 4],
    measurements: Option<&HeightMeasurements>,
    dependencies: &[ConsumedGuideDependency],
) -> Result<f64, Error> {
    let hard = plot.intrinsic_body_minimum(theme)?;
    let preference = body_preference(plot, theme, hard)?;
    let sources = plot.prose();
    let present = sources.iter().any(Option::is_some);
    let prose = RootTitleBlockPlan::content_reserves(
        theme,
        sources,
        if present {
            theme.root_inner_width(width)?
        } else {
            width
        },
        RootAlignments::resolve(theme, plot.prose_alignments(), [None; 2]),
        &FitScope::root(),
    )
    .map_err(fit::PlanError::into_error)?;
    let identity = HeightInputs::new(
        plot,
        width,
        parent_insets,
        round,
        owner,
        per_plot,
        dependencies,
    )?;
    if let Some(measurements) = measurements {
        if let Some(height) = measurements.measured(&identity, owner, round)? {
            return Ok(height);
        }
    }
    let mut local = [0.0; 4];
    for (index, value) in local.iter_mut().enumerate() {
        let quanta = identity.own_insets[0][index]
            .checked_add(identity.own_insets[1][index])
            .ok_or_else(|| Error::Render("cell owned inset sum overflowed".into()))?;
        #[expect(
            clippy::cast_precision_loss,
            reason = "cell insets are root-bounded spatial quanta"
        )]
        {
            *value = quanta as f64 / 1024.0;
        }
    }
    let body_width = width - local[0] - local[1];
    let guide = if per_plot {
        crate::layout::interior::sizing::guided_preference(plot, inputs, theme)?
    } else {
        None
    };
    let height = if guide.is_some() {
        crate::layout::interior::sizing::guided_body_height(
            plot,
            theme,
            body_width,
            guide.as_ref(),
            &FitScope::new(owner, 0),
        )?
    } else {
        plot.natural_body_height(theme, body_width, preference.1)?
    };

    let height = prose.height_with_body(height + local[2] + local[3])?;
    if present {
        theme.root_outer_height(&height)
    } else {
        crate::theme::composition::hard(&height)
    }
}

pub(super) fn grid_gutters(
    source: &Canvas,

    round: &FeedbackRound,
) -> Result<Vec<(usize, usize, PanelRequirements)>, Error> {
    let grid_owner = ComponentAddress::root().child(AddressRole::CanvasGrid, 0)?;
    let mut gutters = Vec::new();
    fit::reserve(
        &mut gutters,
        source.placements.len(),
        "Canvas content gutters",
    )?;
    for index in 0..source.placements.len() {
        let owner = grid_owner.child(
            AddressRole::CanvasCell,
            u32::try_from(index)
                .map_err(|_| Error::Render("Canvas content address overflow".into()))?,
        )?;
        let [left, right, bottom, top] = round.insets(&owner, FeedbackKind::AxisGutter);
        let ink = round.insets(&owner, FeedbackKind::OwnedInk);
        gutters.push((
            index / source.columns,
            index % source.columns,
            PanelRequirements {
                left,
                right,
                bottom,
                top: top + ink[3],
            },
        ));
    }
    for (index, group) in source.x_range_groups.iter().enumerate() {
        let owner = grid_owner.child(
            AddressRole::CanvasRangeGroup,
            u32::try_from(index)
                .map_err(|_| Error::Render("Canvas range group address overflow".into()))?,
        )?;
        let alignment = round.insets(&owner, FeedbackKind::SharedAlignment);
        for &index in group {
            gutters[index].2.left = gutters[index].2.left.max(alignment[0]);
            gutters[index].2.right = gutters[index].2.right.max(alignment[1]);
        }
    }
    Ok(gutters)
}

pub(super) fn content_frame(
    source: &Canvas,
    theme: &ResolvedTheme,
    width: u32,
    height: f64,
) -> Result<crate::layout::RootFramePlan, Error> {
    let sources = source.prose();
    let alignments = RootAlignments::resolve(theme, source.prose_alignments(), [None; 2]);
    let prose = RootTitleBlockPlan::content_reserves(
        theme,
        sources,
        theme.root_inner_width(f64::from(width))?,
        alignments,
        &FitScope::root(),
    )
    .map_err(fit::PlanError::into_error)?;
    let height = integral(theme.root_outer_height(&prose.height_with_body(height)?)?)?;
    crate::layout::RootFramePlan::measure(
        (width, height),
        theme,
        sources,
        alignments,
        (
            crate::layout::canvas::MINIMUM_PANEL_SPAN,
            crate::layout::canvas::MINIMUM_PANEL_SPAN,
        ),
    )
    .map_err(fit::PlanError::into_error)
}

#[expect(
    clippy::too_many_lines,
    reason = "one root composes prepared scientific cells, shared gutters, a guide and root text"
)]
pub(super) fn measure(
    source: &Canvas,
    prepared: &PreparedCanvasInputs,
    requested: RootSize,
    round: &FeedbackRound,
    measurements: &HeightMeasurements,
) -> Result<RootSize, Error> {
    let (requested_width, requested_height) = requested.requested_dimensions();
    if let (Some(width), Some(height)) = (requested_width, requested_height) {
        return Ok(RootSize::new(width, height));
    }
    let theme = prepared.themes.root();

    let mut plots = Vec::new();
    fit::reserve(
        &mut plots,
        source.placements.len(),
        "Canvas content sizing children",
    )?;
    for (index, panel) in source.placements.plots().enumerate() {
        let Some(panel) = panel else {
            plots.push(None);
            continue;
        };
        let input = prepared.interiors[index]
            .as_ref()
            .expect("prepared Canvas plot");

        let mut plot = panel.clone();
        if let Some(labels) = &source.labels {
            plot = plot.labels(labels.clone());
        }
        let scope = input
            .axes
            .first()
            .and_then(|(scope, _)| scope.as_deref())
            .ok_or_else(|| {
                Error::Render("Canvas content child requires its prepared scope".into())
            })?;
        plot = input
            .attach(plot.with_render_scope(crate::data::source::copy_source_string(scope)?))?;
        if let Some(limits) = prepared.shared_x.get(index).copied().flatten() {
            plot = plot.with_planned_x_limits(limits);
        }
        plots.push(Some(plot.with_parent_axis_gutters()));
    }
    source.coordinate_plots(&mut plots, &prepared.shared_x, round)?;
    let grid_owner = ComponentAddress::root().child(AddressRole::CanvasGrid, 0)?;
    let mut gutters = Vec::new();
    fit::reserve(&mut gutters, plots.len(), "Canvas content gutters")?;
    for index in 0..plots.len() {
        let owner = grid_owner.child(
            AddressRole::CanvasCell,
            u32::try_from(index)
                .map_err(|_| Error::Render("Canvas content address overflow".into()))?,
        )?;
        let [left, right, bottom, top] = round.insets(&owner, FeedbackKind::AxisGutter);
        let ink = round.insets(&owner, FeedbackKind::OwnedInk);
        gutters.push((
            index / source.columns,
            index % source.columns,
            PanelRequirements {
                left,
                right,
                bottom,
                top: top + ink[3],
            },
        ));
    }
    for (index, group) in source.x_range_groups.iter().enumerate() {
        let owner = grid_owner.child(
            AddressRole::CanvasRangeGroup,
            u32::try_from(index)
                .map_err(|_| Error::Render("Canvas range group address overflow".into()))?,
        )?;
        let alignment = round.insets(&owner, FeedbackKind::SharedAlignment);
        for &index in group {
            gutters[index].2.left = gutters[index].2.left.max(alignment[0]);
            gutters[index].2.right = gutters[index].2.right.max(alignment[1]);
        }
    }
    let mut guide = (0.0, 0.0);
    if let Some(descriptor) = &prepared.shared_guide {
        guide = crate::decorations::legend::Legend::natural_preference(descriptor, theme)?;
    }
    let guide_gap = if guide.0 > 0.0 {
        theme.legend_shared_gap()?
    } else {
        0.0
    };
    let allocate = |width: Option<f64>, gutters: &[(usize, usize, PanelRequirements)]| {
        fit::measured(super::tracks::allocate(
            PanelBox::new(0.0, width.unwrap_or(0.0), 0.0, 0.0),
            &source.placements,
            &plots,
            &prepared.text_panel_intrinsics,
            &prepared.plot_intrinsics,
            &source.column_headers,
            &source.row_headers,
            &source.row_titles,
            gutters,
            &prepared.themes,
            &FitScope::root(),
            Some(&super::tracks::ContentSizing {
                source,
                interiors: &prepared.interiors,
                round,
                measurements: Some(measurements),
                shared_allocation: None,
                natural_width: width.is_none(),
                natural_height: true,
            }),
        ))
        .map_err(fit::PlanError::into_error)
    };
    // Initial selection excludes feedback. Empty/trailing tracks and spanning text
    // use precisely the rendering allocator's declared topology.
    let mut preferred_width = if requested_width.is_none() {
        theme.root_outer_width(allocate(None, &[])?.size.0 + guide.0 + guide_gap)?
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
                preferred_width = preferred_width.max(theme.root_outer_width(text_width(
                    theme,
                    text.as_str(),
                    appearance,
                )?)?);
            }
        }
    }
    let width = requested_width.map_or_else(|| integral(preferred_width), Ok)?;
    let grid = allocate(Some(theme.root_inner_width(f64::from(width))?), &gutters)?;
    let prose = RootTitleBlockPlan::content_reserves(
        theme,
        sources,
        theme.root_inner_width(f64::from(width))?,
        RootAlignments::resolve(theme, source.prose_alignments(), [None; 2]),
        &FitScope::root(),
    )
    .map_err(fit::PlanError::into_error)?;
    let height =
        integral(theme.root_outer_height(&prose.height_with_body(grid.size.1.max(guide.1))?)?)?;
    Ok(RootSize::new(width, height))
}
