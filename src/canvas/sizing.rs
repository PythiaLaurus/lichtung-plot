//! Content sizing uses the rendering Grid's canonical track requirements.

use super::{Canvas, PreparedCanvasInputs, PreparedGuideDescriptor};
use crate::decorations::title_block::plan::{RootAlignments, RootTitleBlockPlan};
use crate::layout::canvas::PanelRequirements;
use crate::layout::fit::{self, FitScope};
use crate::layout::interior::sizing::{body_preference, integral, text_width};
use crate::layout::kernel::{FeedbackKind, FeedbackRound, RootSize};
use crate::layout::{AddressRole, ComponentAddress, PanelBox};
use crate::theme::ResolvedTheme;
use crate::{Error, Plot};

/// Root-local measured consequences. Source data, labels, typography and guide
/// semantics are immutable under this owner; coordinated inputs below are not.
#[derive(Default)]
pub(super) struct HeightMeasurements(std::cell::RefCell<Vec<Option<HeightMeasurement>>>);

#[derive(Debug, Clone, PartialEq, Eq)]
struct HeightInputs {
    width: i64,
    ranges: [[u64; 2]; 2],
    provenance: [crate::layout::RangeProvenance; 2],
    resolved_x: Option<[u64; 2]>,
    own_insets: [[i64; 4]; 2],
    parent_insets: [i64; 4],
    y_labels: bool,
    legend: crate::LegendPolicy,
    continuous_hidden: bool,
    scope: Option<String>,
}

struct HeightMeasurement {
    inputs: HeightInputs,
    height: f64,
}

impl HeightInputs {
    fn matches(&self, other: &Self, theme: &ResolvedTheme) -> Result<bool, Error> {
        if self.width != other.width
            || self.ranges != other.ranges
            || self.provenance != other.provenance
            || self.resolved_x != other.resolved_x
            || self.own_insets != other.own_insets
            || self.parent_insets != other.parent_insets
            || self.y_labels != other.y_labels
            || self.legend != other.legend
            || self.continuous_hidden != other.continuous_hidden
        {
            return Ok(false);
        }
        match (&self.scope, &other.scope) {
            (None, None) => Ok(true),
            (Some(a), Some(b)) if a.len() == b.len() => {
                for (a, b) in a.bytes().zip(b.bytes()) {
                    theme
                        .prepared_font_environment()?
                        .claim_processing_visits(1)
                        .map_err(|error| {
                            Error::Render(format!("height scope identity: {error:?}"))
                        })?;
                    if a != b {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }
    fn new(
        plot: &Plot,
        width: f64,
        parent_insets: [i64; 4],
        round: &FeedbackRound,
        owner: &ComponentAddress,
        theme: &ResolvedTheme,
    ) -> Result<Self, Error> {
        let axes = plot.prepared_axis_inputs().ok_or_else(|| {
            Error::Render("height measurement requires prepared cell axes".into())
        })?;
        let ranges = [
            plot.x_axis_spec().limits.unwrap_or(axes.ranges[0]),
            plot.y_axis_spec().limits.unwrap_or(axes.ranges[1]),
        ]
        .map(|range| [range.0.to_bits(), range.1.to_bits()]);
        let child = ComponentAddress::root().child(
            AddressRole::CanvasCell,
            owner.leaf_slot_index(),
            &theme.prepared_font_environment()?.generation_resources(),
        )?;
        let mut own_insets = [[0; 4]; 2];
        for (target, kind) in own_insets
            .iter_mut()
            .zip([FeedbackKind::AxisGutter, FeedbackKind::OwnedInk])
        {
            for (target, value) in target.iter_mut().zip(round.insets(&child, kind)?) {
                *target = crate::layout::kernel::nearest_spatial_quanta(value)?;
            }
        }
        let resources = theme.prepared_font_environment()?.generation_resources();
        let data = plot.data().with_axis_resources(&resources);
        Ok(Self {
            width: crate::layout::kernel::nearest_spatial_quanta(width)?,
            ranges,
            provenance: [plot.x_axis_spec().provenance, plot.y_axis_spec().provenance],
            resolved_x: plot
                .resolved_facet_x_range()
                .map(|range| [range.0.to_bits(), range.1.to_bits()]),
            own_insets,
            parent_insets,
            y_labels: plot.y_tick_labels_visible(),
            legend: plot.legend_policy_spec(),
            continuous_hidden: plot.continuous_guide_suppressed(),
            scope: plot
                .axis_scope()
                .map(|scope| data.copy_source_string(scope))
                .transpose()?,
        })
    }
}

impl HeightMeasurements {
    fn measured(
        &self,
        inputs: &HeightInputs,
        owner: &ComponentAddress,
        theme: &ResolvedTheme,
    ) -> Result<Option<f64>, Error> {
        theme
            .prepared_font_environment()?
            .claim_measurement_call()
            .map_err(|error| Error::Render(format!("content height lookup: {error:?}")))?;
        theme
            .prepared_font_environment()?
            .claim_processing_visits(1)
            .map_err(|error| Error::Render(format!("content height input lookup: {error:?}")))?;
        let index = usize::try_from(owner.leaf_slot_index())
            .map_err(|_| Error::Render("content height slot overflowed".into()))?;
        let entries = self.0.borrow();
        let Some(Some(entry)) = entries.get(index) else {
            return Ok(None);
        };
        if entry.inputs.matches(inputs, theme)? {
            Ok(Some(entry.height))
        } else {
            Ok(None)
        }
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "retain one measured consequence under its actual cell dependencies"
    )]
    pub(super) fn record(
        &self,
        plot: &Plot,
        width: f64,
        height: f64,
        parent_insets: [i64; 4],
        round: &FeedbackRound,
        owner: &ComponentAddress,
        theme: &ResolvedTheme,
    ) -> Result<(), Error> {
        let inputs = HeightInputs::new(plot, width, parent_insets, round, owner, theme)?;
        let index = usize::try_from(owner.leaf_slot_index())
            .map_err(|_| Error::Render("content height slot overflowed".into()))?;
        let mut entries = self.0.borrow_mut();
        let needed = index
            .checked_add(1)
            .ok_or_else(|| Error::Render("content height slot count overflowed".into()))?;
        if entries.len() < needed {
            let additional = needed - entries.len();
            fit::reserve(
                theme,
                &mut entries,
                additional,
                "content height measurement inputs",
            )?;
            entries.resize_with(needed, || None);
        }
        theme
            .prepared_font_environment()?
            .claim_cache_entries(1)
            .map_err(|error| Error::Render(format!("content height consequence: {error:?}")))?;
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
                guide.legend.compute_box_height() + theme.legend.outside_gap,
            )?;
        } else if guide.bounds.left >= panel.x1 || guide.bounds.right <= panel.x0 {
            height = height.max(crate::layout::kernel::required_length(
                guide.legend.compute_box_height(),
            )?);
        }
    }
    let gutters = round.insets(owner, FeedbackKind::AxisGutter)?;
    let ink = round.insets(owner, FeedbackKind::OwnedInk)?;
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
) -> Result<f64, Error> {
    let hard = plot.intrinsic_body_minimum(theme)?;
    let preference = body_preference(plot, theme, hard)?;
    let sources = [plot.title_text(), plot.subtitle_text(), plot.caption_text()];
    let present = sources
        .iter()
        .any(|source| source.is_some_and(|text| !text.is_empty()));
    let (top, bottom) = RootTitleBlockPlan::content_reserves(
        theme,
        sources,
        width - if present { 32.0 } else { 0.0 },
        RootAlignments::resolve(
            theme,
            [plot.title_alignment, plot.subtitle_alignment],
            [None; 2],
        ),
        &FitScope::root(),
    )
    .map_err(fit::PlanError::into_error)?;
    let identity = HeightInputs::new(plot, width, parent_insets, round, owner, theme)?;
    if let Some(measurements) = measurements {
        if let Some(height) = measurements.measured(&identity, owner, theme)? {
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
    Ok(height + local[2] + local[3] + top + bottom + if present { 20.0 } else { 0.0 })
}

pub(super) fn grid_gutters(
    source: &Canvas,
    theme: &ResolvedTheme,
    round: &FeedbackRound,
) -> Result<Vec<(usize, usize, PanelRequirements)>, Error> {
    let grid_owner = ComponentAddress::root().child(
        AddressRole::CanvasGrid,
        0,
        &theme.prepared_font_environment()?.generation_resources(),
    )?;
    let mut gutters = Vec::new();
    fit::reserve(
        theme,
        &mut gutters,
        source.panels.len(),
        "Canvas content gutters",
    )?;
    for index in 0..source.panels.len() {
        let owner = grid_owner.child(
            AddressRole::CanvasCell,
            u32::try_from(index)
                .map_err(|_| Error::Render("Canvas content address overflow".into()))?,
            &theme.prepared_font_environment()?.generation_resources(),
        )?;
        let [left, right, bottom, top] = round.insets(&owner, FeedbackKind::AxisGutter)?;
        let ink = round.insets(&owner, FeedbackKind::OwnedInk)?;
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
            &theme.prepared_font_environment()?.generation_resources(),
        )?;
        let alignment = round.insets(&owner, FeedbackKind::SharedAlignment)?;
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
    let sources = [
        source.title.as_deref(),
        source.subtitle.as_deref(),
        source.caption.as_deref(),
    ];
    let alignments = RootAlignments::resolve(
        theme,
        [source.title_alignment, source.subtitle_alignment],
        [None; 2],
    );
    let (top, bottom) = RootTitleBlockPlan::content_reserves(
        theme,
        sources,
        f64::from(width) - 32.0,
        alignments,
        &FitScope::root(),
    )
    .map_err(fit::PlanError::into_error)?;
    let padding = crate::layout::canvas::ROOT_PADDING;
    let height = integral(height + padding.top + top + padding.bottom + bottom)?;
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
    let resources = theme.prepared_font_environment()?.generation_resources();
    resources
        .claim_calls(1)
        .map_err(|error| Error::Render(format!("Canvas content sizing: {error:?}")))?;
    let mut plots = Vec::new();
    fit::reserve(
        theme,
        &mut plots,
        source.panels.len(),
        "Canvas content sizing children",
    )?;
    for (index, panel) in source.panels.iter().enumerate() {
        resources
            .claim_visits(1)
            .map_err(|error| Error::Render(format!("Canvas content child: {error:?}")))?;
        let Some(panel) = panel else {
            plots.push(None);
            continue;
        };
        let input = prepared.interiors[index]
            .as_ref()
            .expect("prepared Canvas plot");
        let data = panel.data().with_axis_resources(&resources);
        let mut plot = panel.admitted_clone(data)?;
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
        plot = input.attach(plot.with_render_scope(data.copy_source_string(scope)?))?;
        if let Some(limits) = prepared.shared_x.get(index).copied().flatten() {
            plot = plot.with_planned_x_limits(limits);
        }
        plots.push(Some(plot.with_parent_axis_gutters()));
    }
    source.coordinate_plots(&mut plots, &prepared.shared_x, round, &resources)?;
    let grid_owner = ComponentAddress::root().child(AddressRole::CanvasGrid, 0, &resources)?;
    let mut gutters = Vec::new();
    fit::reserve(theme, &mut gutters, plots.len(), "Canvas content gutters")?;
    for index in 0..plots.len() {
        let owner = grid_owner.child(
            AddressRole::CanvasCell,
            u32::try_from(index)
                .map_err(|_| Error::Render("Canvas content address overflow".into()))?,
            &resources,
        )?;
        let [left, right, bottom, top] = round.insets(&owner, FeedbackKind::AxisGutter)?;
        let ink = round.insets(&owner, FeedbackKind::OwnedInk)?;
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
            &resources,
        )?;
        let alignment = round.insets(&owner, FeedbackKind::SharedAlignment)?;
        for &index in group {
            gutters[index].2.left = gutters[index].2.left.max(alignment[0]);
            gutters[index].2.right = gutters[index].2.right.max(alignment[1]);
        }
    }
    let mut guide = (0.0, 0.0);
    if let Some(PreparedGuideDescriptor::Discrete((title, entries))) = &prepared.shared_guide {
        guide = crate::decorations::legend::Legend::natural_preference(title, entries, theme)?;
    } else if let Some(PreparedGuideDescriptor::Continuous(descriptor)) = &prepared.shared_guide {
        guide = crate::layout::interior::sizing::continuous_preference(
            descriptor,
            theme,
            &FitScope::root().child(AddressRole::CanvasSharedLegend, 0, &resources)?,
        )?;
    }
    let guide_gap = if guide.0 > 0.0 {
        source.shared_legend_gap.unwrap_or(theme.legend.shared_gap)
    } else {
        0.0
    };
    let allocate = |width: Option<f64>, gutters: &[(usize, usize, PanelRequirements)]| {
        fit::measured(super::tracks::allocate(
            PanelBox::new(0.0, width.unwrap_or(0.0), 0.0, 0.0),
            (source.rows, source.columns),
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
                natural_width: width.is_none(),
                natural_height: true,
            }),
        ))
        .map_err(fit::PlanError::into_error)
    };
    // Initial selection excludes feedback. Empty/trailing tracks and spanning text
    // use precisely the rendering allocator's declared topology.
    let mut preferred_width = if requested_width.is_none() {
        allocate(None, &[])?.size.0 + guide.0 + guide_gap + 32.0
    } else {
        0.0
    };
    let sources = [
        source.title.as_deref(),
        source.subtitle.as_deref(),
        source.caption.as_deref(),
    ];
    if requested_width.is_none() {
        for (text, appearance) in sources.into_iter().zip([
            &theme.typography.title,
            &theme.typography.subtitle,
            &theme.typography.caption,
        ]) {
            if let Some(text) = text {
                preferred_width = preferred_width.max(text_width(theme, text, appearance)? + 32.0);
            }
        }
    }
    let width = requested_width.map_or_else(|| integral(preferred_width), Ok)?;
    let grid = allocate(Some(f64::from(width) - 32.0), &gutters)?;
    let (top, bottom) = RootTitleBlockPlan::content_reserves(
        theme,
        sources,
        f64::from(width) - 32.0,
        RootAlignments::resolve(
            theme,
            [source.title_alignment, source.subtitle_alignment],
            [None; 2],
        ),
        &FitScope::root(),
    )
    .map_err(fit::PlanError::into_error)?;
    let padding = crate::layout::canvas::ROOT_PADDING;
    let height = integral(grid.size.1.max(guide.1) + padding.top + top + padding.bottom + bottom)?;
    Ok(RootSize::new(width, height))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_width_local_gutter_replacement_remeasures_and_contracts_height() {
        let theme = crate::Theme::Publication.resolve();
        theme.prepare_font_environment().unwrap();
        let resources = theme
            .prepared_font_environment()
            .unwrap()
            .generation_resources();
        let source = Plot::scatter(
            crate::NumericValues::new("x", [0.0, 1.0]),
            crate::NumericValues::new("y", [1.0, 2.0]),
        )
        .erase();
        let inputs = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
        let plot = inputs.attach(source).unwrap();
        let owner = ComponentAddress::root()
            .child(AddressRole::CanvasGrid, 0, &resources)
            .unwrap()
            .child(AddressRole::CanvasCell, 0, &resources)
            .unwrap();
        let child = ComponentAddress::root()
            .child(AddressRole::CanvasCell, 0, &resources)
            .unwrap();
        let measurements = HeightMeasurements::default();
        let mut history = Vec::new();
        crate::layout::plan_exact_root(
            800,
            600,
            |_| Ok(()),
            |(), assigned, round| {
                let height = cell_height(
                    &plot,
                    &theme,
                    160.0,
                    &inputs,
                    false,
                    round,
                    &owner,
                    [0; 4],
                    Some(&measurements),
                )?;
                let fresh = cell_height(
                    &plot, &theme, 160.0, &inputs, false, round, &owner, [0; 4], None,
                )?;
                assert_eq!(height.to_bits(), fresh.to_bits());
                measurements.record(&plot, 160.0, height, [0; 4], round, &owner, &theme)?;
                let bottom = if history.is_empty() { 100.0 } else { 0.0 };
                history.push(height);
                round.request_insets(
                    &child,
                    &child,
                    FeedbackKind::AxisGutter,
                    [0.0, 0.0, bottom, 0.0],
                )?;
                Ok(crate::layout::MeasuredLayout::new(assigned, ()))
            },
            |(), _, (), _| Ok(()),
        )
        .unwrap();
        assert_eq!(history.len(), 3);
        assert_eq!(history[0].to_bits(), history[2].to_bits());
        assert_eq!((history[1] - history[2]).to_bits(), 100.0_f64.to_bits());
    }

    #[test]
    fn retained_height_rejects_width_domain_gutter_and_visibility_changes() {
        let theme = crate::Theme::Publication.resolve();
        theme.prepare_font_environment().unwrap();
        let resources = theme
            .prepared_font_environment()
            .unwrap()
            .generation_resources();
        let source = Plot::beeswarm(
            crate::CategoryValues::new("group", ["a"; 40]).unwrap(),
            crate::NumericValues::new("x", (0..40).map(f64::from)),
            crate::NumericValues::new("color", [0.0; 40]),
        )
        .erase();
        let inputs = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
        let plot = inputs
            .attach(source)
            .unwrap()
            .with_suppressed_continuous_guide();
        let measurements = HeightMeasurements::default();
        crate::layout::plan_exact_root(
            800,
            600,
            |_| Ok(()),
            |(), assigned, round| {
                let owner = ComponentAddress::root()
                    .child(AddressRole::CanvasGrid, 0, &resources)?
                    .child(AddressRole::CanvasCell, 0, &resources)?;
                let key = HeightInputs::new(&plot, 160.0, [0; 4], round, &owner, &theme)?;
                let original = cell_height(
                    &plot, &theme, 160.0, &inputs, false, round, &owner, [0; 4], None,
                )?;
                measurements.record(&plot, 160.0, original, [0; 4], round, &owner, &theme)?;
                assert!(measurements.measured(&key, &owner, &theme)?.is_some());
                let smaller = HeightInputs::new(&plot, 120.0, [0; 4], round, &owner, &theme)?;
                assert!(measurements.measured(&smaller, &owner, &theme)?.is_none());
                let widened = plot.clone().with_planned_x_limits((0.0, 1_000_000.0));
                let domain = HeightInputs::new(&widened, 160.0, [0; 4], round, &owner, &theme)?;
                assert!(measurements.measured(&domain, &owner, &theme)?.is_none());
                let recomputed = cell_height(
                    &widened,
                    &theme,
                    160.0,
                    &inputs,
                    false,
                    round,
                    &owner,
                    [0; 4],
                    Some(&measurements),
                )?;
                assert!(
                    recomputed > original,
                    "occupied packing must be remeasured at the coordinated domain"
                );
                let parent =
                    HeightInputs::new(&plot, 160.0, [1024, 0, 0, 0], round, &owner, &theme)?;
                assert!(measurements.measured(&parent, &owner, &theme)?.is_none());
                let hidden = plot.clone().with_y_tick_labels_visible(false);
                let visibility = HeightInputs::new(&hidden, 160.0, [0; 4], round, &owner, &theme)?;
                assert!(measurements
                    .measured(&visibility, &owner, &theme)?
                    .is_none());
                // Replacement can contract under a later actual dependency identity.
                measurements.record(&widened, 160.0, recomputed, [0; 4], round, &owner, &theme)?;
                let contracted = cell_height(
                    &plot,
                    &theme,
                    160.0,
                    &inputs,
                    false,
                    round,
                    &owner,
                    [0; 4],
                    Some(&measurements),
                )?;
                assert_eq!(contracted.to_bits(), original.to_bits());
                assert!(contracted < recomputed);
                measurements.record(&plot, 160.0, original, [0; 4], round, &owner, &theme)?;
                let retained = cell_height(
                    &plot,
                    &theme,
                    160.0,
                    &inputs,
                    false,
                    round,
                    &owner,
                    [0; 4],
                    Some(&measurements),
                )?;
                assert_eq!(retained.to_bits(), contracted.to_bits());
                assert_eq!(
                    measurements
                        .measured(&key, &owner, &theme)?
                        .unwrap()
                        .to_bits(),
                    crate::layout::kernel::required_length(original)?.to_bits()
                );
                Ok(crate::layout::MeasuredLayout::new(assigned, ()))
            },
            |(), _, (), _| Ok(()),
        )
        .unwrap();
    }
}
