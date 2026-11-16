//! One-canvas chart lifecycle and immutable standalone scene planning.

use crate::Error;

use super::assignment::{
    GuideTrialAssignment, ParentInsets, PlotAllocationInputs, PlotMeasureDimensions, RootAssignment,
};
use super::plan::{MeasuredOwnedPlan, MeasuredRootContent};
use super::registration::PlotRegistration;
use super::root::{CachedRootFrame, RootLayoutCaches};
use super::root::{CompositionIntrinsics, PreparedComposition, Root, RootSource};
use super::{capture_calls, validate_text_replay_plan, TextReplayPlan};
#[cfg(test)]
use crate::builder::Plot;
use crate::builder::PlotDescription;
use crate::decorations::plan::MeasuredPlotPlan;
use crate::layout::PixelRect;
use crate::render::backend::{Backend, SurfaceRole};
#[cfg(test)]
use crate::spec::PlotRenderResult;
use crate::spec::{DecorationMode, PlotKind};
use crate::text::{TextRequestContext, TextService};
use crate::theme::ResolvedTheme;

pub(crate) fn plan_scene(
    source: RootSource<'_>,
) -> Result<crate::layout::FrozenRoot<TextReplayPlan>, Error> {
    let frozen = crate::layout::plan_root(
        source.request(),
        |stage| Root::prepare(source, stage),
        Root::measure,
        |root, size, plan, stage| {
            let _ = &root.census;
            size.dimensions()?;
            let verifier = root.sealed_verifier()?;
            super::capture_selected(root, plan, &verifier, stage)
        },
    )?;
    let (width, height) = frozen.dimensions();
    validate_text_replay_plan(
        width,
        height,
        frozen.plan(),
        match source {
            RootSource::Plot(_) => "prepared plot Null replay",
            RootSource::Canvas(_) => "prepared canvas Null replay",
        },
    )?;
    Ok(frozen)
}

pub(crate) fn render_scene(source: RootSource<'_>) -> Result<crate::scene::RenderedScene, Error> {
    let frozen = plan_scene(source)?;
    let (width, height) = frozen.dimensions();
    let mut backend = frozen.plan().take_svg_backend()?;
    #[cfg(test)]
    if matches!(source, RootSource::Plot(_)) {
        super::FINAL_PLOT_RENDER_CALLS.with(|calls| calls.set(calls.get() + 1));
    }
    frozen.plan().replay(
        width,
        height,
        &mut backend,
        match source {
            RootSource::Plot(_) => "prepared plot scene replay",
            RootSource::Canvas(_) => "prepared canvas scene replay",
        },
    )?;
    crate::scene::RenderedScene::new(width, height, backend.into_output())
}

#[expect(
    clippy::needless_pass_by_value,
    reason = "capture consumes the selected plan but keeps it alive through command and SVG-storage finalization"
)]
pub(crate) fn capture_selected(
    root: &Root<'_>,
    plan: MeasuredOwnedPlan,
    verifier: &crate::font::canonical::SealedTextVerifier<'_>,
    stage: &mut crate::layout::kernel::FinalStage<'_>,
) -> Result<TextReplayPlan, Error> {
    capture_calls(plan.size, verifier, stage, |backend| {
        let (width, height) = plan.size.dimensions()?;
        match root.source {
            RootSource::Plot(source) => {
                render_plot_scene(source, source.kind(), width, height, &plan, backend)
            }
            RootSource::Canvas(source) => {
                source.render_planned_canvas(&plan, width, height, backend)
            }
        }
    })
}

pub(super) fn measure_plot_root(
    root: &Root<'_>,
    source: &PlotDescription,
    assigned: crate::layout::RootRequest,
    round: &mut crate::layout::kernel::FeedbackRound,
    children: &mut crate::decorations::plan::children::ChildPlans,
    frames: &mut Option<CachedRootFrame>,
) -> Result<MeasuredOwnedPlan, Error> {
    let request = root.request();
    let route = crate::layout::interior::InteriorRoute::classify(source.kind());
    let size = if let crate::layout::RootRequest::ExactSize { width, height } = assigned {
        crate::layout::RootSize::new(width, height)
    } else {
        route.validate(source)?;
        match &root.composition {
            PreparedComposition::Facets(topology) => {
                super::facets::natural_size(source, root, topology, &request, assigned, round)?
            }
            PreparedComposition::Single { plot_id } => crate::layout::interior::sizing::plot_size(
                root.plot(plot_id)?,
                root.plot_intrinsic(plot_id)?,
                &root.operation,
                &request,
                assigned,
                round,
            )?,
            PreparedComposition::Canvas(_) => unreachable!("Plot composition"),
        }
    };
    let (width, _) = size.dimensions()?;
    root.root_text
        .check_width(
            root.root_appearance.root_inner_width(f64::from(width))?,
            &crate::layout::fit::FitScope::root(),
        )
        .map_err(crate::layout::fit::PlanError::into_error)?;
    route.validate(source)?;
    let measurement = root.assignment_factory().assign(size);
    match &root.composition {
        PreparedComposition::Facets(topology) => super::facets::OwnedFacetPlan::measure(
            source,
            root,
            topology,
            measurement,
            &request,
            round,
            children,
            root.source.request().automatic_height(),
        )
        .map_err(crate::layout::fit::PlanError::into_error),
        PreparedComposition::Single { plot_id } => {
            let registered = root.plot(plot_id)?;
            let intrinsic = root.plot_intrinsic(plot_id)?;
            let frame = cached_root_frame(
                registered,
                size,
                &request,
                intrinsic,
                &root.operation,
                frames,
            )?;
            measure_single_plot(root, registered, measurement, &request, frame, round)
        }
        PreparedComposition::Canvas(_) => unreachable!("Plot composition"),
    }
}

pub(super) fn prepare_plot_root<'src>(
    source: &'src PlotDescription,
    prepared_stage: &mut crate::layout::kernel::PreparedStage<'_>,
) -> Result<Root<'src>, Error> {
    prepare_plot_root_with_theme(source, source.resolve_theme()?, prepared_stage)
}

pub(super) fn prepare_plot_root_with_theme<'src>(
    source: &'src PlotDescription,
    theme: ResolvedTheme,
    prepared_stage: &mut crate::layout::kernel::PreparedStage<'_>,
) -> Result<Root<'src>, Error> {
    let text = TextService::default();
    source.validate_configuration(source.kind())?;
    let (registry, composition) =
        crate::plots::shared_x::prepare_plot(source, &prepared_stage.operation(), &theme)?;
    crate::layout::interior::guide::validate_categorical_scale(
        source,
        &registry.panels,
        &theme,
        None,
    )?;
    complete_plot_root(source, theme, text, registry, composition, prepared_stage)
}

#[expect(
    clippy::too_many_lines,
    reason = "one preparation tail preserves the original guide, font census and post-font intrinsic producer order"
)]
pub(super) fn complete_plot_root<'src>(
    source: &'src PlotDescription,
    theme: ResolvedTheme,
    text: TextService,
    mut registry: crate::orchestrator::registration::PlotRegistryBuilder<'src>,
    mut composition: PreparedComposition,
    prepared_stage: &mut crate::layout::kernel::PreparedStage<'_>,
) -> Result<Root<'src>, Error> {
    let request = TextRequestContext::new(&theme, &text);
    let facet_guide = source
        .has_facets()
        .then(|| {
            super::prepare_discrete_guide(
                source,
                source.kind(),
                &mut registry,
                &prepared_stage.operation(),
            )
        })
        .transpose()?;
    let root_guide = if source.has_facets() {
        facet_guide
            .as_ref()
            .and_then(|guide| guide.descriptor.clone())
    } else {
        crate::layout::interior::guide::prepare_discrete_guides(
            source,
            &mut registry.panels,
            &prepared_stage.operation(),
            false,
        )?
    };
    if let PreparedComposition::Facets(topology) = &mut composition {
        topology.parent_guide = root_guide
            .as_ref()
            .map(|descriptor| {
                crate::decorations::legend::discrete::GuideSlot::new(
                    descriptor.clone(),
                    &crate::layout::ComponentAddress::root(),
                    source.legend_position_spec(),
                )
            })
            .transpose()?;
    }
    let plots = registry.finish()?;
    let plot_intrinsics = crate::data::source::collect_source(plots.iter().map(|plot| {
        Ok((
            plot.scope.id.clone(),
            super::root::PlotIntrinsics::for_kind(plot.source.kind()),
        ))
    }))?;
    let mut requirements = crate::layout::foundational_plot_requirements(
        Some(&plots),
        root_guide.as_ref(),
        source,
        source.kind(),
        &request,
        &crate::layout::ComponentAddress::root(),
        true,
        true,
        true,
        Some(prepared_stage),
    )?;
    crate::layout::PreparedTextRequirement::append_preference_metrics(
        &mut requirements,
        &request,
        &crate::layout::ComponentAddress::root(),
        prepared_stage,
    )?;
    let text_topology = crate::layout::PreparedTextCensus::structural_address_count(&requirements)
        .map_err(|error| Error::Render(format!("prepared text census: {error}")))?;
    let mut scalar_presence = prepared_stage.reserve_scalar_presence_map()?;
    let mut census = crate::layout::PreparedTextCensus::materialize(
        requirements,
        Some(prepared_stage),
        Some(&mut scalar_presence),
    )
    .map_err(|error| Error::Render(format!("prepared text census: {error}")))?;
    if census.counts.structural_addresses != text_topology {
        return Err(Error::Render(format!(
            "prepared text topology counted {text_topology} addresses but materialized {}",
            census.counts.structural_addresses
        )));
    }
    text.prepare_font_environment_from_census(&mut census, scalar_presence, prepared_stage)?;
    // Scientific preparation and the font census are complete. No
    // parent width has been allocated when these summaries are retained.
    crate::layout::interior::InteriorRoute::classify(source.kind()).validate(source)?;
    for (registered, (_, intrinsic)) in plots.iter().zip(&plot_intrinsics) {
        if registered.axes.coordinate().is_none() {
            continue;
        }
        super::root::prepare_plot_intrinsics(
            intrinsic,
            &super::root::PlotIntrinsicInputs {
                registered,
                request: TextRequestContext {
                    theme: &registered.appearance,
                    ..request
                },
                operation: &prepared_stage.operation(),
                diagnostic_scope: registered.scope.render_name.as_deref(),
                y_visible: registered.policy.axis_text.tick_labels[1],
            },
        )?;
    }
    let root_text = crate::decorations::title_block::plan::RootTitleBlockPlan::intrinsic_widths(
        &request,
        source.prose(),
        None,
    )?;
    Ok(Root {
        source: RootSource::Plot(source),
        identity: prepared_stage.operation().root_identity(),
        next_candidate: std::cell::Cell::new(0),
        operation: prepared_stage.operation(),
        plots,
        plot_intrinsics,
        composition,
        composition_intrinsics: if source.has_facets() {
            CompositionIntrinsics::Facets
        } else {
            CompositionIntrinsics::Single
        },
        root_appearance: theme,
        text,
        census,
        root_text,
        caches: std::cell::RefCell::new(RootLayoutCaches::new(
            crate::decorations::plan::children::ChildPlans::facets(),
        )),
    })
}

/// Continue a fixture's real, already completed scientific registry at the
/// original guide/census/font boundary. No statistics or scope is prepared here.
#[cfg(test)]
pub(crate) fn complete_prepared_plot_for_test<'src>(
    source: &'src PlotDescription,
    appearance: ResolvedTheme,
    registry: crate::orchestrator::registration::PlotRegistryBuilder<'src>,
    composition: PreparedComposition,
    stage: &mut crate::layout::kernel::PreparedStage<'_>,
) -> Result<Root<'src>, Error> {
    complete_plot_root(
        source,
        appearance,
        TextService::default(),
        registry,
        composition,
        stage,
    )
}

/// Keep producer fixtures inside the production prepare/font/measure operation.
/// Only owning results, scalar observations and liveness witnesses may escape.
#[cfg(test)]
pub(crate) fn with_prepared_plot_for_test<T>(
    source: &Plot,
    appearance: &TextRequestContext<'_, '_>,
    configure: impl FnOnce(
        &mut crate::orchestrator::registration::PlotRegistryBuilder<'_>,
    ) -> Result<(), Error>,
    visit: impl FnOnce(
        &PlotDescription,
        &Root<'_>,
        &TextRequestContext<'_, '_>,
        &crate::font::canonical::OperationContext,
        &mut crate::layout::kernel::FeedbackRound,
    ) -> T,
) -> Result<T, Error> {
    let mut visit = Some(visit);
    let mut result = None;
    let sizing = match source.description().sizing_request() {
        exact @ crate::layout::RootRequest::ExactSize { .. } => exact,
        _ => crate::layout::RootRequest::ExactSize {
            width: 1,
            height: 1,
        },
    };
    crate::layout::plan_root(
        sizing,
        |stage| {
            let theme = appearance.theme.clone();
            let text = TextService::default();
            source.description().validate_configuration(source.kind())?;
            let (mut registry, composition) = crate::plots::shared_x::prepare_plot(
                source.description(),
                &stage.operation(),
                &theme,
            )?;
            crate::layout::interior::guide::validate_categorical_scale(
                source.description(),
                &registry.panels,
                &theme,
                None,
            )?;
            configure(&mut registry)?;
            complete_plot_root(
                source.description(),
                theme,
                text,
                registry,
                composition,
                stage,
            )
        },
        |prepared, assigned, round| {
            assigned.assigned_size()?.dimensions()?;
            crate::layout::interior::InteriorRoute::classify(source.kind())
                .validate(source.description())?;
            let request = prepared.request();
            result = Some(visit.take().expect("one feedback-free producer fixture")(
                source.description(),
                prepared,
                &request,
                &prepared.operation,
                round,
            ));
            Ok(crate::layout::MeasuredLayout::new(
                assigned.assigned_size()?,
                (),
            ))
        },
        |_, _, (), _| Ok(()),
    )?;
    Ok(result.expect("producer fixture was measured"))
}

pub(crate) fn cached_root_frame(
    registered: &PlotRegistration<'_>,
    size: crate::layout::RootSize,
    request: &TextRequestContext<'_, '_>,
    intrinsic: &super::root::PlotIntrinsics,
    operation: &crate::font::canonical::OperationContext,
    frames: &mut Option<CachedRootFrame>,
) -> Result<std::sync::Arc<crate::layout::RootFramePlan>, Error> {
    let source = registered.source;
    let dimensions = size.dimensions()?;
    #[cfg(test)]
    super::root::FRAME_CACHE_READS.with(|events| events.borrow_mut().push("dimensions"));
    let minimum = crate::layout::interior::intrinsic_body_minimum(
        registered,
        crate::builder::RootFrameDomainInput::Prepared,
        intrinsic,
        operation,
        request,
    )?;
    let minimum = [minimum.0.to_bits(), minimum.1.to_bits()];
    #[cfg(test)]
    super::root::FRAME_CACHE_READS.with(|events| events.borrow_mut().push("minimum"));

    if let Some(cached) = frames.as_ref() {
        if cached.dimensions == dimensions
            && cached.minimum == minimum
            && source.same_prose(&cached.prose)
        {
            #[cfg(test)]
            super::root::FRAME_CACHE_READS.with(|events| events.borrow_mut().push("hit"));
            return Ok(std::sync::Arc::clone(&cached.frame));
        }
    }
    let frame = crate::builder::measure_root_frame(
        registered,
        size,
        crate::builder::RootFrameDomainInput::Prepared,
        request,
        intrinsic,
        operation,
        &crate::layout::fit::FitScope::root(),
    )
    .map_err(crate::layout::fit::PlanError::into_error)?;

    let frame = frame.into_shared();
    *frames = Some(CachedRootFrame {
        prose: source.copy_prose()?,
        dimensions,
        minimum,
        frame: std::sync::Arc::clone(&frame),
    });
    Ok(frame)
}

#[cfg(test)]
#[test]
fn accepted_scene_holds_no_operation_fonts_or_analysis_during_publication() {
    let source = Plot::scatter(
        crate::NumericValues::new("x", [0.0, 1.0]),
        crate::NumericValues::new("y", [1.0, 0.0]),
    )
    .title("Independent frozen title")
    .size(640, 480)
    .erase();
    let mut probes = None;
    let plan = PlotPlan::new_with_theme_factory(
        &source,
        source.kind(),
        || Ok(Box::new(source.description().resolve_theme()?)),
        |prepared, replay| {
            let environment = prepared.text.prepared_font_environment().unwrap();
            let core = environment.liveness_probe();
            let operation = environment.operation().liveness_probe();
            let caches = prepared.caches.borrow();
            let (_, title) = caches
                .frames
                .as_ref()
                .unwrap()
                .frame
                .root_text
                .placed_text()
                .next()
                .unwrap();
            let analysis = title.analysis_liveness_probe();
            assert!(core() && operation() && analysis());
            assert!(replay
                .retained_text()
                .iter()
                .any(|(_, text)| text.shares_content(title)));
            probes = Some((core, operation, analysis));
        },
    )
    .unwrap();
    let (core, operation, analysis) = probes.unwrap();
    assert!(!core() && !operation() && !analysis());
    drop(source);
    let mut backend = plan.take_svg_backend().unwrap();
    plan.render(&mut backend).unwrap();
    let scene = crate::scene::RenderedScene::new(640, 480, backend.into_output()).unwrap();
    drop(plan);
    let outline = scene.svg_with_text_as_paths().unwrap();
    assert!(!std::str::from_utf8(&outline).unwrap().contains("<text"));
    assert!(scene.png_bytes(&crate::PngOptions::default()).is_ok());
    assert!(!core() && !operation() && !analysis());
}

#[cfg(test)]
pub(super) struct PlotPlan {
    root: crate::layout::FrozenRoot<TextReplayPlan>,
}

#[cfg(test)]
impl PlotPlan {
    pub(super) fn new(source: &Plot, kind: PlotKind) -> Result<Self, Error> {
        Self::new_with_theme_factory(
            source,
            kind,
            || Ok(Box::new(source.description().resolve_theme()?)),
            #[cfg(test)]
            |_, _| {},
        )
    }

    fn new_with_theme_factory(
        source: &Plot,
        _kind: PlotKind,
        resolve_theme: impl FnOnce() -> Result<Box<ResolvedTheme>, Error>,
        inspect: impl FnOnce(&Root<'_>, &TextReplayPlan),
    ) -> Result<Self, Error> {
        let sizing = source.description().sizing_request();
        let root = crate::layout::plan_root(
            sizing,
            |prepared_stage| {
                prepare_plot_root_with_theme(
                    source.description(),
                    *resolve_theme()?,
                    prepared_stage,
                )
            },
            Root::measure,
            |prepared, assigned, layout, final_stage| {
                assigned.dimensions()?;
                #[cfg(test)]
                let fonts = crate::font::canonical::planning_counts();
                let verifier = prepared.sealed_verifier()?;
                let replay = capture_selected(prepared, layout, &verifier, final_stage)?;
                #[cfg(test)]
                assert_eq!(
                    crate::font::canonical::planning_counts(),
                    fonts,
                    "Plot capture must not discover, resolve, probe, admit, seal or shape fonts"
                );
                #[cfg(test)]
                inspect(prepared, &replay);
                Ok(replay)
            },
        )?;
        let (width, height) = root.dimensions();
        validate_text_replay_plan(width, height, root.plan(), "prepared plot Null replay")?;
        Ok(Self { root })
    }

    pub(super) fn render(&self, backend: &mut dyn Backend) -> Result<(), Error> {
        #[cfg(test)]
        super::FINAL_PLOT_RENDER_CALLS.with(|calls| calls.set(calls.get() + 1));
        let (width, height) = self.dimensions();
        self.root
            .plan()
            .replay(width, height, backend, "prepared plot scene replay")
    }

    pub(super) fn dimensions(&self) -> (u32, u32) {
        self.root.dimensions()
    }

    pub(super) fn take_svg_backend(&self) -> Result<crate::render::svg_backend::SvgBackend, Error> {
        self.root.plan().take_svg_backend()
    }
}

fn render_plot_scene(
    source: &PlotDescription,
    kind: PlotKind,
    width: u32,
    height: u32,
    layout: &MeasuredOwnedPlan,
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    backend.draw_surface(
        PixelRect::new(0.0, f64::from(width), 0.0, f64::from(height)),
        layout.root_paint,
        SurfaceRole::CanvasRoot,
    )?;
    match &layout.content {
        MeasuredRootContent::Single { plot, legend } => {
            render_single_plot(source, kind, plot, legend.as_ref(), &layout.frame, backend)
        }
        MeasuredRootContent::Facets(plan) => plan.render(source, kind, &layout.frame, backend),
        MeasuredRootContent::Canvas(_) => unreachable!("standalone measured content"),
    }
}

fn measure_single_plot(
    root: &Root<'_>,
    registered: &PlotRegistration<'_>,
    measurement: RootAssignment,
    theme: &TextRequestContext<'_, '_>,
    canvas_layout: std::sync::Arc<crate::layout::RootFramePlan>,
    round: &mut crate::layout::kernel::FeedbackRound,
) -> Result<MeasuredOwnedPlan, Error> {
    let source = registered.source;
    let kind = source.kind();
    let intrinsic = root.plot_intrinsic(&registered.scope.id)?;
    let cell = canvas_layout.panel_box;
    source.configured_source_rows()?;
    theme.text.prepared_font_environment()?;
    let factory = root.plot_assignment_factory(&measurement, &registered.scope.id)?;
    let scope = crate::layout::fit::FitScope::root();
    let entry = factory.assign(
        PlotAllocationInputs {
            dimensions: PlotMeasureDimensions {
                canvas: measurement.measurement_size,
                output_override: None,
            },
            cell,
            frame: cell,
            body: cell,
            axis_label_width: cell.width(),
            content_height: root.source.request().automatic_height(),
            fill_content_height: false,
            content_preference: None,
            insets: ParentInsets::consumed(round, &registered.scope.address),
            guide: GuideTrialAssignment {
                edge: source.legend_position_spec(),
                compact: false,
                allocation: None,
            },
        },
        round,
        None,
    )?;
    super::validate_interior_preparation(registered)?;
    let discrete_guide = if kind == PlotKind::Matrix {
        None
    } else {
        super::registration::guide_descriptor(Some(registered))?
            .map(|descriptor| {
                crate::decorations::legend::discrete::GuideSlot::new(
                    descriptor,
                    &crate::layout::ComponentAddress::root()
                        .child(crate::layout::AddressRole::PlotBody, 0)?,
                    source.legend_position_spec(),
                )
            })
            .transpose()?
    };
    let guides = crate::decorations::legend::discrete::scoped::GuideConsumption {
        use_kind: super::registration::discrete_guide_use(source, Some(registered)),
        local_guide: discrete_guide.as_ref(),
        dependencies: &[],
    };
    let panel_plan = crate::layout::fit::measured(crate::decorations::plan::plan_plot(
        &factory,
        entry,
        crate::decorations::plan::LegendRole::Panel,
        guides,
        &scope,
        round,
    ))
    .map_err(crate::layout::fit::PlanError::into_error)?;

    complete_single(
        registered,
        measurement,
        canvas_layout,
        panel_plan,
        theme,
        intrinsic,
        &root.operation,
        round,
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "standalone completion keeps selected geometry and its original intrinsic, operation and feedback owners explicit"
)]
pub(crate) fn complete_single(
    registered: &PlotRegistration<'_>,
    measurement: RootAssignment,
    mut canvas_layout: std::sync::Arc<crate::layout::RootFramePlan>,
    panel_plan: std::sync::Arc<MeasuredPlotPlan>,
    theme: &TextRequestContext<'_, '_>,
    intrinsic: &super::root::PlotIntrinsics,
    operation: &crate::font::canonical::OperationContext,
    round: &mut crate::layout::kernel::FeedbackRound,
) -> Result<MeasuredOwnedPlan, Error> {
    let source = registered.source;
    let kind = source.kind();
    let mut size = measurement.measurement_size;
    if panel_plan.selected.content_height {
        let height = crate::layout::kernel::root_dimension(
            canvas_layout.height_for_body(
                panel_plan
                    .preference
                    .as_ref()
                    .expect("content frame height preference")
                    .height,
                theme,
            )?,
        )?;
        size = crate::layout::RootSize::new(
            panel_plan.selected.dimensions.canvas_size().dimensions()?.0,
            height,
        );
        let mut resized = crate::builder::measure_root_frame(
            registered,
            size,
            crate::builder::RootFrameDomainInput::Selected(&panel_plan.selected),
            theme,
            intrinsic,
            operation,
            &crate::layout::fit::FitScope::root(),
        )
        .map_err(crate::layout::fit::PlanError::into_error)?;
        resized.panel_box = crate::orchestrator::assignment::completion_frame(
            kind,
            &panel_plan.selected,
            &panel_plan.result,
        );
        canvas_layout = resized.into_shared();
    }
    round.observe_canvas(&canvas_layout)?;
    let legend = panel_plan
        .legend
        .as_ref()
        .map(|resolved| {
            let (x, y) = resolved.bounds.anchor();
            let draw = resolved.legend.select(x, y)?;
            let mut geometry = crate::render::geometry::GeometrySnapshot::new();
            draw.geometry(&mut geometry)?;
            geometry.observe_feedback(round)?;
            let scope = crate::layout::fit::FitScope::root()
                .child(crate::layout::AddressRole::PlotBody, 0)?
                .child(
                    crate::decorations::legend::address_role(source.legend_position_spec()),
                    0,
                )?;
            let (width, height) = size.dimensions()?;
            if geometry.bounds().iter().any(|ink| {
                ink.left < 0.0
                    || ink.right > f64::from(width)
                    || ink.bottom < 0.0
                    || ink.top > f64::from(height)
            }) {
                round.record_failure(scope.failure(
                    crate::layout::fit::FitKind::LegendOrGuide,
                    "selected standalone legend ink exceeds the exact root",
                ))?;
            }
            let mut layer = String::new();
            let length = super::registration::render_name(Some(registered))
                .map_or(Some(6), |scope| scope.len().checked_add(7))
                .ok_or_else(|| Error::Render("standalone legend layer length overflow".into()))?;
            crate::data::source::reserve_source_string(&mut layer, length)?;
            if let Some(scope) = super::registration::render_name(Some(registered)) {
                layer.push_str(scope);
                layer.push('-');
            }
            layer.push_str("legend");
            Ok::<_, Error>((layer, draw))
        })
        .transpose()?;
    Ok(MeasuredOwnedPlan {
        measurement,
        size,
        frame: canvas_layout,
        root_paint: theme.surfaces.canvas_root,
        content: MeasuredRootContent::Single {
            plot: panel_plan,
            legend,
        },
    })
}

fn render_single_plot(
    source: &PlotDescription,
    kind: PlotKind,
    plot: &MeasuredPlotPlan,
    legend: Option<&(String, crate::decorations::legend::LegendDrawPlan)>,
    frame: &crate::layout::RootFramePlan,
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    backend.draw_surface(plot.panel_box, plot.surface, SurfaceRole::PlotPanel)?;
    if plot.result.decoration_mode == DecorationMode::Standard {
        crate::decorations::apply::apply_grid(
            &plot.layers.as_ref().expect("standalone plot layers").grid,
            plot,
            backend,
        )?;
    }
    source.configured_source_rows()?;
    crate::layout::interior::InteriorRoute::classify(kind).validate(source)?;
    let rendered = plot.result.draw_body(backend)?;

    crate::significance::render_selected(&plot.result, backend)?;

    match plot.result.decoration_mode {
        DecorationMode::Standard => {
            crate::decorations::apply::apply_axes(
                &plot.layers.as_ref().expect("standalone plot layers").axes,
                plot,
                backend,
            )?;
            if let Some((layer, legend)) = legend {
                crate::render::backend::with_layer(backend, layer, |backend| legend.draw(backend))?;
            }
        }
        DecorationMode::SelfManaged => {}
    }
    frame.root_text.draw(source.prose(), backend)?;
    debug_assert_eq!(rendered, plot.result.decoration_mode);
    Ok(())
}
#[cfg(test)]
impl Plot {
    pub(crate) fn preflight_panel(
        &self,
        kind: PlotKind,
        panel: crate::layout::PixelRect,
        theme: &TextRequestContext<'_, '_>,
    ) -> Result<PlotRenderResult, Error> {
        with_prepared_plot_for_test(
            self,
            theme,
            |_| Ok(()),
            |source, root, request, _operation, round| {
                crate::layout::interior::InteriorRoute::classify(kind).validate(source)?;
                source.validate_category_order_configuration(kind)?;
                if !source.significance_annotations().is_empty()
                    && !kind.supports_significance_annotations()
                {
                    return Err(Error::Data(format!(
                        "significance annotations are not supported for {kind:?}; use a supported categorical family or remove the annotations"
                    )));
                }
                source.configured_source_rows()?;
                request.text.prepared_font_environment()?;
                let measurement = root.assignment_factory().assign(
                    source
                        .sizing_request()
                        .assigned_size()
                        .unwrap_or(crate::layout::RootSize::new(1, 1)),
                );
                let registered = &root.plots[0];
                let factory = root.plot_assignment_factory(&measurement, &registered.scope.id)?;
                let entry = factory.assign(
                    super::assignment::PlotAllocationInputs::for_test(
                        measurement.measurement_size,
                        panel,
                        round,
                        &registered.scope.address,
                    ),
                    round,
                    None,
                )?;
                crate::layout::fit::measured(crate::layout::fit::outcome(
                    super::measure_probe(
                        &factory,
                        entry,
                        crate::decorations::legend::discrete::scoped::SUPPRESSED_GUIDES,
                        &crate::layout::fit::ROOT_SCOPE,
                    )
                    .map(|(_, result)| result),
                ))
                .map_err(crate::layout::fit::PlanError::into_error)
            },
        )?
    }
}

#[cfg(test)]
mod natural_sizing_tests {
    use super::*;

    #[test]
    fn content_root_capture_is_immutable_and_png_refusal_remains_terminal_local() {
        for width in [None, Some(300)] {
            let plot = Plot::scatter(
                crate::NumericValues::new("x", [0.0, 1.0, 2.0]),
                crate::NumericValues::new("y", [0.0, 1.0, 2.0]),
            )
            .caption("A complete explanatory caption that is retained at the chosen width.")
            .erase();
            let plot = width.map_or_else(|| plot.clone(), |width| plot.clone().width(width));
            crate::scene::PNG_TARGET_VALIDATION_CALLS.with(|calls| calls.set(0));
            let plan = PlotPlan::new(&plot, plot.kind()).unwrap();
            assert_eq!(
                crate::scene::PNG_TARGET_VALIDATION_CALLS.with(std::cell::Cell::get),
                0
            );
            let dimensions = plan.dimensions();
            drop(plot);
            let frozen = || {
                [
                    crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get),
                    crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get),
                    crate::plots::shared_x::AXIS_PREPARATIONS.with(std::cell::Cell::get),
                    crate::orchestrator::SELECTED_INTERIOR_MEASUREMENTS.with(std::cell::Cell::get),
                ]
            };
            let before = frozen();
            let mut svg = plan.take_svg_backend().unwrap();
            plan.render(&mut svg).unwrap();
            let scene =
                crate::scene::RenderedScene::new(dimensions.0, dimensions.1, svg.into_output())
                    .unwrap();
            let preserved = scene.svg_bytes().to_vec();
            let outlined = scene.svg_with_text_as_paths().unwrap();
            assert!(!std::str::from_utf8(&outlined).unwrap().contains("<text"));
            assert!(scene
                .png_bytes(&crate::PngOptions {
                    sizing: crate::PngSizing::Dpi(120_000_000.0),
                    ..crate::PngOptions::default()
                })
                .is_err());
            assert_eq!(scene.svg_bytes(), preserved);
            assert_eq!(scene.svg_with_text_as_paths().unwrap(), outlined);
            assert_eq!(frozen(), before);
        }
    }
}
