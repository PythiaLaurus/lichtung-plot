//! Latest successful outcomes of immutable Canvas/facet child owners, not a
//! general plot cache. One instance belongs to one prepared root operation.

use std::sync::Arc;

use super::{LegendRole, PanelPlan};
use crate::layout::fit::FitOutcome;
use crate::layout::kernel::{ChildFeedback, ChildFeedbackInputs, FeedbackRound};
use crate::layout::{AddressRole, ComponentAddress, PanelBox};
use crate::spec::{FacetScales, PlotRenderContext, PlotRenderResult};
use crate::{Error, Plot};

#[cfg(test)]
thread_local! {
    pub(crate) static DISABLE_CHILD_REUSE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    pub(crate) static CHILD_MEASUREMENTS: std::cell::RefCell<Vec<(ComponentAddress, &'static str)>> = const { std::cell::RefCell::new(Vec::new()) };
}

// These boundaries construct fresh producer contexts. Source rows, source
// configuration and prepared fonts are fixed by the borrowed root and cell;
// coordination's computed replacements are compared by value, not pointers.
struct Inputs {
    owners: [ComponentAddress; 2],
    geometry: [u64; 4],
    plot: crate::builder::ComposedInputs,
    theme: crate::theme::ResolvedTheme,
    scales: Option<FacetScales>,
    feedback: ChildFeedbackInputs,
}

impl Inputs {
    fn new(
        plot: &Plot,
        context: PlotRenderContext<'_>,
        owner: &ComponentAddress,
        round: &FeedbackRound,
    ) -> Result<Self, Error> {
        let environment = context.theme.prepared_font_environment()?;
        // under_grid requires one cell segment: its intermediate/final addresses
        // use one/two segments, and the copied cell address uses one more.
        environment
            .claim_text_generation_slots(4)
            .map_err(|error| Error::Render(format!("child address records: {error:?}")))?;
        environment
            .claim_unicode_bytes(
                4 * std::mem::size_of::<crate::layout::kernel::AddressSegment>() as u64,
            )
            .map_err(|error| Error::Render(error.to_string()))?;
        let grid_role = if owner.leaf_role() == AddressRole::CanvasCell {
            AddressRole::CanvasGrid
        } else {
            AddressRole::FacetGrid
        };
        let grid_owner = owner.under_grid(grid_role)?;
        let owners = [owner.clone(), grid_owner];
        Ok(Self {
            geometry: geometry(context.panel),
            plot: plot.retain_composed_inputs(environment)?,
            theme: context.theme.clone(),
            scales: context.facet_scales(),
            feedback: round.child_inputs(&owners)?,
            owners,
        })
    }

    fn matches(
        &self,
        plot: &Plot,
        context: PlotRenderContext<'_>,
        owner: &ComponentAddress,
        round: &FeedbackRound,
    ) -> bool {
        self.owners[0] == *owner
            && self.geometry == geometry(context.panel)
            && plot.same_composed_inputs(&self.plot)
            && self.theme == *context.theme
            && self.scales == context.facet_scales()
            && round.child_inputs_equal(&self.feedback, &self.owners)
            && round.child_stable(&self.owners)
    }
}

fn geometry(panel: PanelBox) -> [u64; 4] {
    [panel.x0, panel.x1, panel.y0, panel.y1].map(f64::to_bits)
}

struct Measurement {
    inputs: Inputs,
    phase: MeasurementPhase,
    result: Arc<PlotRenderResult>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum MeasurementPhase {
    Layout,
    Probe,
    Hue,
    Legend,
    Range,
    Category,
}

#[cfg(test)]
impl MeasurementPhase {
    fn label(self) -> &'static str {
        match self {
            Self::Layout => "layout",
            Self::Probe => "probe",
            Self::Hue => "hue-probe",
            Self::Legend => "legend-probe",
            Self::Range => "range-probe",
            Self::Category => "category-probe",
        }
    }
}

struct Panel {
    inputs: Inputs,
    role: LegendRole,
    legend_entries: Option<Arc<[crate::decorations::legend::LegendEntry]>>,
    result: Arc<PanelPlan>,
    feedback: ChildFeedback,
}

enum RootSource<'a> {
    Canvas { _source: &'a crate::Canvas },
    Facets { _source: &'a Plot },
}

pub(crate) struct ChildPlans<'a> {
    source: RootSource<'a>,
    measurements: Vec<Measurement>,
    panels: Vec<Panel>,
}

impl<'a> ChildPlans<'a> {
    pub(crate) fn canvas(source: &'a crate::Canvas) -> Self {
        Self {
            source: RootSource::Canvas { _source: source },
            measurements: Vec::new(),
            panels: Vec::new(),
        }
    }

    pub(crate) fn facets(source: &'a Plot) -> Self {
        Self {
            source: RootSource::Facets { _source: source },
            measurements: Vec::new(),
            panels: Vec::new(),
        }
    }

    fn validate_owner(&self, owner: &ComponentAddress) -> Result<(), Error> {
        let role = match self.source {
            RootSource::Canvas { .. } => AddressRole::CanvasCell,
            RootSource::Facets { .. } => AddressRole::FacetCell,
        };
        if owner.leaf_role() != role {
            return Err(Error::Render(
                "child result belongs to a different root topology".into(),
            ));
        }
        Ok(())
    }

    pub(crate) fn canvas_probe(
        &mut self,
        plot: &Plot,
        panel: PanelBox,
        theme: &crate::theme::ResolvedTheme,
        index: usize,
        phase: MeasurementPhase,
        round: &FeedbackRound,
    ) -> Result<Arc<PlotRenderResult>, Error> {
        let owner = ComponentAddress::root().child(
            AddressRole::CanvasCell,
            u32::try_from(index).map_err(|_| Error::Render("canvas address overflowed".into()))?,
        )?;
        let context = PlotRenderContext::new(plot.data(), panel, theme);
        crate::layout::fit::measured(self.measure(plot, context, &owner, phase, round, || {
            plot.preflight_measurement(plot.kind(), panel, theme, &crate::layout::fit::ROOT_SCOPE)
        }))
        .map_err(crate::layout::fit::PlanError::into_error)
    }

    /// Called after all siblings and parent requests have been measured. A
    /// provisional result must not become an active entry for the next round.
    pub(crate) fn finish_round(&mut self, round: &FeedbackRound) {
        if round.child_stable(&[ComponentAddress::root()]) {
            // The driver will freeze this plan. Do not retain discarded probes
            // and child keys through final capture and output admission.
            self.measurements.clear();
            self.panels.clear();
            return;
        }
        self.measurements
            .retain(|entry| round.child_stable(&entry.inputs.owners));
        self.panels
            .retain(|entry| round.child_stable(&entry.inputs.owners));
    }

    pub(crate) fn measure(
        &mut self,
        plot: &Plot,
        context: PlotRenderContext<'_>,
        owner: &ComponentAddress,
        phase: MeasurementPhase,
        round: &FeedbackRound,
        measure: impl FnOnce() -> Result<FitOutcome<PlotRenderResult>, Error>,
    ) -> Result<FitOutcome<Arc<PlotRenderResult>>, Error> {
        self.validate_owner(owner)?;
        if reuse_enabled() {
            for entry in &self.measurements {
                context
                    .theme
                    .prepared_font_environment()?
                    .claim_processing_visits(1)
                    .map_err(|error| Error::Render(format!("child lookup visits: {error:?}")))?;
                if entry.phase == phase && entry.inputs.matches(plot, context, owner, round) {
                    return Ok(FitOutcome::Measured(Arc::clone(&entry.result)));
                }
            }
        }
        #[cfg(test)]
        CHILD_MEASUREMENTS.with(|events| events.borrow_mut().push((owner.clone(), phase.label())));
        let result = match measure()? {
            FitOutcome::Measured(result) => Arc::new(result),
            FitOutcome::Miss(miss) => return Ok(FitOutcome::Miss(miss)),
        };
        if !reuse_enabled() {
            return Ok(FitOutcome::Measured(result));
        }
        self.measurements
            .retain(|entry| entry.inputs.owners[0] != *owner || entry.phase != phase);
        crate::render::backend::reserve_text_records(
            &mut self.measurements,
            1,
            context.theme.prepared_font_environment()?,
        )?;
        self.measurements.push(Measurement {
            inputs: Inputs::new(plot, context, owner, round)?,
            phase,
            result: Arc::clone(&result),
        });
        Ok(FitOutcome::Measured(result))
    }

    pub(crate) fn plan(
        &mut self,
        plot: &Plot,
        role: LegendRole,
        context: PlotRenderContext<'_>,
        legend_entries: Option<&Arc<[crate::decorations::legend::LegendEntry]>>,
        round: &mut FeedbackRound,
        owner: &ComponentAddress,
    ) -> Result<FitOutcome<Arc<PanelPlan>>, Error> {
        self.validate_owner(owner)?;
        if reuse_enabled() {
            for entry in &self.panels {
                context
                    .theme
                    .prepared_font_environment()?
                    .claim_processing_visits(1)
                    .map_err(|error| Error::Render(format!("child lookup visits: {error:?}")))?;
                if entry.role == role
                    && match (
                        entry.legend_entries.as_deref(),
                        legend_entries.map(AsRef::as_ref),
                    ) {
                        (None, None) => true,
                        (Some(a), Some(b)) => {
                            a.len() == b.len()
                                && a.iter().zip(b).all(|(a, b)| a.same_child_input(b))
                        }
                        _ => false,
                    }
                    && entry.inputs.matches(plot, context, owner, round)
                {
                    round.replay_child(&entry.feedback)?;
                    return Ok(FitOutcome::Measured(Arc::clone(&entry.result)));
                }
            }
        }
        #[cfg(test)]
        CHILD_MEASUREMENTS.with(|events| events.borrow_mut().push((owner.clone(), "panel")));
        let checkpoint = round.child_checkpoint();
        let result = match super::plan_panel_with_context(
            plot,
            plot.kind(),
            role,
            context,
            legend_entries.map(AsRef::as_ref),
            round,
            owner,
        )? {
            FitOutcome::Measured(result) => Arc::new(result),
            FitOutcome::Miss(miss) => return Ok(FitOutcome::Miss(miss)),
        };
        if !reuse_enabled() {
            return Ok(FitOutcome::Measured(result));
        }
        self.panels.retain(|entry| entry.inputs.owners[0] != *owner);
        if let Some(feedback) = round.retain_child_feedback(checkpoint)? {
            let inputs = Inputs::new(plot, context, owner, round)?;
            if round.child_stable(&inputs.owners) {
                crate::render::backend::reserve_text_records(
                    &mut self.panels,
                    1,
                    context.theme.prepared_font_environment()?,
                )?;
                self.panels.push(Panel {
                    inputs,
                    role,
                    legend_entries: legend_entries.map(Arc::clone),
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
mod tests {
    use super::*;

    #[test]
    #[expect(
        clippy::too_many_lines,
        reason = "one key matrix keeps all concrete parent-input invalidation cases together"
    )]
    fn child_keys_include_exact_origin_computed_policy_and_relevant_feedback() {
        let source = Plot::scatter(
            crate::NumericValues::new("x", [0.0, 1.0]),
            crate::NumericValues::new("y", [0.0, 1.0]),
        )
        .erase();
        let theme = crate::Theme::Publication.resolve();
        let owner = ComponentAddress::root()
            .child(AddressRole::CanvasCell, 0)
            .unwrap();
        let neighbor = ComponentAddress::root()
            .child(AddressRole::CanvasCell, 1)
            .unwrap();
        crate::layout::plan_exact_root(
            800,
            600,
            |_| Ok(()),
            |(), assigned, round| {
                let panel = PanelBox::new(10.0, 200.0, 30.0, 400.0);
                let context = PlotRenderContext::new(source.data(), panel, &theme);
                assert!(Inputs::new(
                    &source,
                    context,
                    &owner.child(AddressRole::CanvasCell, 1)?,
                    round
                )
                .is_err());
                let key = Inputs::new(&source, context, &owner, round)?;
                assert!(key.matches(&source, context, &owner, round));
                for moved in [
                    PanelBox::new(11.0, 201.0, 30.0, 400.0),
                    PanelBox::new(10.0, 200.0, 31.0, 401.0),
                    PanelBox::new(10.0 + 1.0 / 4096.0, 200.0, 30.0, 400.0),
                ] {
                    assert!(!key.matches(&source, context.with_panel(moved), &owner, round));
                }
                for plot in vec![
                    source.clone().with_planned_x_limits((-1.0, 2.0)),
                    source.clone().with_planned_y_limits((-1.0, 2.0)),
                    source.clone().with_resolved_facet_ranges(
                        crate::PlotKind::Forest,
                        Some((-1.0, 2.0)),
                        None,
                    ),
                    source
                        .clone()
                        .with_global_hue_levels(vec!["new hue".into()]),
                    source
                        .clone()
                        .with_global_group_levels(vec!["new group".into()]),
                    source
                        .clone()
                        .with_global_line_condition_levels(vec!["new condition".into()]),
                    source.clone().with_global_collective_pairs(Vec::new()),
                    source.clone().with_y_tick_labels_visible(false),
                    source.clone().with_measured_composed_axis_text(),
                    source.clone().with_suppressed_legend(),
                ] {
                    assert!(!key.matches(&plot, context, &owner, round));
                }
                let mut appearance = theme.clone();
                appearance.typography.tick_label.font_size += 1;
                assert!(!key.matches(
                    &source,
                    PlotRenderContext::new(source.data(), panel, &appearance),
                    &owner,
                    round
                ));
                assert!(!key.matches(
                    &source,
                    context.with_facet_scales(FacetScales::Free),
                    &owner,
                    round
                ));
                assert!(!key.matches(&source, context, &neighbor, round));
                round.request_insets(
                    &neighbor,
                    &neighbor,
                    crate::layout::kernel::FeedbackKind::OwnedInk,
                    [1.0; 4],
                )?;
                assert!(key.matches(&source, context, &owner, round));
                for kind in [
                    crate::layout::kernel::FeedbackKind::AxisGutter,
                    crate::layout::kernel::FeedbackKind::SharedAlignment,
                    crate::layout::kernel::FeedbackKind::OwnedInk,
                ] {
                    let checkpoint = round.begin_trial()?;
                    round.request_insets(&owner, &owner, kind, [1.0; 4])?;
                    assert!(!key.matches(&source, context, &owner, round));
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
                Ok(crate::layout::MeasuredLayout::new(assigned, ()))
            },
            |(), _, (), _| Ok(()),
        )
        .unwrap();
    }
}
