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
thread_local! {
    pub(crate) static DISABLE_CHILD_REUSE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    pub(crate) static CHILD_MEASUREMENTS: std::cell::RefCell<Vec<(ComponentAddress, &'static str)>> = const { std::cell::RefCell::new(Vec::new()) };
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
                    return Ok(FitOutcome::Measured(Arc::clone(&entry.result)));
                }
            }
        }
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
                    round.replay_child(&cached.feedback)?;
                    return Ok(FitOutcome::Measured(Arc::clone(&cached.result)));
                }
            }
        }
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
