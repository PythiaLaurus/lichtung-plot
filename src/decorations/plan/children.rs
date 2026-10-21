//! Latest successful outcomes of immutable Canvas/facet child owners, not a
//! general plot cache. One instance belongs to one prepared root operation.

use std::sync::Arc;

use super::{LegendRole, PanelPlan};
use crate::decorations::legend::discrete::scoped::{GuideConsumption, GuideUse, ScopedGuideInputs};
use crate::layout::fit::FitOutcome;
use crate::layout::kernel::{ChildFeedback, ChildFeedbackInputs, FeedbackRound};
use crate::layout::{AddressRole, ComponentAddress, PanelBox};
use crate::spec::{FacetScales, PlotRenderContext, PlotRenderResult};
use crate::{Error, Plot};

#[cfg(test)]
use crate::orchestrator::measurements::{child_cache, Counter};

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
    axis_label_width: u64,
    content_height: bool,
    fill_content_height: bool,
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
        let theme = context.theme.clone();
        let grid_role = if owner.leaf_role() == AddressRole::CanvasCell {
            AddressRole::CanvasGrid
        } else {
            AddressRole::FacetGrid
        };
        let grid_owner = owner.under_grid(grid_role)?;
        let owners = [owner.clone(), grid_owner];
        Ok(Self {
            geometry: geometry(context.panel),
            axis_label_width: context.axis_label_width.to_bits(),
            content_height: context.content_height,
            fill_content_height: context.fill_content_height,
            plot: plot.retain_composed_inputs(ScopedGuideInputs::new(
                plot,
                context.guides.use_kind,
                context.guides.local_guide,
                context.guides.dependencies,
            )?)?,
            theme,
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
    ) -> Result<bool, Error> {
        Ok(self.owners[0] == *owner
            && self.geometry == geometry(context.panel)
            && self.axis_label_width == context.axis_label_width.to_bits()
            && self.content_height == context.content_height
            && self.fill_content_height == context.fill_content_height
            && self.plot.guides.matches_consumption(
                context.guides.use_kind,
                context.guides.local_guide,
                context.guides.dependencies,
            )
            && plot.same_composed_inputs(&self.plot, &self.plot.guides)
            && self.plot.guides.stable(round)
            && self.theme.same_validated(context.theme)?
            && self.scales == context.facet_scales()
            && round.child_inputs_equal(&self.feedback, &self.owners)
            && round.child_stable(&self.owners))
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

struct Panel {
    inputs: Inputs,
    role: LegendRole,
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
        context: PlotRenderContext<'_>,
        index: usize,
        phase: MeasurementPhase,
        round: &FeedbackRound,
    ) -> Result<Arc<PlotRenderResult>, Error> {
        let owner = ComponentAddress::root().child(
            AddressRole::CanvasCell,
            u32::try_from(index).map_err(|_| Error::Render("canvas address overflowed".into()))?,
        )?;
        crate::layout::fit::measured(self.measure(plot, context, &owner, phase, round, || {
            plot.preflight_measurement(
                plot.kind(),
                context.panel,
                context.theme,
                context.fit_scope(),
            )
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
        self.measurements.retain(|entry| {
            round.child_stable(&entry.inputs.owners) && entry.inputs.plot.guides.stable(round)
        });
        self.panels.retain(|entry| {
            round.child_stable(&entry.inputs.owners) && entry.inputs.plot.guides.stable(round)
        });
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
        let guides = GuideConsumption {
            use_kind: if plot.legend_policy_spec() == crate::LegendPolicy::Hidden {
                GuideUse::Hidden
            } else {
                context.guides.use_kind
            },
            local_guide: context.guides.local_guide,
            dependencies: context.guides.dependencies,
        };
        let context = context.with_guides(&guides);
        if reuse_enabled() {
            for entry in &self.measurements {
                if entry.phase == phase && entry.inputs.matches(plot, context, owner, round)? {
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
        discrete_guide: Option<&crate::decorations::legend::discrete::DiscreteGuideSlot>,
        round: &mut FeedbackRound,
        owner: &ComponentAddress,
    ) -> Result<FitOutcome<Arc<PanelPlan>>, Error> {
        self.validate_owner(owner)?;
        let use_kind = if plot.legend_policy_spec() == crate::LegendPolicy::Hidden {
            GuideUse::Hidden
        } else if role == LegendRole::Panel {
            GuideUse::Local
        } else {
            GuideUse::SharedSuppressed
        };
        let guides = GuideConsumption {
            use_kind,
            local_guide: discrete_guide,
            dependencies: context.guides.dependencies,
        };
        let context = context.with_guides(&guides);
        if reuse_enabled() {
            for entry in &self.panels {
                if entry.role == role && entry.inputs.matches(plot, context, owner, round)? {
                    #[cfg(test)]
                    child_cache(owner, "panel", Counter::ChildCacheHit);
                    round.replay_child(&entry.feedback)?;
                    #[cfg(test)]
                    child_cache(owner, "panel", Counter::ChildFeedbackReplay);
                    return Ok(FitOutcome::Measured(Arc::clone(&entry.result)));
                }
            }
        }
        #[cfg(test)]
        child_cache(owner, "panel", Counter::ChildCacheMiss);
        #[cfg(test)]
        CHILD_MEASUREMENTS.with(|events| events.borrow_mut().push((owner.clone(), "panel")));
        let checkpoint = round.child_checkpoint();
        let result = match super::plan_panel_with_context(
            plot,
            plot.kind(),
            role,
            context,
            discrete_guide,
            round,
            owner,
        )? {
            FitOutcome::Measured(result) => crate::data::source::source_arc(result)?,
            FitOutcome::Miss(miss) => return Ok(FitOutcome::Miss(miss)),
        };
        if !reuse_enabled() {
            return Ok(FitOutcome::Measured(result));
        }
        self.panels.retain(|entry| entry.inputs.owners[0] != *owner);
        if let Some(feedback) = round.retain_child_feedback(checkpoint)? {
            let inputs = Inputs::new(plot, context, owner, round)?;
            if round.child_stable(&inputs.owners) {
                crate::render::backend::reserve_text_records(&mut self.panels, 1)?;
                self.panels.push(Panel {
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
mod tests {
    use super::*;

    #[test]
    fn content_height_and_row_fill_each_invalidate_the_actual_child_input_key() {
        let source = Plot::scatter(
            crate::NumericValues::new("x", [0.0, 1.0]),
            crate::NumericValues::new("y", [0.0, 1.0]),
        )
        .erase();
        let theme = crate::Theme::publication().resolve().unwrap();
        theme.prepare_font_environment().unwrap();
        let root = crate::layout::plan_exact_root(
            800,
            600,
            |_| Ok(()),
            |(), assigned, round| {
                let owner = ComponentAddress::root().child(AddressRole::CanvasCell, 0)?;
                let context = PlotRenderContext::new(
                    source.data(),
                    crate::layout::PanelBox::new(20.0, 400.0, 20.0, 500.0),
                    &theme,
                );
                let key = Inputs::new(&source, context, &owner, round)?;
                assert!(key.matches(&source, context, &owner, round)?);
                for (content_height, fill_content_height) in
                    [(true, false), (false, true), (true, true)]
                {
                    let mut changed = context;
                    changed.content_height = content_height;
                    changed.fill_content_height = fill_content_height;
                    assert!(!key.matches(&source, changed, &owner, round)?);
                }
                Ok(crate::layout::MeasuredLayout::new(assigned, ()))
            },
            |(), _, (), _| Ok(()),
        )
        .unwrap();
        assert_eq!(root.dimensions(), (800, 600));
    }

    #[test]
    fn actual_child_hit_is_invalidated_by_legend_only_paint_and_geometry_changes() {
        let source = Plot::scatter(
            crate::NumericValues::new("x", [0.0, 1.0]),
            crate::NumericValues::new("y", [0.0, 1.0]),
        )
        .erase();
        let theme = crate::Theme::publication().resolve().unwrap();
        let owner = ComponentAddress::root()
            .child(AddressRole::FacetCell, 0)
            .unwrap();
        let mut children = ChildPlans::facets(&source);
        let mut produced = 0;
        crate::layout::plan_exact_root(
            800,
            600,
            |_| Ok(()),
            |(), assigned, round| {
                let panel = PanelBox::new(10.0, 200.0, 30.0, 400.0);
                let context = PlotRenderContext::new(source.data(), panel, &theme);
                let mut produce = || {
                    produced += 1;
                    Ok(FitOutcome::Measured(PlotRenderResult::new(
                        crate::layout::Layout::envelope(panel),
                        None,
                    )))
                };
                let first = crate::layout::fit::measured(children.measure(
                    &source,
                    context,
                    &owner,
                    MeasurementPhase::Legend,
                    round,
                    &mut produce,
                ))
                .map_err(crate::layout::fit::PlanError::into_error)?;
                let hit = crate::layout::fit::measured(children.measure(
                    &source,
                    context,
                    &owner,
                    MeasurementPhase::Legend,
                    round,
                    &mut produce,
                ))
                .map_err(crate::layout::fit::PlanError::into_error)?;
                assert!(Arc::ptr_eq(&first, &hit));
                let mut paint = theme.clone();
                let frame = paint
                    .legend
                    .frame()
                    .unwrap()
                    .clone()
                    .with_fill(crate::theme::Color::rgba(1, 2, 3, 120))?;
                paint.legend = paint.legend.clone().with_frame(Some(frame))?;
                let mut geometry = paint.clone();
                let frame = geometry.legend.frame().unwrap();
                let changed = crate::theme::LegendFrameTheme::new(
                    frame.fill(),
                    Some(crate::theme::OutlineTheme::new(
                        frame.outline().unwrap().color(),
                        7.0,
                    )?),
                    frame.radius().clone(),
                )?;
                geometry.legend = geometry.legend.clone().with_frame(Some(changed))?;
                for custom in [&paint, &geometry] {
                    let changed = crate::layout::fit::measured(children.measure(
                        &source,
                        PlotRenderContext::new(source.data(), panel, custom),
                        &owner,
                        MeasurementPhase::Legend,
                        round,
                        &mut produce,
                    ))
                    .map_err(crate::layout::fit::PlanError::into_error)?;
                    assert!(!Arc::ptr_eq(&first, &changed));
                    let hit = crate::layout::fit::measured(children.measure(
                        &source,
                        PlotRenderContext::new(source.data(), panel, custom),
                        &owner,
                        MeasurementPhase::Legend,
                        round,
                        &mut produce,
                    ))
                    .map_err(crate::layout::fit::PlanError::into_error)?;
                    assert!(Arc::ptr_eq(&changed, &hit));
                }
                Ok(crate::layout::MeasuredLayout::new(assigned, ()))
            },
            |(), _, (), _| Ok(()),
        )
        .unwrap();
        assert_eq!(produced, 3);
    }

    #[test]
    #[expect(
        clippy::too_many_lines,
        reason = "one actual child cache verifies hits and full normalized policy invalidation"
    )]
    fn child_measurement_reuses_hits_and_replaces_changed_numeric_policies() {
        use crate::numeric::{
            decimal::{NegativeZero, NumericNotation, TrailingZeros},
            format::{NumericFormat, NumericPrecision},
        };
        {
            let format = NumericFormat::default();
            let make = |format| {
                Plot::scatter(
                    crate::NumericValues::new("x", [0.0, 1.0]),
                    crate::NumericValues::new("y", [0.0, 1.0]),
                )
                .x_tick_format(format)
            };
            let source = make(format.clone()).erase();
            let formats = [
                format
                    .clone()
                    .with_notation(NumericNotation::Scientific)
                    .unwrap(),
                format
                    .clone()
                    .with_precision(NumericPrecision::Significant(2))
                    .unwrap(),
                format
                    .clone()
                    .with_precision(NumericPrecision::AdaptiveFractional { min: 2, max: 3 })
                    .unwrap(),
                format.clone().with_decimal_scale(1).unwrap(),
                format.clone().with_prefix("raw ").unwrap(),
                format.clone().with_suffix(" u").unwrap(),
                format
                    .clone()
                    .with_negative_zero(NegativeZero::Normalize)
                    .unwrap(),
                format
                    .clone()
                    .with_trailing_zeros(TrailingZeros::Trim)
                    .unwrap(),
            ];
            let mut variants = formats.map(|format| make(format).erase()).to_vec();
            for generation in [
                crate::TickGeneration::target(5).unwrap(),
                crate::TickGeneration::maximum(5).unwrap(),
            ] {
                variants.push(make(format.clone()).x_tick_generation(generation).erase());
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
            let theme = crate::Theme::publication().resolve().unwrap();
            let owner = ComponentAddress::root()
                .child(AddressRole::FacetCell, 0)
                .unwrap();
            let mut children = ChildPlans::facets(&source);
            let mut produced = 0;
            crate::layout::plan_exact_root(
                800,
                600,
                |_| Ok(()),
                |(), assigned, round| {
                    let context = PlotRenderContext::new(
                        source.data(),
                        PanelBox::new(10.0, 200.0, 30.0, 400.0),
                        &theme,
                    );
                    let mut produce = || {
                        produced += 1;
                        Ok(FitOutcome::Measured(PlotRenderResult::new(
                            crate::layout::Layout::envelope(crate::layout::PanelBox::new(
                                80.0, 770.0, 60.0, 570.0,
                            )),
                            None,
                        )))
                    };
                    let first = crate::layout::fit::measured(children.measure(
                        &source,
                        context,
                        &owner,
                        MeasurementPhase::Probe,
                        round,
                        &mut produce,
                    ))
                    .map_err(crate::layout::fit::PlanError::into_error)?;

                    let hit = crate::layout::fit::measured(children.measure(
                        &source,
                        context,
                        &owner,
                        MeasurementPhase::Probe,
                        round,
                        &mut produce,
                    ))
                    .map_err(crate::layout::fit::PlanError::into_error)?;
                    assert!(Arc::ptr_eq(&first, &hit));

                    let changed = crate::layout::fit::measured(children.measure(
                        &source,
                        context.with_axis_label_width(1.0),
                        &owner,
                        MeasurementPhase::Probe,
                        round,
                        &mut produce,
                    ))
                    .map_err(crate::layout::fit::PlanError::into_error)?;
                    assert_eq!(children.measurements.len(), 1);
                    assert!(!Arc::ptr_eq(&first, &changed));
                    assert!(Arc::ptr_eq(&changed, &children.measurements[0].result));
                    for plot in &variants {
                        let changed = crate::layout::fit::measured(children.measure(
                            plot,
                            context,
                            &owner,
                            MeasurementPhase::Probe,
                            round,
                            &mut produce,
                        ))
                        .map_err(crate::layout::fit::PlanError::into_error)?;
                        let hit = crate::layout::fit::measured(children.measure(
                            plot,
                            context,
                            &owner,
                            MeasurementPhase::Probe,
                            round,
                            &mut produce,
                        ))
                        .map_err(crate::layout::fit::PlanError::into_error)?;
                        assert!(!Arc::ptr_eq(&first, &changed));
                        assert!(Arc::ptr_eq(&changed, &hit));
                        let inputs = &children.measurements[0].inputs.plot;
                        assert!(!source.same_composed_inputs(inputs, &inputs.guides));
                    }
                    Ok(crate::layout::MeasuredLayout::new(assigned, ()))
                },
                |(), _, (), _| Ok(()),
            )
            .unwrap();
            assert_eq!(produced, 2 + variants.len());
        }
    }

    #[test]
    fn later_panel_cache_replays_feedback_without_remeasurement() {
        let source = Plot::scatter(
            crate::NumericValues::new("x", [0.0, 1.0]),
            crate::NumericValues::new("y", [0.0, 1.0]),
        )
        .erase();
        let theme = crate::Theme::publication().resolve().unwrap();
        let owner = ComponentAddress::root()
            .child(AddressRole::FacetCell, 0)
            .unwrap();
        let sibling = ComponentAddress::root()
            .child(AddressRole::FacetCell, 1)
            .unwrap();
        let mut children = ChildPlans::facets(&source);
        let mut rounds = 0;
        let mut finalized = false;
        let result = crate::layout::plan_exact_root(
            800,
            600,
            |_| Ok(()),
            |(), assigned, round| {
                rounds += 1;
                let context = PlotRenderContext::new(
                    source.data(),
                    PanelBox::new(10.0, 200.0, 30.0, 400.0),
                    &theme,
                );
                if rounds == 1 {
                    let start = round.child_checkpoint();
                    round.observe_number(1.0)?;
                    // Seed a successful immutable panel to isolate the replay boundary.
                    children.panels.push(Panel {
                        inputs: Inputs::new(&source, context, &owner, round)?,
                        role: LegendRole::Suppressed,
                        result: Arc::new(PanelPlan {
                            frame_box: context.panel,
                            panel_box: context.panel,
                            result: PlotRenderResult::new(
                                crate::layout::Layout::envelope(crate::layout::PanelBox::new(
                                    80.0, 770.0, 60.0, 570.0,
                                )),
                                None,
                            ),
                            legend: None,
                            planned_x_limits: None,
                            planned_y_limits: None,
                        }),
                        feedback: round.retain_child_feedback(start)?.unwrap(),
                    });
                } else {
                    CHILD_MEASUREMENTS.with(|events| events.borrow_mut().clear());
                    let replayed = crate::layout::fit::measured(children.plan(
                        &source,
                        LegendRole::Suppressed,
                        context,
                        None,
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
                Ok(crate::layout::MeasuredLayout::new(assigned, ()))
            },
            |(), _, (), _| {
                finalized = true;
                Ok(())
            },
        );
        result.unwrap();
        assert_eq!(rounds, 2);
        assert!(finalized);
    }

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
        let theme = crate::Theme::publication().resolve().unwrap();
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
                assert!(key.matches(&source, context, &owner, round)?);
                assert!(!key.matches(
                    &source,
                    context.with_axis_label_width(context.axis_label_width + 1.0),
                    &owner,
                    round
                )?);
                for moved in [
                    PanelBox::new(11.0, 201.0, 30.0, 400.0),
                    PanelBox::new(10.0, 200.0, 31.0, 401.0),
                    PanelBox::new(10.0 + 1.0 / 4096.0, 200.0, 30.0, 400.0),
                ] {
                    assert!(!key.matches(&source, context.with_panel(moved), &owner, round)?);
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
                    source.clone().with_parent_axis_gutters(),
                    source.clone().with_suppressed_legend(),
                ] {
                    assert!(!key.matches(&plot, context, &owner, round)?);
                }
                let mut appearance = theme.clone();
                appearance.typography.axes.tick_label.font_size += 1;
                assert!(!key.matches(
                    &source,
                    PlotRenderContext::new(source.data(), panel, &appearance),
                    &owner,
                    round
                )?);
                assert!(!key.matches(
                    &source,
                    context.with_facet_scales(FacetScales::Free),
                    &owner,
                    round
                )?);
                assert!(!key.matches(&source, context, &neighbor, round)?);
                round.request_insets(
                    &neighbor,
                    &neighbor,
                    crate::layout::kernel::FeedbackKind::OwnedInk,
                    [1.0; 4],
                )?;
                assert!(key.matches(&source, context, &owner, round)?);
                for kind in [
                    crate::layout::kernel::FeedbackKind::AxisGutter,
                    crate::layout::kernel::FeedbackKind::SharedAlignment,
                    crate::layout::kernel::FeedbackKind::OwnedInk,
                ] {
                    let checkpoint = round.begin_trial()?;
                    round.request_insets(&owner, &owner, kind, [1.0; 4])?;
                    assert!(!key.matches(&source, context, &owner, round)?);
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
