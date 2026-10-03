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
    standard: crate::layout::StandardPanelInputs,
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
        let resources = environment.generation_resources();
        let data = context.data.with_axis_resources(&resources);
        Self::admit_theme_clone(context.theme, data)?;
        let grid_role = if owner.leaf_role() == AddressRole::CanvasCell {
            AddressRole::CanvasGrid
        } else {
            AddressRole::FacetGrid
        };
        let grid_owner = owner.under_grid(grid_role, &resources)?;
        let owners = [owner.clone(), grid_owner];
        Ok(Self {
            geometry: geometry(context.panel),
            axis_label_width: context.axis_label_width.to_bits(),
            content_height: context.content_height,
            fill_content_height: context.fill_content_height,
            standard: crate::layout::StandardPanelInputs::new(context.theme)?,
            plot: plot.retain_composed_inputs(environment)?,
            theme: context.theme.clone(),
            scales: context.facet_scales(),
            feedback: round.child_inputs(&owners)?,
            owners,
        })
    }

    fn admit_theme_clone(
        theme: &crate::theme::ResolvedTheme,
        data: crate::data::PlotData<'_>,
    ) -> Result<(), Error> {
        data.admit_clone_slice(&theme.colors)?;
        data.admit_clone_str(&theme.font_family)?;
        for appearance in [
            &theme.typography.title,
            &theme.typography.subtitle,
            &theme.typography.caption,
            &theme.typography.axis_title,
            &theme.typography.tick_label,
            &theme.typography.significance_label,
            &theme.typography.estimate_detail,
            &theme.typography.waterfall_total,
            &theme.typography.point_layer_label,
            &theme.typography.panel_text,
            &theme.typography.contrast_light,
            &theme.typography.contrast_dark,
            &theme.typography.legend_title,
            &theme.typography.legend_label,
        ] {
            data.admit_clone_str(&appearance.stack)?;
        }
        Ok(())
    }

    fn matches(
        &self,
        plot: &Plot,
        context: PlotRenderContext<'_>,
        owner: &ComponentAddress,
        round: &FeedbackRound,
    ) -> Result<bool, Error> {
        let resources = context
            .theme
            .prepared_font_environment()?
            .generation_resources();
        Ok(self.owners[0] == *owner
            && self.geometry == geometry(context.panel)
            && self.axis_label_width == context.axis_label_width.to_bits()
            && self.content_height == context.content_height
            && self.fill_content_height == context.fill_content_height
            && self.standard.matches(context.theme)
            && plot
                .same_composed_inputs(&self.plot, context.data.with_axis_resources(&resources))?
            && self.theme == *context.theme
            && self.scales == context.facet_scales()
            && round.child_inputs_equal(&self.feedback, &self.owners)?
            && round.child_stable(&self.owners)?)
    }
}

fn geometry(panel: PanelBox) -> [u64; 4] {
    [panel.x0, panel.x1, panel.y0, panel.y1].map(f64::to_bits)
}

fn retain_stable<T>(
    entries: &mut Vec<T>,
    mut stable: impl FnMut(&T) -> Result<bool, Error>,
) -> Result<(), Error> {
    let mut retained = 0;
    for examined in 0..entries.len() {
        if stable(&entries[examined])? {
            entries.swap(retained, examined);
            retained += 1;
        }
    }
    entries.truncate(retained);
    Ok(())
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
            &theme.prepared_font_environment()?.generation_resources(),
        )?;
        let context = PlotRenderContext::new(plot.data(), panel, theme);
        crate::layout::fit::measured(self.measure(plot, context, &owner, phase, round, |call| {
            plot.preflight_measurement_in_call(
                plot.kind(),
                panel,
                theme,
                &crate::layout::fit::ROOT_SCOPE,
                call,
            )
        }))
        .map_err(crate::layout::fit::PlanError::into_error)
    }

    /// Called after all siblings and parent requests have been measured. A
    /// provisional result must not become an active entry for the next round.
    pub(crate) fn finish_round(&mut self, round: &FeedbackRound) -> Result<(), Error> {
        if round.child_stable(&[ComponentAddress::root()])? {
            // The driver will freeze this plan. Do not retain discarded probes
            // and child keys through final capture and output admission.
            self.measurements.clear();
            self.panels.clear();
            return Ok(());
        }
        retain_stable(&mut self.measurements, |entry| {
            round.child_stable(&entry.inputs.owners)
        })?;
        retain_stable(&mut self.panels, |entry| {
            round.child_stable(&entry.inputs.owners)
        })?;
        Ok(())
    }

    pub(crate) fn measure(
        &mut self,
        plot: &Plot,
        context: PlotRenderContext<'_>,
        owner: &ComponentAddress,
        phase: MeasurementPhase,
        round: &FeedbackRound,
        measure: impl FnOnce(
            &crate::font::canonical::MeasurementCall,
        ) -> Result<FitOutcome<PlotRenderResult>, Error>,
    ) -> Result<FitOutcome<Arc<PlotRenderResult>>, Error> {
        self.validate_owner(owner)?;
        let call = context
            .theme
            .prepared_font_environment()?
            .claim_measurement_call()
            .map_err(|error| {
                Error::Render(format!("child measurement call admission: {error:?}"))
            })?;
        if reuse_enabled() {
            for entry in &self.measurements {
                context
                    .theme
                    .prepared_font_environment()?
                    .claim_processing_visits(1)
                    .map_err(|error| Error::Render(format!("child lookup visits: {error:?}")))?;
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
        let result = match measure(&call)? {
            FitOutcome::Measured(result) => {
                let resources = context
                    .theme
                    .prepared_font_environment()?
                    .generation_resources();
                context
                    .data
                    .with_axis_resources(&resources)
                    .source_arc(result)?
            }
            FitOutcome::Miss(miss) => return Ok(FitOutcome::Miss(miss)),
        };
        if !reuse_enabled() {
            return Ok(FitOutcome::Measured(result));
        }
        context
            .theme
            .prepared_font_environment()?
            .claim_cache_entries(1)
            .map_err(|error| Error::Render(format!("child cache admission: {error:?}")))?;
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
        let environment = context.theme.prepared_font_environment()?;
        let call = environment
            .claim_measurement_call()
            .map_err(|error| Error::Render(format!("child panel call admission: {error:?}")))?;
        if reuse_enabled() {
            for entry in &self.panels {
                environment
                    .claim_processing_visits(1)
                    .map_err(|error| Error::Render(format!("child lookup visits: {error:?}")))?;
                if entry.role == role
                    && match (
                        entry.legend_entries.as_deref(),
                        legend_entries.map(AsRef::as_ref),
                    ) {
                        (None, None) => true,
                        (Some(a), Some(b)) => {
                            let mut equal = a.len() == b.len();
                            if equal {
                                for (a, b) in a.iter().zip(b) {
                                    environment.claim_processing_visits(1).map_err(|error| {
                                        Error::Render(format!(
                                            "child legend lookup visits: {error:?}"
                                        ))
                                    })?;
                                    if !a.same_child_input(b) {
                                        equal = false;
                                        break;
                                    }
                                }
                            }
                            equal
                        }
                        _ => false,
                    }
                    && entry.inputs.matches(plot, context, owner, round)?
                {
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
            legend_entries.map(AsRef::as_ref),
            round,
            owner,
            &call,
        )? {
            FitOutcome::Measured(result) => {
                let resources = environment.generation_resources();
                context
                    .data
                    .with_axis_resources(&resources)
                    .source_arc(result)?
            }
            FitOutcome::Miss(miss) => return Ok(FitOutcome::Miss(miss)),
        };
        if !reuse_enabled() {
            return Ok(FitOutcome::Measured(result));
        }
        self.panels.retain(|entry| entry.inputs.owners[0] != *owner);
        if let Some(feedback) = round.retain_child_feedback(checkpoint)? {
            let inputs = Inputs::new(plot, context, owner, round)?;
            if round.child_stable(&inputs.owners)? {
                environment
                    .claim_cache_entries(1)
                    .map_err(|error| Error::Render(format!("child cache admission: {error:?}")))?;
                crate::render::backend::reserve_text_records(&mut self.panels, 1, environment)?;
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
    fn content_height_and_row_fill_each_invalidate_the_actual_child_input_key() {
        let source = Plot::scatter(
            crate::NumericValues::new("x", [0.0, 1.0]),
            crate::NumericValues::new("y", [0.0, 1.0]),
        )
        .erase();
        let theme = crate::Theme::Publication.resolve();
        theme.prepare_font_environment().unwrap();
        let root = crate::layout::plan_exact_root(
            800,
            600,
            |_| Ok(()),
            |(), assigned, round| {
                let owner = ComponentAddress::root().child(
                    AddressRole::CanvasCell,
                    0,
                    round.resources(),
                )?;
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
    fn complete_theme_clone_admits_exact_owned_bytes_and_records_before_copy() {
        let source = Plot::scatter(
            crate::NumericValues::new("x", [0.0, 1.0]),
            crate::NumericValues::new("y", [0.0, 1.0]),
        )
        .erase();
        let mut theme = crate::Theme::Publication.resolve();
        for (index, stack) in [
            &mut theme.typography.title.stack,
            &mut theme.typography.subtitle.stack,
            &mut theme.typography.caption.stack,
            &mut theme.typography.axis_title.stack,
            &mut theme.typography.tick_label.stack,
            &mut theme.typography.significance_label.stack,
            &mut theme.typography.estimate_detail.stack,
            &mut theme.typography.waterfall_total.stack,
            &mut theme.typography.point_layer_label.stack,
            &mut theme.typography.panel_text.stack,
            &mut theme.typography.contrast_light.stack,
            &mut theme.typography.contrast_dark.stack,
            &mut theme.typography.legend_title.stack,
            &mut theme.typography.legend_label.stack,
        ]
        .into_iter()
        .enumerate()
        {
            *stack = "s".repeat(index + 1);
        }
        theme.font_family = "s".repeat(15);
        let bytes = 120 + std::mem::size_of_val(theme.colors.as_slice()) as u64;
        let records = theme.colors.len() as u128 + 15;
        let resources = crate::font::canonical::OperationGenerationResources::new();
        resources.set_test_limits(u128::MAX, bytes);
        resources.claim_slots(1_000_000 - records).unwrap();
        Inputs::admit_theme_clone(&theme, source.data().with_axis_resources(&resources)).unwrap();
        assert_eq!(resources.byte_usage().unwrap(), bytes);
        assert_eq!(resources.usage().unwrap().0, 1_000_000);

        for refuse_bytes in [false, true] {
            let resources = crate::font::canonical::OperationGenerationResources::new();
            resources.set_test_limits(u128::MAX, bytes - u64::from(refuse_bytes));
            resources
                .claim_slots(1_000_000 - records + u128::from(!refuse_bytes))
                .unwrap();
            let error =
                Inputs::admit_theme_clone(&theme, source.data().with_axis_resources(&resources))
                    .unwrap_err();
            assert!(error.to_string().contains(if refuse_bytes {
                "system-font bytes"
            } else {
                "retained planning records"
            }));
            // The last stack has fourteen bytes. Its rejected claim never copies it.
            assert_eq!(resources.byte_usage().unwrap(), bytes - 14);
            assert_eq!(resources.usage().unwrap().0, 1_000_000);
        }
    }

    #[test]
    fn actual_child_hit_is_invalidated_by_legend_only_paint_and_geometry_changes() {
        let source = Plot::scatter(
            crate::NumericValues::new("x", [0.0, 1.0]),
            crate::NumericValues::new("y", [0.0, 1.0]),
        )
        .erase();
        let theme = crate::Theme::Publication.resolve();
        let owner = ComponentAddress::root()
            .child(
                AddressRole::FacetCell,
                0,
                &crate::font::canonical::OperationGenerationResources::new(),
            )
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
                let mut produce = |_: &crate::font::canonical::MeasurementCall| {
                    produced += 1;
                    Ok(FitOutcome::Measured(PlotRenderResult::new(
                        crate::layout::Layout::envelope(panel),
                        Vec::new(),
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
                paint.legend.frame_fill = crate::render::backend::Rgba::new(1, 2, 3, 120);
                let mut geometry = paint.clone();
                geometry.legend.frame_stroke_width = 7.0;
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
    fn child_measurement_hits_and_insertions_use_independent_cumulative_limits() {
        for cache_limit in [false, true] {
            let source = Plot::scatter(
                crate::NumericValues::new("x", [0.0, 1.0]),
                crate::NumericValues::new("y", [0.0, 1.0]),
            )
            .erase();
            let theme = crate::Theme::Publication.resolve();
            let owner = ComponentAddress::root()
                .child(
                    AddressRole::FacetCell,
                    0,
                    &crate::font::canonical::OperationGenerationResources::new(),
                )
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
                    let mut produce = |_: &crate::font::canonical::MeasurementCall| {
                        produced += 1;
                        Ok(FitOutcome::Measured(PlotRenderResult::new(
                            crate::layout::Layout::envelope(crate::layout::PanelBox::new(
                                80.0, 770.0, 60.0, 570.0,
                            )),
                            Vec::new(),
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
                    let resources = theme.prepared_font_environment()?.generation_resources();
                    if cache_limit {
                        resources
                            .claim_cache_entries(1_000_000 - resources.call_cache_usage().1)
                            .unwrap();
                    } else {
                        resources
                            .claim_calls(999_999 - resources.call_cache_usage().0)
                            .unwrap();
                    }
                    let entries = resources.call_cache_usage().1;
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
                    assert_eq!(resources.call_cache_usage().1, entries);
                    let error = children
                        .measure(
                            &source,
                            context.with_axis_label_width(1.0),
                            &owner,
                            MeasurementPhase::Probe,
                            round,
                            &mut produce,
                        )
                        .err()
                        .expect("call or cache refusal");
                    assert!(error.to_string().contains(if cache_limit {
                        "root-local cache entries"
                    } else {
                        "measurement/intrinsic call attempts"
                    }));
                    assert_eq!(children.measurements.len(), 1);
                    assert!(Arc::ptr_eq(&first, &children.measurements[0].result));
                    Ok(crate::layout::MeasuredLayout::new(assigned, ()))
                },
                |(), _, (), _| Ok(()),
            )
            .unwrap();
            assert_eq!(produced, if cache_limit { 2 } else { 1 });
        }
    }

    #[test]
    fn later_panel_cache_replay_record_refusal_is_fatal_without_remeasurement() {
        let source = Plot::scatter(
            crate::NumericValues::new("x", [0.0, 1.0]),
            crate::NumericValues::new("y", [0.0, 1.0]),
        )
        .erase();
        let theme = crate::Theme::Publication.resolve();
        let owner = ComponentAddress::root()
            .child(
                AddressRole::FacetCell,
                0,
                &crate::font::canonical::OperationGenerationResources::new(),
            )
            .unwrap();
        let sibling = ComponentAddress::root()
            .child(
                AddressRole::FacetCell,
                1,
                &crate::font::canonical::OperationGenerationResources::new(),
            )
            .unwrap();
        let mut children = ChildPlans::facets(&source);
        let mut rounds = 0;
        let mut finalized = false;
        let result = crate::layout::plan_exact_root(
            800,
            600,
            |stage| Ok(stage.generation_resources()),
            |resources, assigned, round| {
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
                        legend_entries: None,
                        result: Arc::new(PanelPlan {
                            frame_box: context.panel,
                            panel_box: context.panel,
                            result: PlotRenderResult::new(
                                crate::layout::Layout::envelope(crate::layout::PanelBox::new(
                                    80.0, 770.0, 60.0, 570.0,
                                )),
                                Vec::new(),
                            ),
                            legend: None,
                            planned_x_limits: None,
                            planned_y_limits: None,
                        }),
                        feedback: round.retain_child_feedback(start)?.unwrap(),
                    });
                } else {
                    resources
                        .claim_slots(1_000_000 - resources.usage().unwrap().0)
                        .unwrap();
                    CHILD_MEASUREMENTS.with(|events| events.borrow_mut().clear());
                    let Err(error) = children.plan(
                        &source,
                        LegendRole::Suppressed,
                        context,
                        None,
                        round,
                        &owner,
                    ) else {
                        panic!("replay must refuse its observation record");
                    };
                    assert!(error.to_string().contains("retained planning records"));
                    CHILD_MEASUREMENTS.with(|events| assert!(events.borrow().is_empty()));
                    return Err(error);
                }
                round.request_insets(
                    &sibling,
                    &sibling,
                    crate::layout::kernel::FeedbackKind::AxisGutter,
                    [1.0; 4],
                )?;
                Ok(crate::layout::MeasuredLayout::new(assigned, ()))
            },
            |_, _, (), _| {
                finalized = true;
                Ok(())
            },
        );
        assert!(result.is_err());
        assert_eq!(rounds, 2);
        assert!(!finalized);
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
        let theme = crate::Theme::Publication.resolve();
        let owner = ComponentAddress::root()
            .child(
                AddressRole::CanvasCell,
                0,
                &crate::font::canonical::OperationGenerationResources::new(),
            )
            .unwrap();
        let neighbor = ComponentAddress::root()
            .child(
                AddressRole::CanvasCell,
                1,
                &crate::font::canonical::OperationGenerationResources::new(),
            )
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
                    &owner.child(AddressRole::CanvasCell, 1, round.resources())?,
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
                appearance.typography.tick_label.font_size += 1;
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
                        round.finish_trial(checkpoint)?,
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
