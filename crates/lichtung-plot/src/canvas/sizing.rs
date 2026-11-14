//! Content sizing uses the rendering Grid's canonical track requirements.

use super::Canvas;
use crate::decorations::legend::discrete::scoped::{
    ConsumedGuideDependency, GuideConsumption, GuideUse, ScopedGuideInputs,
};
use crate::decorations::title_block::plan::{RootAlignments, RootTitleBlockPlan};
use crate::layout::canvas::EdgeSpaceRequirements;
use crate::layout::fit::{self, FitScope};
use crate::layout::interior::sizing::{body_preference, text_width};
use crate::layout::kernel::{root_dimension, FeedbackKind, FeedbackRound, RootSize};
use crate::layout::{AddressRole, ComponentAddress, PixelRect};
use crate::orchestrator::registration::{PlotOccurrenceId, PlotRegistration};
use crate::orchestrator::root::Root;
use crate::Error;

#[cfg(test)]
mod tests;

#[cfg(test)]
thread_local! {
    static HEIGHT_TRACES_ARMED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static CELL_HEIGHT_STEPS: std::cell::RefCell<Vec<&'static str>> = const { std::cell::RefCell::new(Vec::new()) };
    pub(super) static CHILD_HEIGHT_STEPS: std::cell::RefCell<Vec<(&'static str, usize, [f64; 3])>> = const { std::cell::RefCell::new(Vec::new()) };
    #[expect(clippy::type_complexity, reason = "test-only observation at the original HeightInputs constructor")]
    static HEIGHT_AXIS_INPUTS: std::cell::RefCell<Vec<(Option<RootSize>, [[u64; 2]; 2], Option<[u64; 2]>)>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Only the height fixtures arm these value-only observations. The marker keeps
/// cleanup on the thread whose TLS was armed, including during unwinding.
#[cfg(test)]
#[must_use]
struct HeightTraceGuard {
    _same_thread: std::marker::PhantomData<std::rc::Rc<()>>,
}

#[cfg(test)]
impl HeightTraceGuard {
    fn start() -> Self {
        assert!(!HEIGHT_TRACES_ARMED.with(|armed| armed.replace(true)));
        let guard = Self {
            _same_thread: std::marker::PhantomData,
        };
        CELL_HEIGHT_STEPS.with(|steps| steps.borrow_mut().clear());
        CHILD_HEIGHT_STEPS.with(|steps| steps.borrow_mut().clear());
        HEIGHT_AXIS_INPUTS.with(|inputs| inputs.borrow_mut().clear());
        guard
    }
}

#[cfg(test)]
impl Drop for HeightTraceGuard {
    fn drop(&mut self) {
        let _ = HEIGHT_TRACES_ARMED.try_with(|armed| armed.set(false));
        let _ = CELL_HEIGHT_STEPS.try_with(|steps| {
            if let Ok(mut steps) = steps.try_borrow_mut() {
                *steps = Vec::new();
            }
        });
        let _ = CHILD_HEIGHT_STEPS.try_with(|steps| {
            if let Ok(mut steps) = steps.try_borrow_mut() {
                *steps = Vec::new();
            }
        });
        let _ = HEIGHT_AXIS_INPUTS.try_with(|inputs| {
            if let Ok(mut inputs) = inputs.try_borrow_mut() {
                *inputs = Vec::new();
            }
        });
    }
}

#[cfg(test)]
fn cell_height_step(step: &'static str) {
    if HEIGHT_TRACES_ARMED.with(std::cell::Cell::get) {
        CELL_HEIGHT_STEPS.with(|steps| steps.borrow_mut().push(step));
    }
}

#[cfg(test)]
pub(super) fn child_height_step(step: &'static str, index: usize, operands: [f64; 3]) {
    if HEIGHT_TRACES_ARMED.with(std::cell::Cell::get) {
        CHILD_HEIGHT_STEPS.with(|steps| steps.borrow_mut().push((step, index, operands)));
    }
}

/// Root-local measured consequences. Source data, labels, typography and guide
/// semantics are immutable under this owner; coordinated inputs below are not.
#[derive(Default)]
pub(crate) struct HeightMeasurements {
    records: Vec<Option<HeightMeasurement>>,
}

#[derive(Debug, Clone, PartialEq)]
struct HeightInputs {
    plot_id: PlotOccurrenceId,
    text_layouts: crate::text::roles::TextRoles,
    output_dimensions: Option<RootSize>,
    prose: [crate::Text; 3],
    axis_titles: [crate::Text; 2],
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
        if self.plot_id != other.plot_id
            || self.width != other.width
            || self.prose != other.prose
            || self.text_layouts != other.text_layouts
            || self.output_dimensions != other.output_dimensions
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
    #[expect(
        clippy::too_many_arguments,
        reason = "capture the selected scientific owner and its complete consumed assignment inputs"
    )]
    fn new(
        registered: &PlotRegistration<'_>,
        canvas: Option<&Canvas>,
        output_dimensions: Option<RootSize>,
        ranges: [(f64, f64); 2],
        provenance: [crate::layout::RangeProvenance; 2],
        resolved_x: Option<(f64, f64)>,
        width: f64,
        parent_insets: [i64; 4],
        round: &FeedbackRound,
        owner: &ComponentAddress,
        per_plot: bool,
        dependencies: &[ConsumedGuideDependency],
    ) -> Result<Self, Error> {
        registered.axes.coordinate().ok_or_else(|| {
            Error::Render("height measurement requires prepared cell axes".into())
        })?;
        let ranges = ranges.map(|range| [range.0.to_bits(), range.1.to_bits()]);
        let source = registered.source;
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

        let use_kind = if crate::orchestrator::registration::legend_policy(source, Some(registered))
            == crate::LegendPolicy::Hidden
        {
            GuideUse::Hidden
        } else if per_plot {
            GuideUse::Local
        } else {
            GuideUse::SharedSuppressed
        };
        let local = if use_kind == GuideUse::Local {
            crate::orchestrator::registration::guide_descriptor(Some(registered))?
                .map(|descriptor| {
                    crate::decorations::legend::discrete::GuideSlot::new(
                        descriptor,
                        &child,
                        source.legend_position_spec(),
                    )
                })
                .transpose()?
        } else {
            None
        };
        let inputs = Self {
            plot_id: registered.scope.id.clone(),
            prose: source.copy_prose()?,
            text_layouts: source.text_layouts().clone(),
            output_dimensions,
            axis_titles: source.copy_axis_titles()?,
            width: crate::layout::kernel::nearest_spatial_quanta(width)?,
            ranges,
            provenance,
            generation: [
                source.x_axis_spec().generation.clone(),
                source.y_axis_spec().generation.clone(),
            ],
            category_labels: [
                source.x_axis_spec().category_labels.clone(),
                source.y_axis_spec().category_labels.clone(),
            ],
            resolved_x: resolved_x.map(|range| [range.0.to_bits(), range.1.to_bits()]),
            own_insets,
            parent_insets,
            y_labels: registered.policy.axis_text.tick_labels[1],
            legend: crate::orchestrator::registration::legend_policy(source, Some(registered)),
            continuous_hidden: crate::orchestrator::registration::continuous_guide_suppressed(
                Some(registered),
            ),
            numeric: crate::numeric::inputs::NumericInputs::new(registered, canvas)?,
            alpha: source.alpha_inputs().clone(),
            guides: ScopedGuideInputs::capture_sizing(
                registered,
                GuideConsumption {
                    use_kind,
                    local_guide: local.as_ref(),
                    dependencies,
                },
            )?,
            scope: registered
                .scope
                .render_name
                .as_deref()
                .map(crate::data::source::copy_source_string)
                .transpose()?,
        };
        #[cfg(test)]
        if HEIGHT_TRACES_ARMED.with(std::cell::Cell::get) {
            HEIGHT_AXIS_INPUTS.with(|snapshots| {
                snapshots.borrow_mut().push((
                    inputs.output_dimensions,
                    inputs.ranges,
                    inputs.resolved_x,
                ));
            });
        }
        Ok(inputs)
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
        let Some(Some(entry)) = self.records.get(index) else {
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
        &mut self,
        registered: &PlotRegistration<'_>,
        canvas: Option<&Canvas>,
        output_dimensions: Option<RootSize>,
        ranges: [(f64, f64); 2],
        provenance: [crate::layout::RangeProvenance; 2],
        resolved_x: Option<(f64, f64)>,
        width: f64,
        height: f64,
        parent_insets: [i64; 4],
        round: &FeedbackRound,
        owner: &ComponentAddress,
        dependencies: &[ConsumedGuideDependency],
    ) -> Result<(), Error> {
        let inputs = HeightInputs::new(
            registered,
            canvas,
            output_dimensions,
            ranges,
            provenance,
            resolved_x,
            width,
            parent_insets,
            round,
            owner,
            true,
            dependencies,
        )?;
        let index = usize::try_from(owner.leaf_slot_index())
            .map_err(|_| Error::Render("content height slot overflowed".into()))?;
        let entries = &mut self.records;
        let needed = index
            .checked_add(1)
            .ok_or_else(|| Error::Render("content height slot count overflowed".into()))?;
        if entries.len() < needed {
            let additional = needed - entries.len();
            fit::reserve(entries, additional, "content height measurement inputs")?;
            entries.resize_with(needed, || None);
        }

        entries[index] = Some(HeightMeasurement {
            inputs,
            height: crate::layout::kernel::required_length(height)?,
        });
        #[cfg(test)]
        child_height_step("record", index, [width, height, 0.0]);
        Ok(())
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "successful child height reads the same occurrence's intrinsic slot before recording the original height inputs"
)]
pub(super) fn measured_preference(
    registered: &PlotRegistration<'_>,
    intrinsic: &crate::orchestrator::root::PlotIntrinsics,
    operation: &crate::font::canonical::OperationContext,
    plan: &crate::decorations::plan::MeasuredPlotPlan,
    local_text_height: f64,
    round: &FeedbackRound,
    owner: &ComponentAddress,
    theme: &crate::text::TextRequestContext<'_, '_>,
    ranges: [(f64, f64); 2],
) -> Result<f64, Error> {
    let source = registered.source;
    let hard = crate::layout::interior::intrinsic_body_minimum_for_ranges(
        registered,
        intrinsic,
        operation,
        theme,
        Some(ranges),
    )?;
    let preference = body_preference(source, Some(registered), intrinsic, operation, theme, hard)?;
    #[cfg(test)]
    child_height_step(
        "measured-preference",
        usize::try_from(owner.leaf_slot_index()).expect("Canvas slot fits usize"),
        [plan.panel_box.width(), local_text_height, 0.0],
    );
    let mut height = source.natural_body_height(
        Some(registered),
        intrinsic,
        operation,
        theme,
        plan.panel_box.width(),
        preference.1,
        Some(ranges),
    )?;
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
    clippy::too_many_lines,
    clippy::type_complexity,
    reason = "keep the ordered hard/preference/prose/key/lookup program and its three original narrow axis operands together"
)]
pub(super) fn cell_height(
    registered: &PlotRegistration<'_>,
    operation: &crate::font::canonical::OperationContext,
    canvas: &Canvas,
    theme: &crate::text::TextRequestContext<'_, '_>,
    width: f64,
    intrinsic: &crate::orchestrator::root::PlotIntrinsics,
    output_dimensions: Option<RootSize>,
    axes: (
        [(f64, f64); 2],
        [crate::layout::RangeProvenance; 2],
        Option<(f64, f64)>,
    ),
    per_plot: bool,
    round: &FeedbackRound,
    owner: &ComponentAddress,
    parent_insets: [i64; 4],
    measurements: Option<&HeightMeasurements>,
    dependencies: &[ConsumedGuideDependency],
) -> Result<f64, Error> {
    let source = registered.source;
    #[cfg(test)]
    cell_height_step("hard");
    let hard = crate::layout::interior::intrinsic_body_minimum_for_ranges(
        registered,
        intrinsic,
        operation,
        theme,
        Some(axes.0),
    )?;
    #[cfg(test)]
    cell_height_step("preference");
    let preference = body_preference(source, Some(registered), intrinsic, operation, theme, hard)?;
    let sources = registered.source.prose();
    let present = sources.iter().any(Option::is_some);
    #[cfg(test)]
    cell_height_step("prose");
    let prose = RootTitleBlockPlan::content_reserves(
        theme,
        sources,
        if present {
            theme.root_inner_width(width)?
        } else {
            width
        },
        RootAlignments::resolve(theme, registered.source.prose_alignments(), [None; 2]),
        &FitScope::root(),
    )
    .map_err(fit::PlanError::into_error)?;
    #[cfg(test)]
    cell_height_step("key");
    let identity = HeightInputs::new(
        registered,
        Some(canvas),
        output_dimensions,
        axes.0,
        axes.1,
        axes.2,
        width,
        parent_insets,
        round,
        owner,
        per_plot,
        dependencies,
    )?;
    if let Some(measurements) = measurements {
        #[cfg(test)]
        cell_height_step("lookup");
        if let Some(height) = measurements.measured(&identity, owner, round)? {
            #[cfg(test)]
            cell_height_step("hit");
            return Ok(height);
        }
    }
    #[cfg(test)]
    cell_height_step("computed");
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
        crate::layout::interior::sizing::guided_preference(source, Some(registered))?
    } else {
        None
    };
    let height = if guide.is_some() {
        crate::layout::interior::sizing::guided_body_height(
            source,
            Some(registered),
            intrinsic,
            operation,
            theme,
            body_width,
            guide.as_ref(),
            &FitScope::new(owner, 0),
            Some(axes.0),
        )?
    } else {
        source.natural_body_height(
            Some(registered),
            intrinsic,
            operation,
            theme,
            body_width,
            preference.1,
            Some(axes.0),
        )?
    };

    let height = prose.height_with_body(height + local[2] + local[3])?;
    if present {
        theme.root_outer_height(&height)
    } else {
        crate::theme::composition::hard(&height)
    }
}

/// Original working windows, including the raw Forest window that a resolved
/// x override can mask. These are sizing/key operands, not assigned domains.
#[expect(
    clippy::type_complexity,
    reason = "the original three narrow axis operands need no new retained input type"
)]
pub(super) fn axis_inputs(
    root: &Root<'_>,
    registered: &PlotRegistration<'_>,
    round: &FeedbackRound,
    initial_sizing: bool,
) -> (
    [(f64, f64); 2],
    [crate::layout::RangeProvenance; 2],
    Option<(f64, f64)>,
) {
    use crate::layout::RangeProvenance::{Automatic, Configured, Shared};
    let source = registered.source;
    let authored = [source.x_axis_spec(), source.y_axis_spec()];
    let base = registered.axes.ranges();
    let mut ranges = std::array::from_fn(|side| {
        authored[side]
            .window
            .map_or(base[side], crate::layout::RawWindow::pair)
    });
    let mut provenance = authored.map(|axis| {
        if axis.window.is_some() {
            Configured
        } else {
            Automatic
        }
    });
    let mut resolved_x = None;
    let mut grouped = false;
    for group in &root.canvas_topology().range_groups {
        if !group.contains(&registered.scope.id) {
            continue;
        }
        let mut shared: Option<(f64, f64)> = None;
        for id in group {
            let member = &root.plots[id.slot];
            let base = member.axes.ranges()[0];
            let range = round
                .feedback_domain(&member.scope.address, FeedbackKind::AutomaticDomainX)
                .map_or(base, |range| (base.0.min(range.0), base.1.max(range.1)));
            shared =
                Some(shared.map_or(range, |prior| (prior.0.min(range.0), prior.1.max(range.1))));
        }
        let shared = shared.expect("validated nonempty shared x group");
        if source.kind() == crate::PlotKind::Forest
            && source.axis_scale_spec() == crate::AxisScale::Log10
        {
            if initial_sizing {
                ranges[0] = base[0];
            }
            resolved_x = Some(shared);
        } else {
            ranges[0] = shared;
        }
        if authored[0].window.is_none() {
            provenance[0] = Shared;
        }
        grouped = true;
    }
    if !grouped {
        if let Some(range) =
            round.feedback_domain(&registered.scope.address, FeedbackKind::AutomaticDomainX)
        {
            ranges[0] = range;
        }
    }
    if let Some(range) =
        round.feedback_domain(&registered.scope.address, FeedbackKind::AutomaticDomainY)
    {
        ranges[1] = range;
    }
    (ranges, provenance, resolved_x)
}

pub(super) fn grid_gutters(
    source: &Canvas,
    root: &Root<'_>,
    round: &FeedbackRound,
) -> Result<Vec<(usize, usize, EdgeSpaceRequirements)>, Error> {
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
            EdgeSpaceRequirements {
                left,
                right,
                bottom,
                top: top + ink[3],
            },
        ));
    }
    for (index, group) in root.canvas_topology().range_groups.iter().enumerate() {
        let owner = grid_owner.child(
            AddressRole::CanvasRangeGroup,
            u32::try_from(index)
                .map_err(|_| Error::Render("Canvas range group address overflow".into()))?,
        )?;
        let alignment = round.insets(&owner, FeedbackKind::SharedAlignment);
        for id in group {
            let index = usize::try_from(root.plots[id.slot].scope.address.leaf_slot_index())
                .expect("Canvas slot fits usize");
            gutters[index].2.left = gutters[index].2.left.max(alignment[0]);
            gutters[index].2.right = gutters[index].2.right.max(alignment[1]);
        }
    }
    Ok(gutters)
}

pub(super) fn content_frame(
    source: &Canvas,
    theme: &crate::text::TextRequestContext<'_, '_>,
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
    let height = root_dimension(theme.root_outer_height(&prose.height_with_body(height)?)?)?;
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
    prepared: &Root<'_>,
    requested: crate::layout::RootRequest,
    round: &FeedbackRound,
    measurements: &HeightMeasurements,
) -> Result<RootSize, Error> {
    let requested_width = requested.exact_width();
    if let crate::layout::RootRequest::ExactSize { width, height } = requested {
        return Ok(RootSize::new(width, height));
    }
    let theme = &prepared.request();
    for (index, panel) in source.placements.plots().enumerate() {
        let Some(panel) = panel else { continue };
        let registered = prepared.canvas_plot(index).expect("prepared Canvas plot");
        registered.scope.render_name.as_deref().ok_or_else(|| {
            Error::Render("Canvas content child requires its prepared scope".into())
        })?;
        crate::layout::interior::InteriorRoute::classify(registered.source.kind())
            .validate(panel)?;
    }
    source.coordinate_domains(prepared, round)?;
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
            EdgeSpaceRequirements {
                left,
                right,
                bottom,
                top: top + ink[3],
            },
        ));
    }
    for (index, group) in prepared.canvas_topology().range_groups.iter().enumerate() {
        let owner = grid_owner.child(
            AddressRole::CanvasRangeGroup,
            u32::try_from(index)
                .map_err(|_| Error::Render("Canvas range group address overflow".into()))?,
        )?;
        let alignment = round.insets(&owner, FeedbackKind::SharedAlignment);
        for id in group {
            let index = usize::try_from(prepared.plots[id.slot].scope.address.leaf_slot_index())
                .expect("Canvas slot fits usize");
            gutters[index].2.left = gutters[index].2.left.max(alignment[0]);
            gutters[index].2.right = gutters[index].2.right.max(alignment[1]);
        }
    }
    let mut guide = (0.0, 0.0);
    if let Some(slot) = &prepared.canvas_topology().shared_guide {
        guide = crate::decorations::legend::Legend::natural_preference(&slot.descriptor, theme)?;
    }
    let guide_gap = if guide.0 > 0.0 {
        theme.legend_shared_gap()?
    } else {
        0.0
    };
    let allocate = |width: Option<f64>, gutters: &[(usize, usize, EdgeSpaceRequirements)]| {
        fit::measured(super::tracks::allocate(
            PixelRect::new(0.0, width.unwrap_or(0.0), 0.0, 0.0),
            &source.placements,
            prepared,
            Some(round),
            &source.column_headers,
            &source.row_headers,
            &source.row_titles,
            gutters,
            theme,
            &FitScope::root(),
            Some(&super::tracks::ContentSizing {
                source,
                root: prepared,
                output_dimensions: None,
                initial_sizing: true,
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
        let width = fit::measured(super::tracks::natural_width(
            &source.placements,
            &source.column_headers,
            &source.row_headers,
            &source.row_titles,
            theme,
            &FitScope::root(),
            &super::tracks::ContentSizing {
                source,
                root: prepared,
                output_dimensions: None,
                initial_sizing: true,
                round,
                measurements: Some(measurements),
                shared_allocation: None,
                natural_width: true,
                natural_height: true,
            },
        ))
        .map_err(fit::PlanError::into_error)?;
        theme.root_outer_width(width + guide.0 + guide_gap)?
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
    let width = requested.select_width(preferred_width)?;
    let grid = allocate(Some(theme.root_inner_width(f64::from(width))?), &gutters)?;
    let prose = RootTitleBlockPlan::content_reserves(
        theme,
        sources,
        theme.root_inner_width(f64::from(width))?,
        RootAlignments::resolve(theme, source.prose_alignments(), [None; 2]),
        &FitScope::root(),
    )
    .map_err(fit::PlanError::into_error)?;
    let height = root_dimension(
        theme.root_outer_height(&prose.height_with_body(grid.size.1.max(guide.1))?)?,
    )?;
    Ok(RootSize::new(width, height))
}
