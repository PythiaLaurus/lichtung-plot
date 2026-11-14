//! Latest successful outcomes of immutable Canvas/facet child owners, not a
//! general plot cache. One instance belongs to one prepared root operation.

use super::{LegendRole, MeasuredPlotPlan};
use crate::decorations::legend::discrete::scoped::{GuideConsumption, GuideUse, ScopedGuideInputs};
use crate::layout::fit::FitOutcome;
use crate::layout::kernel::{ChildFeedback, ChildFeedbackInputs, FeedbackRound};
use crate::layout::{AddressRole, ComponentAddress, PixelRect};
use crate::orchestrator::assignment::{PlotMeasureAssignment, PlotMeasureAssignmentFactory};
use crate::spec::{FacetScales, PlotMeasureContext, PlotRenderResult};
use crate::Error;
use std::sync::Arc;

#[cfg(test)]
use crate::orchestrator::measurements::{child_cache, Counter};
#[cfg(test)]
use crate::Plot;

#[cfg(test)]
thread_local! {
    pub(crate) static DISABLE_CHILD_REUSE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    pub(crate) static CHILD_MEASUREMENTS: std::cell::RefCell<Vec<(ComponentAddress, &'static str)>> = const { std::cell::RefCell::new(Vec::new()) };
    static CHILD_AXIS_INPUTS_ARMED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    #[expect(clippy::type_complexity, reason = "test-only observation of the retained working windows and resolved Forest override")]
    pub(crate) static CHILD_AXIS_INPUTS: std::cell::RefCell<Vec<(ComponentAddress, [Option<(f64, f64)>; 2], Option<(f64, f64)>)>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// A Forest fixture's raw-window observations must not accumulate in other
/// tests, or be cleared from a thread other than the one that armed them.
#[cfg(test)]
#[must_use]
pub(crate) struct ChildAxisInputsTraceGuard {
    _same_thread: std::marker::PhantomData<std::rc::Rc<()>>,
}

#[cfg(test)]
impl ChildAxisInputsTraceGuard {
    pub(crate) fn start() -> Self {
        assert!(!CHILD_AXIS_INPUTS_ARMED.with(|armed| armed.replace(true)));
        let guard = Self {
            _same_thread: std::marker::PhantomData,
        };
        CHILD_AXIS_INPUTS.with(|inputs| inputs.borrow_mut().clear());
        guard
    }
}

#[cfg(test)]
impl Drop for ChildAxisInputsTraceGuard {
    fn drop(&mut self) {
        let _ = CHILD_AXIS_INPUTS_ARMED.try_with(|armed| armed.set(false));
        let _ = CHILD_AXIS_INPUTS.try_with(|inputs| {
            if let Ok(mut inputs) = inputs.try_borrow_mut() {
                *inputs = Vec::new();
            }
        });
    }
}

struct PlotEntryInputs {
    plot_id: crate::orchestrator::registration::PlotOccurrenceId,
    owners: [ComponentAddress; 2],
    geometry: [u64; 4],
    axis_label_width: u64,
    content_height: bool,
    fill_content_height: bool,
    plot: crate::builder::ComposedInputs,
    theme: crate::theme::ResolvedTheme,
    #[expect(
        clippy::type_complexity,
        reason = "the original direction comparison retains only its value projection"
    )]
    direction: Option<(
        [(f64, f64); 2],
        PixelRect,
        [Option<Vec<(f64, f64)>>; 2],
        Option<String>,
    )>,
    scales: Option<FacetScales>,
    feedback: ChildFeedbackInputs,
}

impl PlotEntryInputs {
    fn new(
        context: &PlotMeasureContext<'_, '_>,
        owner: &ComponentAddress,
        round: &FeedbackRound,
        measurement_phase: Option<MeasurementPhase>,
    ) -> Result<Self, Error> {
        let request = context.request();
        let theme = request.theme.clone();
        let direction = request.reference.map(|reference| reference.snapshot());
        let grid_role = if owner.leaf_role() == AddressRole::CanvasCell {
            AddressRole::CanvasGrid
        } else {
            AddressRole::FacetGrid
        };
        let grid_owner = owner.under_grid(grid_role)?;
        let owners = [owner.clone(), grid_owner];
        let inputs = Self {
            plot_id: context.registered.scope.id.clone(),
            geometry: geometry(context.assignment.local.body),
            axis_label_width: context.assignment.local.axis_label_width.to_bits(),
            content_height: context.assignment.local.content_height,
            fill_content_height: context.assignment.local.fill_content_height,
            plot: crate::builder::ComposedInputs::retain(
                context,
                round,
                ScopedGuideInputs::capture(context)?,
                measurement_phase,
            )?,
            theme,
            direction,
            scales: context.registered.policy.facet_scales,
            feedback: round.child_inputs(&owners)?,
            owners,
        };
        #[cfg(test)]
        if CHILD_AXIS_INPUTS_ARMED.with(std::cell::Cell::get) {
            CHILD_AXIS_INPUTS.with(|snapshots| {
                let (windows, _, resolved_x) = inputs.plot.axis_snapshot_for_test();
                snapshots
                    .borrow_mut()
                    .push((owner.clone(), windows, resolved_x));
            });
        }
        Ok(inputs)
    }

    fn matches(
        &self,
        context: &PlotMeasureContext<'_, '_>,
        owner: &ComponentAddress,
        round: &FeedbackRound,
        measurement_phase: Option<MeasurementPhase>,
    ) -> Result<bool, Error> {
        Ok(self.owners[0] == *owner
            && self.plot_id == context.registered.scope.id
            && self.geometry == geometry(context.assignment.local.body)
            && self.axis_label_width == context.assignment.local.axis_label_width.to_bits()
            && self.content_height == context.assignment.local.content_height
            && self.fill_content_height == context.assignment.local.fill_content_height
            && self.plot.guides.matches_consumption(
                measurement_guides(context).use_kind,
                context.guides.local_guide,
                context.guides.dependencies,
            )
            && self
                .plot
                .matches(context, round, &self.plot.guides, measurement_phase)
            && self.plot.guides.stable(round)
            && self.theme.same_validated(
                context.request().theme,
                &self.direction,
                context.request().reference,
            )?
            && self.scales == context.registered.policy.facet_scales
            && round.child_inputs_equal(&self.feedback, &self.owners)
            && round.child_stable(&self.owners))
    }
}

fn measurement_guides<'v>(context: &PlotMeasureContext<'v, '_>) -> GuideConsumption<'v> {
    GuideConsumption {
        use_kind: if crate::orchestrator::registration::legend_policy(
            context.registered.source,
            Some(context.registered),
        ) == crate::LegendPolicy::Hidden
        {
            GuideUse::Hidden
        } else {
            context.guides.use_kind
        },
        ..context.guides
    }
}

fn geometry(panel: PixelRect) -> [u64; 4] {
    [panel.x0, panel.x1, panel.y0, panel.y1].map(f64::to_bits)
}

struct Measurement {
    inputs: PlotEntryInputs,
    phase: MeasurementPhase,
    result: Arc<PlotRenderResult>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum MeasurementPhase {
    Probe,
    Hue,
    Legend,
    Category,
}

#[cfg(test)]
impl MeasurementPhase {
    fn label(self) -> &'static str {
        match self {
            Self::Probe => "probe",
            Self::Hue => "hue-probe",
            Self::Legend => "legend-probe",
            Self::Category => "category-probe",
        }
    }
}

struct CompleteChildCacheEntry {
    inputs: PlotEntryInputs,
    role: LegendRole,
    result: Arc<MeasuredPlotPlan>,
    feedback: ChildFeedback,
}

pub(crate) struct ChildPlans {
    source: AddressRole,
    measurements: Vec<Measurement>,
    panels: Vec<CompleteChildCacheEntry>,
}

impl ChildPlans {
    pub(crate) fn canvas() -> Self {
        Self {
            source: AddressRole::CanvasCell,
            measurements: Vec::new(),
            panels: Vec::new(),
        }
    }

    pub(crate) fn facets() -> Self {
        Self {
            source: AddressRole::FacetCell,
            measurements: Vec::new(),
            panels: Vec::new(),
        }
    }

    fn validate_owner(&self, owner: &ComponentAddress) -> Result<(), Error> {
        if owner.leaf_role() != self.source {
            return Err(Error::Render(
                "child result belongs to a different root topology".into(),
            ));
        }
        Ok(())
    }

    /// A provisional result must not become an active entry for the next round.
    pub(crate) fn finish_round(&mut self, round: &FeedbackRound) {
        if round.child_stable(&[ComponentAddress::root()]) {
            self.measurements.clear();
            self.panels.clear();
            return;
        }
        self.measurements.retain(|entry| {
            round.child_stable(&entry.inputs.owners) && entry.inputs.plot.guides.stable(round)
        });
        self.panels.retain(|entry| {
            round.child_stable(&entry.inputs.owners) && entry.inputs.plot.guides.stable(round)
        });
    }

    pub(crate) fn measure(
        &mut self,
        context: &PlotMeasureContext<'_, '_>,
        owner: &ComponentAddress,
        phase: MeasurementPhase,
        round: &FeedbackRound,
        measure: impl FnOnce() -> Result<FitOutcome<PlotRenderResult>, Error>,
    ) -> Result<FitOutcome<Arc<PlotRenderResult>>, Error> {
        self.validate_owner(owner)?;
        if reuse_enabled() {
            for entry in &self.measurements {
                if entry.phase == phase
                    && entry.inputs.matches(context, owner, round, Some(phase))?
                {
                    #[cfg(test)]
                    child_cache(owner, phase.label(), Counter::ChildCacheHit);
                    return Ok(FitOutcome::Measured(Arc::clone(&entry.result)));
                }
            }
        }
        #[cfg(test)]
        child_cache(owner, phase.label(), Counter::ChildCacheMiss);
        #[cfg(test)]
        CHILD_MEASUREMENTS.with(|events| events.borrow_mut().push((owner.clone(), phase.label())));
        let result = match measure()? {
            FitOutcome::Measured(result) => crate::data::source::source_arc(result)?,
            FitOutcome::Miss(miss) => return Ok(FitOutcome::Miss(miss)),
        };
        if !reuse_enabled() {
            return Ok(FitOutcome::Measured(result));
        }
        self.measurements
            .retain(|entry| entry.inputs.owners[0] != *owner || entry.phase != phase);
        crate::render::backend::reserve_text_records(&mut self.measurements, 1)?;
        self.measurements.push(Measurement {
            inputs: PlotEntryInputs::new(context, owner, round, Some(phase))?,
            phase,
            result: Arc::clone(&result),
        });
        Ok(FitOutcome::Measured(result))
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "the original child cache receives the unique factory, selected entry and unchanged guide/feedback operands"
    )]
    pub(crate) fn plan(
        &mut self,
        factory: &PlotMeasureAssignmentFactory<'_, '_>,
        entry: PlotMeasureAssignment,
        role: LegendRole,
        guides: GuideConsumption<'_>,
        fit_scope: &crate::layout::fit::FitScope,
        round: &mut FeedbackRound,
        owner: &ComponentAddress,
    ) -> Result<FitOutcome<Arc<MeasuredPlotPlan>>, Error> {
        self.validate_owner(owner)?;
        let use_kind = if crate::orchestrator::registration::legend_policy(
            factory.registered.source,
            Some(factory.registered),
        ) == crate::LegendPolicy::Hidden
        {
            GuideUse::Hidden
        } else if role == LegendRole::Panel {
            GuideUse::Local
        } else {
            GuideUse::SharedSuppressed
        };
        let guides = GuideConsumption { use_kind, ..guides };
        let retained = entry.clone();
        let context = factory.context(&retained, guides, fit_scope)?;
        if reuse_enabled() {
            for cached in &self.panels {
                if cached.role == role && cached.inputs.matches(&context, owner, round, None)? {
                    #[cfg(test)]
                    child_cache(owner, "panel", Counter::ChildCacheHit);
                    round.replay_child(&cached.feedback)?;
                    #[cfg(test)]
                    child_cache(owner, "panel", Counter::ChildFeedbackReplay);
                    return Ok(FitOutcome::Measured(Arc::clone(&cached.result)));
                }
            }
        }
        #[cfg(test)]
        child_cache(owner, "panel", Counter::ChildCacheMiss);
        #[cfg(test)]
        CHILD_MEASUREMENTS.with(|events| events.borrow_mut().push((owner.clone(), "panel")));
        let checkpoint = round.child_checkpoint();
        let result = match super::plan_plot(factory, entry, role, guides, fit_scope, round)? {
            FitOutcome::Measured(result) => result,
            FitOutcome::Miss(miss) => return Ok(FitOutcome::Miss(miss)),
        };
        if !reuse_enabled() {
            return Ok(FitOutcome::Measured(result));
        }
        self.panels
            .retain(|cached| cached.inputs.owners[0] != *owner);
        if let Some(feedback) = round.retain_child_feedback(checkpoint)? {
            let inputs = PlotEntryInputs::new(&context, owner, round, None)?;
            if round.child_stable(&inputs.owners) {
                crate::render::backend::reserve_text_records(&mut self.panels, 1)?;
                self.panels.push(CompleteChildCacheEntry {
                    inputs,
                    role,
                    result: Arc::clone(&result),
                    feedback,
                });
            }
        }
        Ok(FitOutcome::Measured(result))
    }
}

fn reuse_enabled() -> bool {
    #[cfg(test)]
    if DISABLE_CHILD_REUSE.with(std::cell::Cell::get) {
        return false;
    }
    true
}

#[cfg(test)]
mod scoped_tests;

#[cfg(test)]
fn prepare_child_inputs_for_test<'src>(
    source: &'src Plot,
    stage: &mut crate::layout::kernel::PreparedStage<'_>,
) -> Result<crate::orchestrator::root::Root<'src>, Error> {
    crate::orchestrator::root::Root::prepare(
        crate::orchestrator::root::RootSource::Plot(source.description()),
        stage,
    )
}

#[cfg(test)]
fn child_allocation_for_test(
    size: crate::layout::RootSize,
    panel: PixelRect,
    owner: &ComponentAddress,
    round: &FeedbackRound,
) -> crate::orchestrator::assignment::PlotAllocationInputs {
    use crate::orchestrator::assignment::{
        GuideTrialAssignment, ParentInsets, PlotAllocationInputs, PlotMeasureDimensions,
    };
    PlotAllocationInputs {
        dimensions: PlotMeasureDimensions {
            canvas: size,
            output_override: None,
        },
        cell: panel,
        frame: panel,
        body: panel,
        axis_label_width: panel.width(),
        content_height: false,
        fill_content_height: false,
        content_preference: None,
        insets: ParentInsets::consumed(round, owner),
        guide: GuideTrialAssignment {
            edge: crate::LegendPosition::Auto,
            compact: false,
            allocation: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decorations::legend::discrete::scoped::SUPPRESSED_GUIDES;
    use crate::layout::fit::FitScope;

    fn source() -> Plot<crate::kind::Scatter> {
        Plot::scatter(
            crate::NumericValues::new("x", [0.0, 1.0]),
            crate::NumericValues::new("y", [0.0, 1.0]),
        )
    }

    #[test]
    fn content_height_and_row_fill_each_invalidate_the_actual_child_input_key() {
        let source = source().erase();
        let frozen = crate::layout::plan_root(
            crate::layout::RootRequest::ExactSize {
                width: 800,
                height: 600,
            },
            |stage| prepare_child_inputs_for_test(&source, stage),
            |root, assigned, round| {
                let measurement = root.assignment_factory().assign(assigned.assigned_size()?);
                let factory =
                    root.plot_assignment_factory(&measurement, &root.plots[0].scope.id)?;
                let owner = ComponentAddress::root().child(AddressRole::CanvasCell, 0)?;
                let entry = factory.assign(
                    child_allocation_for_test(
                        measurement.measurement_size,
                        PixelRect::new(20.0, 400.0, 20.0, 500.0),
                        &owner,
                        round,
                    ),
                    round,
                    None,
                )?;
                let scope = FitScope::root();
                let context = factory.context(&entry, SUPPRESSED_GUIDES, &scope)?;
                let key = PlotEntryInputs::new(&context, &owner, round, None)?;
                assert!(key.matches(&context, &owner, round, None)?);
                for (content_height, fill_content_height) in
                    [(true, false), (false, true), (true, true)]
                {
                    let mut changed = entry.clone();
                    changed.content_height = content_height;
                    changed.fill_content_height = fill_content_height;
                    let context = factory.context(&changed, SUPPRESSED_GUIDES, &scope)?;
                    assert!(!key.matches(&context, &owner, round, None)?);
                }
                Ok(crate::layout::MeasuredLayout::new(
                    measurement.measurement_size,
                    (),
                ))
            },
            |_, _, (), _| Ok(()),
        )
        .unwrap();
        assert_eq!(frozen.dimensions(), (800, 600));
    }

    #[test]
    fn actual_child_hit_is_invalidated_by_legend_only_paint_and_geometry_changes() {
        let source = source().erase();
        let owner = ComponentAddress::root()
            .child(AddressRole::FacetCell, 0)
            .unwrap();
        let mut children = ChildPlans::facets();
        let mut produced = 0;
        crate::layout::plan_root(
            crate::layout::RootRequest::ExactSize {
                width: 800,
                height: 600,
            },
            |stage| {
                let root = prepare_child_inputs_for_test(&source, stage)?;
                let mut paint = prepare_child_inputs_for_test(&source, stage)?;
                let mut geometry = prepare_child_inputs_for_test(&source, stage)?;
                let appearance = &mut paint.plots[0].appearance;
                let frame = appearance
                    .legend
                    .get_frame()
                    .unwrap()
                    .clone()
                    .fill(crate::theme::Color::rgba(1, 2, 3, 120))?;
                appearance.legend = appearance.legend.clone().frame(Some(frame))?;
                geometry.plots[0].appearance = appearance.clone();
                let appearance = &mut geometry.plots[0].appearance;
                let frame = appearance.legend.get_frame().unwrap();
                let changed = crate::theme::LegendFrameTheme::new(
                    frame.get_fill(),
                    Some(crate::theme::OutlineTheme::new(
                        frame.get_outline().unwrap().get_color(),
                        7.0,
                    )?),
                    frame.get_radius().clone(),
                )?;
                appearance.legend = appearance.legend.clone().frame(Some(changed))?;
                Ok([root, paint, geometry])
            },
            |roots, assigned, round| {
                let mut first = None;
                for root in roots {
                    let measurement = root.assignment_factory().assign(assigned.assigned_size()?);
                    let factory =
                        root.plot_assignment_factory(&measurement, &root.plots[0].scope.id)?;
                    let entry = factory.assign(
                        child_allocation_for_test(
                            measurement.measurement_size,
                            PixelRect::new(10.0, 200.0, 30.0, 400.0),
                            &owner,
                            round,
                        ),
                        round,
                        None,
                    )?;
                    let scope = FitScope::root();
                    let context = factory.context(&entry, SUPPRESSED_GUIDES, &scope)?;
                    let mut produce = || {
                        produced += 1;
                        Ok(FitOutcome::Measured(PlotRenderResult::new(
                            crate::layout::Layout::envelope(entry.body),
                            None,
                        )))
                    };
                    let changed = crate::layout::fit::measured(children.measure(
                        &context,
                        &owner,
                        MeasurementPhase::Legend,
                        round,
                        &mut produce,
                    ))
                    .map_err(crate::layout::fit::PlanError::into_error)?;
                    let hit = crate::layout::fit::measured(children.measure(
                        &context,
                        &owner,
                        MeasurementPhase::Legend,
                        round,
                        &mut produce,
                    ))
                    .map_err(crate::layout::fit::PlanError::into_error)?;
                    assert!(Arc::ptr_eq(&changed, &hit));
                    if let Some(first) = &first {
                        assert!(!Arc::ptr_eq(first, &changed));
                    } else {
                        first = Some(changed);
                    }
                }
                Ok(crate::layout::MeasuredLayout::new(
                    assigned.assigned_size()?,
                    (),
                ))
            },
            |_, _, (), _| Ok(()),
        )
        .unwrap();
        assert_eq!(produced, 3);
    }

    #[test]
    #[expect(
        clippy::too_many_lines,
        reason = "one actual cache retains the complete original numeric-policy matrix"
    )]
    fn child_measurement_reuses_hits_and_replaces_changed_numeric_policies() {
        use crate::numeric::{
            decimal::{NegativeZero, NumericNotation, TrailingZeros},
            format::{NumericFormat, NumericPrecision},
        };
        let format = NumericFormat::default();
        let make = |format| source().x_tick_format(format);
        let source = make(format.clone()).erase();
        let formats = [
            format
                .clone()
                .notation(NumericNotation::Scientific)
                .unwrap(),
            format
                .clone()
                .precision(NumericPrecision::Significant(2))
                .unwrap(),
            format
                .clone()
                .precision(NumericPrecision::AdaptiveFractional { min: 2, max: 3 })
                .unwrap(),
            format.clone().decimal_scale(1).unwrap(),
            format.clone().prefix("raw ").unwrap(),
            format.clone().suffix(" u").unwrap(),
            format
                .clone()
                .negative_zero(NegativeZero::Normalize)
                .unwrap(),
            format.clone().trailing_zeros(TrailingZeros::Trim).unwrap(),
        ];
        let mut variants = formats.map(|format| make(format).erase()).to_vec();
        for generation in [
            crate::TickGeneration::target(5).unwrap(),
            crate::TickGeneration::maximum(5).unwrap(),
        ] {
            variants.push(
                make(format.clone())
                    .x_tick_generation(generation.clone())
                    .erase(),
            );
            variants.push(make(format.clone()).y_tick_generation(generation).erase());
        }
        for alpha in [
            crate::alpha::AlphaValues::new("confidence", [127; 2]).unwrap(),
            crate::alpha::AlphaValues::linear(
                crate::NumericValues::new("confidence", [0.0, 1.0]),
                (0.0, 1.0),
                (127, 127),
            )
            .unwrap(),
            crate::alpha::AlphaValues::continuous(
                crate::NumericValues::new("confidence", [0.0, 1.0]),
                (0.0, 1.0),
                |_| 127,
            )
            .unwrap(),
        ] {
            variants.push(
                make(format.clone())
                    .with_alpha(
                        crate::alpha::AlphaTarget::Point,
                        crate::alpha::AlphaEncoding::observations(alpha),
                    )
                    .erase(),
            );
        }
        let owner = ComponentAddress::root()
            .child(AddressRole::FacetCell, 0)
            .unwrap();
        let mut children = ChildPlans::facets();
        let mut produced = 0;
        crate::layout::plan_root(
            crate::layout::RootRequest::ExactSize {
                width: 800,
                height: 600,
            },
            |stage| {
                let root = prepare_child_inputs_for_test(&source, stage)?;
                let variants = crate::data::source::collect_source(
                    variants
                        .iter()
                        .map(|source| prepare_child_inputs_for_test(source, stage)),
                )?;
                Ok((root, variants))
            },
            |(root, variants), assigned, round| {
                let measurement = root.assignment_factory().assign(assigned.assigned_size()?);
                let factory =
                    root.plot_assignment_factory(&measurement, &root.plots[0].scope.id)?;
                let mut entry = factory.assign(
                    child_allocation_for_test(
                        measurement.measurement_size,
                        PixelRect::new(10.0, 200.0, 30.0, 400.0),
                        &owner,
                        round,
                    ),
                    round,
                    None,
                )?;
                let scope = FitScope::root();
                let mut produce = || {
                    produced += 1;
                    Ok(FitOutcome::Measured(PlotRenderResult::new(
                        crate::layout::Layout::envelope(PixelRect::new(80.0, 770.0, 60.0, 570.0)),
                        None,
                    )))
                };
                let context = factory.context(&entry, SUPPRESSED_GUIDES, &scope)?;
                let first = crate::layout::fit::measured(children.measure(
                    &context,
                    &owner,
                    MeasurementPhase::Probe,
                    round,
                    &mut produce,
                ))
                .map_err(crate::layout::fit::PlanError::into_error)?;
                let hit = crate::layout::fit::measured(children.measure(
                    &context,
                    &owner,
                    MeasurementPhase::Probe,
                    round,
                    &mut produce,
                ))
                .map_err(crate::layout::fit::PlanError::into_error)?;
                assert!(Arc::ptr_eq(&first, &hit));
                entry.axis_label_width = 1.0;
                let context = factory.context(&entry, SUPPRESSED_GUIDES, &scope)?;
                let changed = crate::layout::fit::measured(children.measure(
                    &context,
                    &owner,
                    MeasurementPhase::Probe,
                    round,
                    &mut produce,
                ))
                .map_err(crate::layout::fit::PlanError::into_error)?;
                assert_eq!(children.measurements.len(), 1);
                assert!(!Arc::ptr_eq(&first, &changed));
                assert!(Arc::ptr_eq(&changed, &children.measurements[0].result));
                for variant in variants {
                    assert_eq!(root.plots[0].scope.id, variant.plots[0].scope.id);
                    let factory = variant
                        .plot_assignment_factory(&measurement, &variant.plots[0].scope.id)?;
                    let context = factory.context(&entry, SUPPRESSED_GUIDES, &scope)?;
                    let changed = crate::layout::fit::measured(children.measure(
                        &context,
                        &owner,
                        MeasurementPhase::Probe,
                        round,
                        &mut produce,
                    ))
                    .map_err(crate::layout::fit::PlanError::into_error)?;
                    let hit = crate::layout::fit::measured(children.measure(
                        &context,
                        &owner,
                        MeasurementPhase::Probe,
                        round,
                        &mut produce,
                    ))
                    .map_err(crate::layout::fit::PlanError::into_error)?;
                    assert!(!Arc::ptr_eq(&first, &changed));
                    assert!(Arc::ptr_eq(&changed, &hit));
                }
                Ok(crate::layout::MeasuredLayout::new(
                    measurement.measurement_size,
                    (),
                ))
            },
            |_, _, (), _| Ok(()),
        )
        .unwrap();
        assert_eq!(produced, 2 + variants.len());
    }

    #[test]
    fn later_panel_cache_replays_feedback_without_remeasurement() {
        let source = source().erase();
        let owner = ComponentAddress::root()
            .child(AddressRole::FacetCell, 0)
            .unwrap();
        let sibling = ComponentAddress::root()
            .child(AddressRole::FacetCell, 1)
            .unwrap();
        let mut children = ChildPlans::facets();
        let mut rounds = 0;
        let mut finalized = false;
        crate::layout::plan_root(
            crate::layout::RootRequest::ExactSize {
                width: 800,
                height: 600,
            },
            |stage| prepare_child_inputs_for_test(&source, stage),
            |root, assigned, round| {
                rounds += 1;
                let measurement = root.assignment_factory().assign(assigned.assigned_size()?);
                let factory =
                    root.plot_assignment_factory(&measurement, &root.plots[0].scope.id)?;
                let entry = factory.assign(
                    child_allocation_for_test(
                        measurement.measurement_size,
                        PixelRect::new(10.0, 200.0, 30.0, 400.0),
                        &owner,
                        round,
                    ),
                    round,
                    None,
                )?;
                let scope = FitScope::root();
                let context = factory.context(&entry, SUPPRESSED_GUIDES, &scope)?;
                if rounds == 1 {
                    let start = round.child_checkpoint();
                    round.observe_number(1.0)?;
                    children.panels.push(CompleteChildCacheEntry {
                        inputs: PlotEntryInputs::new(&context, &owner, round, None)?,
                        role: LegendRole::Suppressed,
                        result: Arc::new(MeasuredPlotPlan {
                            plot_id: root.plots[0].scope.id.clone(),
                            selected: entry.clone(),
                            preference: None,
                            panel_box: entry.body,
                            result: PlotRenderResult::new(
                                crate::layout::Layout::envelope(PixelRect::new(
                                    80.0, 770.0, 60.0, 570.0,
                                )),
                                None,
                            ),
                            legend: None,
                            layers: None,
                            surface: root.plots[0].appearance.surfaces.plot_panel,
                        }),
                        feedback: round.retain_child_feedback(start)?.unwrap(),
                    });
                } else {
                    CHILD_MEASUREMENTS.with(|events| events.borrow_mut().clear());
                    let replayed = crate::layout::fit::measured(children.plan(
                        &factory,
                        entry,
                        LegendRole::Suppressed,
                        SUPPRESSED_GUIDES,
                        &scope,
                        round,
                        &owner,
                    ))
                    .map_err(crate::layout::fit::PlanError::into_error)?;
                    assert!(Arc::ptr_eq(&replayed, &children.panels[0].result));
                    CHILD_MEASUREMENTS.with(|events| assert!(events.borrow().is_empty()));
                }
                round.request_insets(
                    &sibling,
                    &sibling,
                    crate::layout::kernel::FeedbackKind::AxisGutter,
                    [1.0; 4],
                )?;
                Ok(crate::layout::MeasuredLayout::new(
                    measurement.measurement_size,
                    (),
                ))
            },
            |_, _, (), _| {
                finalized = true;
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(rounds, 2);
        assert!(finalized);
    }

    #[test]
    #[expect(
        clippy::too_many_lines,
        reason = "one key matrix preserves computed policy and relevant feedback coverage"
    )]
    fn child_keys_include_exact_origin_computed_policy_and_relevant_feedback() {
        let source_variants = [
            source().x_limits(-1.0, 2.0).erase(),
            source().y_limits(-1.0, 2.0).erase(),
            source().title_alignment(crate::TextAlignment::End).erase(),
            source()
                .x_label(crate::Text::new("x").layout(crate::TextLayout::default()))
                .erase(),
        ];
        let source = source().erase();
        let owner = ComponentAddress::root()
            .child(AddressRole::CanvasCell, 0)
            .unwrap();
        let neighbor = ComponentAddress::root()
            .child(AddressRole::CanvasCell, 1)
            .unwrap();
        crate::layout::plan_root(
            crate::layout::RootRequest::ExactSize {
                width: 800,
                height: 600,
            },
            |stage| {
                let root = prepare_child_inputs_for_test(&source, stage)?;
                let mut variants = Vec::new();
                for index in 0..7 {
                    let mut variant = prepare_child_inputs_for_test(&source, stage)?;
                    let panel = &mut variant.plots[0];
                    match index {
                        0 => {
                            panel.policy.gutter_owner =
                                crate::orchestrator::registration::GutterOwner::Composition;
                        }
                        1 => panel.policy.axis_text.tick_labels[1] = false,
                        2 => {
                            panel.parent_orders.collectives =
                                Some(crate::data::source::source_arc(Vec::new())?);
                        }
                        3 => {
                            panel.parent_orders.hues = Some(crate::data::source::source_arc(
                                crate::data::category::CategoryDomain::from_values(
                                    Some(&crate::data::category::RawField::new("cohort")?),
                                    Vec::new(),
                                )?,
                            )?);
                        }
                        4 => panel.policy.facet_scales = Some(FacetScales::Free),
                        5 => {
                            panel.policy.discrete_guide_use = GuideUse::Hidden;
                            panel.policy.continuous_guide_use = GuideUse::Hidden;
                        }
                        _ => panel.appearance.typography.axes.tick_label.font_size += 1.0,
                    }
                    variants.push(variant);
                }
                for source in &source_variants {
                    variants.push(prepare_child_inputs_for_test(source, stage)?);
                }
                Ok((root, variants))
            },
            |(root, variants), assigned, round| {
                let measurement = root.assignment_factory().assign(assigned.assigned_size()?);
                let factory =
                    root.plot_assignment_factory(&measurement, &root.plots[0].scope.id)?;
                let panel = PixelRect::new(10.0, 200.0, 30.0, 400.0);
                let entry = factory.assign(
                    child_allocation_for_test(measurement.measurement_size, panel, &owner, round),
                    round,
                    None,
                )?;
                let scope = FitScope::root();
                let context = factory.context(&entry, SUPPRESSED_GUIDES, &scope)?;
                assert!(PlotEntryInputs::new(
                    &context,
                    &owner.child(AddressRole::CanvasCell, 1)?,
                    round,
                    None,
                )
                .is_err());
                let key = PlotEntryInputs::new(&context, &owner, round, None)?;
                assert!(key.matches(&context, &owner, round, None)?);
                for variant in variants {
                    let factory = variant
                        .plot_assignment_factory(&measurement, &variant.plots[0].scope.id)?;
                    let context = factory.context(&entry, SUPPRESSED_GUIDES, &scope)?;
                    assert!(!key.matches(&context, &owner, round, None)?);
                }
                let mut changed = entry.clone();
                changed.axis_label_width += 1.0;
                assert!(!key.matches(
                    &factory.context(&changed, SUPPRESSED_GUIDES, &scope)?,
                    &owner,
                    round,
                    None,
                )?);
                for moved in [
                    PixelRect::new(11.0, 201.0, 30.0, 400.0),
                    PixelRect::new(10.0, 200.0, 31.0, 401.0),
                    PixelRect::new(10.0 + 1.0 / 4096.0, 200.0, 30.0, 400.0),
                ] {
                    let mut changed = entry.clone();
                    changed.body = moved;
                    assert!(!key.matches(
                        &factory.context(&changed, SUPPRESSED_GUIDES, &scope)?,
                        &owner,
                        round,
                        None,
                    )?);
                }
                assert!(!key.matches(&context, &neighbor, round, None)?);
                round.request_insets(
                    &neighbor,
                    &neighbor,
                    crate::layout::kernel::FeedbackKind::OwnedInk,
                    [1.0; 4],
                )?;
                assert!(key.matches(&context, &owner, round, None)?);
                for kind in [
                    crate::layout::kernel::FeedbackKind::AxisGutter,
                    crate::layout::kernel::FeedbackKind::SharedAlignment,
                    crate::layout::kernel::FeedbackKind::OwnedInk,
                ] {
                    let checkpoint = round.begin_trial()?;
                    round.request_insets(&owner, &owner, kind, [1.0; 4])?;
                    assert!(!key.matches(&context, &owner, round, None)?);
                    round.miss(
                        &owner,
                        crate::layout::kernel::FitKind::Tick,
                        &[],
                        "discard test request",
                    )?;
                    assert!(matches!(
                        round.finish_trial(checkpoint),
                        crate::layout::kernel::TrialOutcome::Miss(_)
                    ));
                }
                Ok(crate::layout::MeasuredLayout::new(
                    measurement.measurement_size,
                    (),
                ))
            },
            |_, _, (), _| Ok(()),
        )
        .unwrap();

        let canvas = crate::Canvas::grid(1, 1)
            .unwrap()
            .size(800, 600)
            .legend_policy(crate::CanvasLegendPolicy::Shared)
            .panel(
                0,
                0,
                Plot::scatter(
                    crate::NumericValues::new("x", [0.0, 1.0]),
                    crate::NumericValues::new("y", [0.0, 1.0]),
                ),
            )
            .unwrap();
        let mut rounds = 0;
        crate::layout::plan_root(
            canvas.sizing_request(),
            |stage| {
                let root = crate::orchestrator::root::Root::prepare(
                    crate::orchestrator::root::RootSource::Canvas(&canvas),
                    stage,
                )?;
                let mut invalid = crate::orchestrator::root::Root::prepare(
                    crate::orchestrator::root::RootSource::Canvas(&canvas),
                    stage,
                )?;
                invalid.plots[0]
                    .appearance
                    .axes
                    .ticks
                    .as_mut()
                    .expect("publication ticks")
                    .length = 0;
                Ok((root, invalid))
            },
            |(root, invalid), assigned, round| {
                use crate::layout::kernel::FeedbackKind;
                rounds += 1;
                let registered = &root.plots[0];
                let owner = &registered.scope.address;
                let base = registered.axes.ranges();
                assert!(registered.raw_windows().iter().all(Option::is_none));
                let crate::orchestrator::root::PreparedComposition::Canvas(topology) =
                    &root.composition
                else {
                    panic!("Canvas topology")
                };
                assert!(topology.range_groups.is_empty());
                assert!(topology.shared_guide.is_none());

                // A protocol-level publisher makes consumed F exactly base.
                // This does not claim an automatic mark producer emits it.
                for (kind, limits) in [
                    (FeedbackKind::AutomaticDomainX, base[0]),
                    (FeedbackKind::AutomaticDomainY, base[1]),
                ] {
                    round.request_domain(owner, owner, kind, limits)?;
                }
                if rounds > 1 {
                    assert_eq!(
                        round.feedback_domain(owner, FeedbackKind::AutomaticDomainX),
                        Some(base[0])
                    );
                    assert_eq!(
                        round.feedback_domain(owner, FeedbackKind::AutomaticDomainY),
                        Some(base[1])
                    );
                    let measurement = root.assignment_factory().assign(assigned.assigned_size()?);
                    let factory =
                        root.plot_assignment_factory(&measurement, &registered.scope.id)?;
                    let invalid_factory = invalid
                        .plot_assignment_factory(&measurement, &invalid.plots[0].scope.id)?;
                    assert_eq!(registered.scope.id, invalid.plots[0].scope.id);
                    let allocation = |phase| {
                        let mut inputs = child_allocation_for_test(
                            measurement.measurement_size,
                            PixelRect::new(20.0, 700.0, 20.0, 500.0),
                            owner,
                            round,
                        );
                        inputs.dimensions.output_override = Some(measurement.measurement_size);
                        factory.assign(inputs, round, phase)
                    };
                    let initial = allocation(Some(MeasurementPhase::Legend))?;
                    let scope = FitScope::root();
                    let initial_context = factory.context(&initial, SUPPRESSED_GUIDES, &scope)?;
                    let mut children = ChildPlans::canvas();
                    let produced = std::cell::Cell::new(0);
                    for phase in [MeasurementPhase::Legend, MeasurementPhase::Probe] {
                        let entry = allocation(Some(phase))?;
                        assert_eq!(entry.ranges(), initial.ranges());
                        assert_eq!(entry.body, initial.body);
                        let context = factory.context(&entry, SUPPRESSED_GUIDES, &scope)?;
                        let result = crate::layout::fit::measured(children.measure(
                            &context,
                            owner,
                            phase,
                            round,
                            || {
                                produced.set(produced.get() + 1);
                                crate::layout::fit::outcome(
                                    crate::orchestrator::measure_probe(
                                        &factory,
                                        entry.clone(),
                                        context.guides,
                                        context.fit_scope,
                                    )
                                    .map(|(_, result)| result),
                                )
                            },
                        ))
                        .map_err(crate::layout::fit::PlanError::into_error)?;
                        let hit = crate::layout::fit::measured(children.measure(
                            &context,
                            owner,
                            phase,
                            round,
                            || panic!("unchanged child input must hit"),
                        ))
                        .map_err(crate::layout::fit::PlanError::into_error)?;
                        assert!(Arc::ptr_eq(&result, &hit));

                        let inputs = &children
                            .measurements
                            .iter()
                            .find(|entry| entry.phase == phase)
                            .expect("successful phase retention")
                            .inputs;
                        let (windows, provenance, resolved_x) =
                            inputs.plot.axis_snapshot_for_test();
                        assert_eq!(
                            windows,
                            if phase == MeasurementPhase::Legend {
                                [None; 2]
                            } else {
                                base.map(Some)
                            }
                        );
                        assert_eq!(provenance, [crate::layout::RangeProvenance::Automatic; 2]);
                        assert!(resolved_x.is_none());
                        assert!(inputs.matches(&initial_context, owner, round, Some(phase))?);
                        let other = if phase == MeasurementPhase::Legend {
                            MeasurementPhase::Probe
                        } else {
                            MeasurementPhase::Legend
                        };
                        assert!(!inputs.matches(&initial_context, owner, round, Some(other))?);

                        // Raw mismatch must still short-circuit before the
                        // original fallible theme comparison. Equal raw inputs
                        // must reach that comparison, not a retained phase tag.
                        let invalid_context =
                            invalid_factory.context(&entry, SUPPRESSED_GUIDES, &scope)?;
                        assert!(inputs
                            .matches(&invalid_context, owner, round, Some(phase))
                            .is_err());
                        assert!(!inputs.matches(&invalid_context, owner, round, Some(other))?);
                        if phase == MeasurementPhase::Probe {
                            assert!(inputs.matches(&initial_context, owner, round, None)?);
                            assert!(inputs
                                .matches(&invalid_context, owner, round, None)
                                .is_err());
                        }
                    }
                    assert_eq!(produced.get(), 2);
                    let entry = allocation(None)?;
                    let full_scope = FitScope::new(owner, 0);
                    let context = factory.context(&entry, SUPPRESSED_GUIDES, &full_scope)?;
                    let inputs = PlotEntryInputs::new(&context, owner, round, None)?;
                    assert_eq!(inputs.plot.axis_snapshot_for_test().0, base.map(Some));
                    assert!(inputs.matches(&context, owner, round, None)?);
                    assert!(inputs.matches(
                        &initial_context,
                        owner,
                        round,
                        Some(MeasurementPhase::Probe)
                    )?);
                }
                Ok(crate::layout::MeasuredLayout::new(
                    assigned.assigned_size()?,
                    (),
                ))
            },
            |_, _, (), _| Ok(()),
        )
        .unwrap();
        assert_eq!(rounds, 2);
    }
}
