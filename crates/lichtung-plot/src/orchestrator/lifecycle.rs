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
    #[cfg(test)]
    if matches!(source, RootSource::Plot(_)) {
        super::PLOT_PLAN_CALLS.with(|calls| calls.set(calls.get() + 1));
    }
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
#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one prepared operation witnesses minimum-before-cache order and complete prose changes across misses and hits"
)]
fn prepared_root_frame_reuses_complete_prose_only_with_identical_constraints_on_its_operation() {
    let source = Plot::scatter(
        crate::NumericValues::new("x", [0.0, 1.0]),
        crate::NumericValues::new("y", [1.0, 0.0]),
    )
    .title("Root title")
    .subtitle("Independent subtitle")
    .caption("Immutable caption")
    .size(640, 480)
    .erase();
    PlotPlan::new_with_theme_factory(
        &source,
        source.kind(),
        || Ok(Box::new(source.description().resolve_theme()?)),
        |prepared, _| {
            crate::layout::interior::InteriorRoute::classify(source.kind())
                .validate(source.description())
                .unwrap();
            let registered = &prepared.plots[0];
            let intrinsic = prepared.plot_intrinsic(&registered.scope.id).unwrap();
            let request = prepared.request();
            let frame = |width, frames: &mut Option<CachedRootFrame>| {
                cached_root_frame(
                    registered,
                    crate::layout::RootSize::new(width, 480),
                    &request,
                    intrinsic,
                    &prepared.operation,
                    frames,
                )
                .unwrap()
            };

            let mut caches = prepared.caches.borrow_mut();
            let frames = &mut caches.frames;
            frames.take();
            super::root::FRAME_CACHE_READS.with(|events| events.borrow_mut().clear());
            let original = frame(640, frames);
            super::root::FRAME_CACHE_READS.with(|events| {
                assert_eq!(
                    &*events.borrow(),
                    &["dimensions", "minimum", "frame minimum"]
                );
                events.borrow_mut().clear();
            });
            let shapes = crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get);

            let repeated = frame(640, frames);
            super::root::FRAME_CACHE_READS.with(|events| {
                assert_eq!(&*events.borrow(), &["dimensions", "minimum", "hit"]);
            });
            assert_eq!(original, repeated);

            assert_eq!(
                crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get),
                shapes
            );
            for ((_, left), (_, right)) in original
                .root_text
                .placed_text()
                .zip(repeated.root_text.placed_text())
            {
                assert!(left.shares_payload(right));
            }
            let changed = frame(641, frames);
            assert_ne!(changed.panel_box, repeated.panel_box);

            assert_eq!(
                changed
                    .root_text
                    .placed_text()
                    .map(|(_, text)| text.source.as_str())
                    .collect::<Vec<_>>(),
                ["Root title", "Independent subtitle", "Immutable caption"]
            );

            let repeated_changed = frame(641, frames);
            assert!(std::sync::Arc::ptr_eq(&changed, &repeated_changed));

            let styled = source.clone().title(
                crate::Text::new("Root title").style(
                    prepared
                        .root_appearance
                        .typography
                        .root
                        .title
                        .clone()
                        .font_size(27.0)
                        .unwrap(),
                ),
            );
            frames.as_mut().unwrap().prose = styled.description().copy_prose().unwrap();
            let restored = frame(641, frames);
            assert!(!std::sync::Arc::ptr_eq(&changed, &restored));
            assert_eq!(restored.panel_box, changed.panel_box);
            let reset =
                styled.title(crate::Text::new("Root title").layout(crate::TextLayout::default()));
            frames.as_mut().unwrap().prose = reset.description().copy_prose().unwrap();
            let reset_frame = frame(641, frames);
            assert!(!std::sync::Arc::ptr_eq(&restored, &reset_frame));
            let error = reset_frame
                .root_text
                .draw(
                    reset.description().prose(),
                    &mut super::backend::NullBackend::default(),
                )
                .unwrap_err();
            assert!(
                error
                    .to_string()
                    .contains("does not match its configured source"),
                "{error}"
            );
        },
    )
    .unwrap();
}

#[cfg(all(test, unix, not(any(target_os = "macos", target_os = "android"))))]
mod soft_hyphen_tests {
    use super::*;

    #[test]
    #[expect(
        clippy::too_many_lines,
        reason = "checks the production census, sealed pair and both final soft-hyphen renderings"
    )]
    fn production_title_census_prepares_soft_hyphen_for_unwrapped_and_wrapped_svg() {
        const SOURCE: &str = "A\u{00AD}AA";
        for (width, wrapped) in [(800, false), (450, true)] {
            let plot = Plot::scatter(
                crate::NumericValues::new("x", [0.0, 1.0]),
                crate::NumericValues::new("y", [0.0, 1.0]),
            )
            .title(SOURCE)
            .x_label("")
            .y_label("")
            .x_ticks(vec![crate::layout::AxisTick::new(0.0, "A")])
            .y_ticks(vec![crate::layout::AxisTick::new(0.0, "A")])
            .size(width, 1000)
            .erase();
            let mut theme = crate::Theme::publication().resolve().unwrap();
            theme.typography.root.title.font_size = 240.0;
            theme.typography.root.title.weight = crate::TextWeight::Bold;
            assert_ne!(
                theme.typography.root.title.weight,
                theme.typography.axes.tick_label.weight
            );
            let plan = PlotPlan::new_with_theme_factory(
                &plot,
                PlotKind::Scatter,
                || Ok(Box::new(theme)),
                |prepared, replay| {
                    let captured = replay.retained_text();
                    let title_plan = captured
                        .iter()
                        .find(|(_, text)| text.source == SOURCE)
                        .unwrap()
                        .1;
                    assert_eq!(title_plan.lines[0].selected_soft_hyphen.is_some(), wrapped);
                    let census = &prepared.census;
                    assert!(census
                        .requirements
                        .iter()
                        .all(|requirement| requirement.generated_alphabet.is_empty()));
                    let title = census
                        .requirements
                        .iter()
                        .find(|requirement| {
                            requirement.text_role == crate::layout::PreparedTextRole::Title
                        })
                        .unwrap();
                    assert_eq!(title.source, SOURCE);
                    let pair = &census.pairs[title.pair_ordinal as usize];
                    assert!(pair.required_scalars.contains(&'-'));
                    assert!(census
                        .pairs
                        .iter()
                        .filter(|pair| pair.pair_ordinal != title.pair_ordinal)
                        .all(|pair| !pair.required_scalars.contains(&'-')));
                    let environment = prepared.text.prepared_font_environment().unwrap();
                    assert!(environment.prepared_pairs_are_sealed().unwrap());
                    let resolved = environment
                        .prepared_pair(&pair.stack, crate::font::canonical::RequestedWeight::Bold)
                        .unwrap()
                        .unwrap();
                    assert!(resolved.selected_scalars.iter().any(|selected| {
                        selected.scalar == '-'
                            && matches!(
                                selected.selection,
                                crate::font::canonical::ScalarFaceSelection::Face(_)
                            )
                    }));
                },
            )
            .unwrap_or_else(|error| panic!("width {width}: {error}"));
            let mut backend = plan.take_svg_backend().unwrap();
            plan.render(&mut backend).unwrap();
            let svg = backend.into_bytes().unwrap();
            let xml = std::str::from_utf8(&svg).unwrap();
            let document = roxmltree::Document::parse(xml).unwrap();
            let titles = document
                .descendants()
                .filter(|node| {
                    node.has_tag_name("text") && node.attribute("font-size") == Some("240")
                })
                .collect::<Vec<_>>();
            let visible = titles
                .iter()
                .map(|title| {
                    title
                        .descendants()
                        .filter(roxmltree::Node::is_text)
                        .filter_map(|node| node.text())
                        .collect::<String>()
                })
                .collect::<Vec<_>>();
            if wrapped {
                assert_eq!(visible, ["A-", "AA"]);
                assert_eq!(titles[0].attribute("aria-label"), Some("A\u{00AD}"));
            } else {
                assert_eq!(visible, [SOURCE]);
            }
            let retained = titles
                .iter()
                .zip(&visible)
                .map(|(title, text)| title.attribute("aria-label").unwrap_or(text))
                .collect::<String>();
            assert_eq!(retained, SOURCE);
        }
    }
}

#[cfg(test)]
mod natural_sizing_tests {
    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    struct NaturalScene {
        dimensions: (u32, u32),
        svg: Vec<u8>,
    }

    fn natural_scene_with_theme_edit(
        source: &Plot,
        width: Option<u32>,
        edit: impl FnOnce(&mut ResolvedTheme) -> Result<(), Error>,
    ) -> Result<NaturalScene, Error> {
        let source = width.map_or_else(|| source.clone(), |width| source.clone().width(width));
        let plan = PlotPlan::new_with_theme_factory(
            &source,
            source.kind(),
            || {
                let mut theme = crate::Theme::publication().resolve()?;
                edit(&mut theme)?;
                Ok(Box::new(theme))
            },
            |_, _| {},
        )?;
        let dimensions = plan.dimensions();
        let mut backend = plan.take_svg_backend()?;
        plan.render(&mut backend)?;
        Ok(NaturalScene {
            dimensions,
            svg: backend.into_bytes()?,
        })
    }

    fn ordinary_width(theme: &mut ResolvedTheme, width: f32) -> Result<(), Error> {
        let frame = theme.axes.get_frame().expect("preset ordinary frame");
        let changed = crate::theme::FrameTheme::new(
            frame.get_sides().clone(),
            frame.get_stroke().clone().width(width)?,
        )?;
        theme.axes = theme.axes.clone().frame(Some(changed))?;
        Ok(())
    }

    #[test]
    fn matrix_natural_roots_ignore_unused_ordinary_frame_in_dimensions_and_full_svg() {
        let source = Plot::matrix(vec![
            crate::NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
            crate::NumericValues::new("y", [4.0, 3.0, 2.0, 1.0]),
        ])
        .erase();
        for width in [None, Some(800)] {
            let baseline = natural_scene_with_theme_edit(&source, width, |_| Ok(())).unwrap();
            for ordinary in [64.0, f32::MAX] {
                let edited = natural_scene_with_theme_edit(&source, width, |theme| {
                    ordinary_width(theme, ordinary)
                })
                .unwrap();
                assert_eq!(
                    edited, baseline,
                    "width {width:?}, unused ordinary stroke {ordinary}"
                );
            }
            let absent = natural_scene_with_theme_edit(&source, width, |theme| {
                theme.axes = theme.axes.clone().frame(None)?;
                Ok(())
            })
            .unwrap();
            assert_eq!(absent, baseline, "width {width:?}, absent ordinary frame");
            for sides in [
                crate::theme::FrameSides::new(false, false, false, true).unwrap(),
                crate::theme::FrameSides::new(true, true, false, false).unwrap(),
            ] {
                let edited = natural_scene_with_theme_edit(&source, width, |theme| {
                    let frame = theme.axes.get_frame().unwrap().clone().sides(sides)?;
                    theme.axes = theme.axes.clone().frame(Some(frame))?;
                    Ok(())
                })
                .unwrap();
                assert_eq!(
                    edited, baseline,
                    "width {width:?}, unused ordinary side selection"
                );
            }
        }
    }

    #[test]
    fn scatter_natural_roots_consume_the_selected_ordinary_frame() {
        let source = Plot::scatter(
            crate::NumericValues::new("x", [0.0, 1.0, 2.0]),
            crate::NumericValues::new("y", [0.0, 1.0, 2.0]),
        )
        .erase();
        for width in [None, Some(800)] {
            let baseline = natural_scene_with_theme_edit(&source, width, |_| Ok(())).unwrap();
            let wide =
                natural_scene_with_theme_edit(&source, width, |theme| ordinary_width(theme, 64.0))
                    .unwrap();
            assert_ne!(wide.dimensions, baseline.dimensions);
            assert_ne!(wide.svg, baseline.svg);
            assert!(
                natural_scene_with_theme_edit(&source, width, |theme| ordinary_width(
                    theme,
                    f32::MAX
                ))
                .is_err()
            );
        }
    }

    #[test]
    fn matrix_natural_roots_keep_selected_cell_frame_paint_and_measured_ink() {
        let source = Plot::matrix(vec![
            crate::NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
            crate::NumericValues::new("y", [4.0, 3.0, 2.0, 1.0]),
        ])
        .erase();
        for width in [None, Some(800)] {
            let baseline = natural_scene_with_theme_edit(&source, width, |_| Ok(())).unwrap();
            let wide = natural_scene_with_theme_edit(&source, width, |theme| {
                let frame = theme.axes.get_cell_frame().unwrap();
                let changed = crate::theme::FrameTheme::new(
                    frame.get_sides().clone(),
                    frame.get_stroke().clone().width(64.0)?,
                )?;
                theme.axes = theme.axes.clone().cell_frame(Some(changed))?;
                Ok(())
            })
            .unwrap();
            assert_ne!(wide.svg, baseline.svg);
            assert!(std::str::from_utf8(&wide.svg)
                .unwrap()
                .contains("stroke-width=\"64\""));
            assert_ne!(
                wide.dimensions, baseline.dimensions,
                "selected cell ink must reach containing root requirements"
            );
            assert!(natural_scene_with_theme_edit(&source, width, |theme| {
                let frame = theme.axes.get_cell_frame().unwrap();
                let changed = crate::theme::FrameTheme::new(
                    frame.get_sides().clone(),
                    frame.get_stroke().clone().width(f32::MAX)?,
                )?;
                theme.axes = theme.axes.clone().cell_frame(Some(changed))?;
                Ok(())
            })
            .is_err());
        }
    }

    #[test]
    fn content_guide_trials_recover_right_to_top_without_changing_selected_width() {
        let plot = Plot::scatter(
            crate::NumericValues::new("x", [0.0, 1.0]),
            crate::NumericValues::new("y", [1.0, 2.0]),
        )
        .hue(
            crate::CategoryValues::new(
                "series",
                ["extraordinarycategory", "extraordinarycategory"],
            )
            .unwrap(),
        )
        .text_annotation(
            crate::PlotTextAnnotation::at(
                crate::TextPosition::new(
                    crate::TextCoordinate::figure_fraction(0.5).unwrap(),
                    crate::TextCoordinate::figure_fraction(0.5).unwrap(),
                ),
                crate::Text::new("Initial Figure")
                    .layout(crate::TextLayout::default().anchor(crate::TextAnchor::center())),
            )
            .unwrap(),
        )
        .width(300)
        .erase();
        crate::orchestrator::measurements::start();
        let plan = crate::layout::plan_root(
            plot.description().sizing_request(),
            |stage| Root::prepare(RootSource::Plot(plot.description()), stage),
            Root::measure,
            |root, size, plan, stage| {
                let MeasuredRootContent::Single { plot, .. } = &plan.content else {
                    panic!("selected standalone plot")
                };
                let dimensions = plot.selected.dimensions;
                let figure = dimensions.figure_size();
                assert_eq!(dimensions.output_override, None);
                assert_eq!(figure, plan.measurement.measurement_size);
                assert_eq!(plan.size, size);
                assert_ne!(figure, size, "the top guide completes a different height");

                let note = plot
                    .result
                    .panel_text
                    .iter()
                    .find(|note| note.text_plan.source == "Initial Figure")
                    .expect("actual Figure-reference consumer");
                let point = note.retained_placement().placement.position;
                let (width, height) = figure.dimensions()?;
                assert!(
                    (point.x + note.text_plan.width() / 2.0 - f64::from(width) / 2.0).abs()
                        <= 1.0 / 1024.0
                );
                assert!((point.y - f64::from(height) / 2.0).abs() <= 1.0 / 1024.0);
                assert!((point.y - f64::from(size.dimensions()?.1) / 2.0).abs() > 1.0 / 1024.0);
                let verifier = root.sealed_verifier()?;
                capture_selected(root, plan, &verifier, stage)
            },
        )
        .unwrap();
        let (width, height) = plan.dimensions();
        validate_text_replay_plan(
            width,
            height,
            plan.plan(),
            "content guide Figure reference Null replay",
        )
        .unwrap();
        let observed = crate::orchestrator::measurements::finish();
        assert_eq!(width, 300);
        assert!(observed
            .guides
            .iter()
            .any(|trial| trial.edge == crate::LegendPosition::OutsideRight
                && trial.outcome == crate::orchestrator::measurements::GuideOutcome::Rejected));
        assert!(observed
            .guides
            .iter()
            .any(|trial| trial.edge == crate::LegendPosition::OutsideTop
                && trial.outcome == crate::orchestrator::measurements::GuideOutcome::Accepted));
    }

    #[test]
    fn natural_prose_metrics_use_the_prepared_role_specific_font_pair() {
        let plot = Plot::scatter(
            crate::NumericValues::new("x", [1.0, 2.0]),
            crate::NumericValues::new("y", [3.0, 4.0]),
        )
        .title("A complete title with independently bold typography and legal boundaries")
        .erase();
        let mut theme = crate::Theme::publication().resolve().unwrap();
        theme.typography.root.title.weight = crate::TextWeight::Bold;
        PlotPlan::new_with_theme_factory(
            &plot,
            plot.kind(),
            || Ok(Box::new(theme)),
            |prepared, _| {
                assert!(prepared
                    .census
                    .requirements
                    .iter()
                    .any(|entry| entry.text_role
                        == crate::layout::PreparedTextRole::PreferenceMetric
                        && entry.weight == crate::TextWeight::Bold
                        && entry.source == "0"));
            },
        )
        .unwrap();
    }

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

    #[test]
    fn free_waterfall_facets_measure_shelves_at_each_consumed_numeric_domain() {
        let source = Plot::waterfall(
            crate::CategoryValues::new(
                "term",
                ["first", "second", "third", "first", "second", "third"],
            )
            .unwrap(),
            crate::NumericValues::new("delta", [1.0, 2.0, 3.0, 100.0, 200.0, 300.0]),
            10.0,
        )
        .total_format(
            crate::NumericFormat::new(
                crate::NumericNotation::Decimal,
                crate::NumericPrecision::AdaptiveFractional { min: 0, max: 2 },
            )
            .unwrap()
            .suffix(" u")
            .unwrap(),
        )
        .facet(
            crate::FacetSpec::column(
                crate::CategoryValues::new(
                    "panel",
                    ["narrow", "narrow", "narrow", "wide", "wide", "wide"],
                )
                .unwrap(),
            )
            .scales(crate::FacetScales::Free),
        )
        .caption("Complete explanatory caption for two independently scaled shelves.")
        .erase();
        for width in [None, Some(800)] {
            let source = width.map_or_else(|| source.clone(), |width| source.clone().width(width));
            let plan = PlotPlan::new(&source, source.kind()).unwrap();
            if let Some(width) = width {
                assert_eq!(plan.dimensions().0, width);
            }
            assert!(plan
                .root
                .plan()
                .retained_text()
                .iter()
                .any(|(_, text)| text.source == source.description().caption_text().unwrap()));
            for source in ["base 10 u", "total 16 u", "total 610 u"] {
                assert!(plan
                    .root
                    .plan()
                    .retained_text()
                    .iter()
                    .any(|(_, text)| text.source == source));
            }
        }
    }

    #[test]
    fn owning_continuous_facets_compose_each_scoped_guide_at_the_chosen_width() {
        let source = Plot::beeswarm(
            crate::CategoryValues::new("group", ["a", "a", "b", "b", "a", "a", "b", "b"]).unwrap(),
            crate::NumericValues::new("x", [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]),
            crate::NumericValues::new("color", [0.0, 1.0, 0.0, 1.0, 100.0, 200.0, 100.0, 200.0]),
        )
        .facet(
            crate::FacetSpec::column(
                crate::CategoryValues::new(
                    "panel",
                    [
                        "first", "first", "first", "first", "second", "second", "second", "second",
                    ],
                )
                .unwrap(),
            )
            .scales(crate::FacetScales::Free),
        )
        .caption("A complete explanatory caption for independently scoped continuous guides.")
        .erase();
        for width in [None, Some(1600)] {
            let source = width.map_or_else(|| source.clone(), |width| source.clone().width(width));
            let plan = PlotPlan::new(&source, source.kind()).unwrap();
            if let Some(width) = width {
                assert_eq!(plan.dimensions().0, width);
            }
            let titles = plan
                .root
                .plan()
                .retained_text()
                .iter()
                .filter(|(_, text)| text.source == "Color")
                .count();
            assert_eq!(titles, 2);
        }
    }

    #[test]
    fn nullable_heatmap_facets_keep_independent_guide_domains_and_missing_blocks() {
        let source = Plot::heatmap(crate::CategoryValues::new("x", ["a", "b", "a", "b", "a", "b", "a", "b"]).unwrap(),
            crate::CategoryValues::new("y", ["first", "first", "second", "second", "first", "first", "second", "second"]).unwrap(),
            crate::NumericValues::nullable("color response with independently measured legal word boundaries", [Some(1.01), None, Some(1.02), Some(1.03), Some(100.1), None, Some(250.2), Some(350.3)]))
            .annotations(crate::HeatmapAnnotations::Values(crate::NumericFormat::new(crate::NumericNotation::Decimal, crate::NumericPrecision::AdaptiveFractional { min: 0, max: 2 }).unwrap().prefix("v=").unwrap()))
            .color_label_format(crate::NumericFormat::default().prefix("g=").unwrap())
            .facet(crate::FacetSpec::column(crate::CategoryValues::new("panel", ["first", "first", "first", "first", "second", "second", "second", "second"]).unwrap()).scales(crate::FacetScales::Fixed))
            .caption("First complete explanatory caption paragraph.\nSecond complete explanatory caption paragraph.").erase();
        for width in [None, Some(1200)] {
            let source = width.map_or_else(|| source.clone(), |width| source.clone().width(width));
            let plan = PlotPlan::new(&source, source.kind()).unwrap();
            if let Some(width) = width {
                assert_eq!(plan.dimensions().0, width);
            }
            let retained = plan.root.plan().retained_text();
            assert_eq!(
                retained
                    .iter()
                    .filter(|(_, text)| text.source == "Missing")
                    .count(),
                2
            );
            for label in [
                "g=1.01", "g=1.03", "g=100.10", "g=350.30", "v=1.01", "v=1.02", "v=1.03", "v=100",
                "v=250", "v=350",
            ] {
                assert!(
                    retained.iter().any(|(_, text)| text.source == label),
                    "missing scoped endpoint {label}"
                );
            }
            assert!(!retained.iter().any(|(_, text)| text.source == "v=100.10"));
        }
    }

    #[test]
    fn owning_facet_discrete_guide_preference_is_composed_once_at_its_explicit_edge() {
        let source = Plot::scatter(
            crate::NumericValues::new("x", [0.0, 1.0, 2.0, 3.0]),
            crate::NumericValues::new("y", [1.0, 2.0, 3.0, 4.0]),
        )
        .hue(crate::CategoryValues::new("series", ["one", "two", "one", "two"]).unwrap())
        .facet(crate::FacetSpec::column(
            crate::CategoryValues::new("panel", ["first", "first", "second", "second"]).unwrap(),
        ));
        let mut widths = Vec::new();
        for policy in [
            crate::LegendPolicy::Hidden,
            crate::LegendPolicy::Visible(crate::LegendPosition::OutsideTop),
            crate::LegendPolicy::Visible(crate::LegendPosition::OutsideRight),
        ] {
            let plot = source.clone().legend(policy).erase();
            widths.push(PlotPlan::new(&plot, plot.kind()).unwrap().dimensions().0);
        }
        assert_eq!(widths[0], widths[1]);
        assert!(widths[2] > widths[0]);
        assert!(
            widths[2] - widths[0] < 150,
            "one shared short-entry guide, not one per facet: {widths:?}"
        );
    }
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

    #[cfg(test)]
    pub(super) fn new_with_resolved_theme(
        source: &Plot,
        kind: PlotKind,
        theme: ResolvedTheme,
    ) -> Result<Self, Error> {
        Self::new_with_theme_factory(source, kind, || Ok(Box::new(theme)), |_, _| {})
    }

    fn new_with_theme_factory(
        source: &Plot,
        _kind: PlotKind,
        resolve_theme: impl FnOnce() -> Result<Box<ResolvedTheme>, Error>,
        inspect: impl FnOnce(&Root<'_>, &TextReplayPlan),
    ) -> Result<Self, Error> {
        let sizing = source.description().sizing_request();
        #[cfg(test)]
        super::PLOT_PLAN_CALLS.with(|calls| calls.set(calls.get() + 1));
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
    pub(crate) fn intrinsic_for_test(
        &self,
        theme: &TextRequestContext<'_, '_>,
    ) -> Result<(f64, f64), Error> {
        with_prepared_plot_for_test(
            self,
            theme,
            |_| Ok(()),
            |_plot, inputs, request, operation, _round| {
                crate::layout::interior::intrinsic_body_minimum(
                    &inputs.plots[0],
                    crate::builder::RootFrameDomainInput::Prepared,
                    inputs.plot_intrinsic(&inputs.plots[0].scope.id).unwrap(),
                    operation,
                    request,
                )
            },
        )?
    }

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
mod root_title_tests {
    use super::*;
    use crate::TextAlignment::{Center, End, Start};

    #[test]
    fn specialized_axis_feedback_cannot_spend_the_root_caption_reserve() {
        let source = Plot::beeswarm(
            crate::CategoryValues::new("group", ["a", "a", "b", "b"]).unwrap(),
            crate::NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
            crate::NumericValues::new("color", [0.0, 1.0, 0.0, 1.0]),
        )
        .x_label("First axis line\nSecond axis line\nThird axis line")
        .size(800, 600)
        .erase();
        let mut reserves = Vec::new();
        for caption in ["", "Independent caption\nSecond caption line"] {
            let source = source.clone().caption(caption);
            let root = crate::layout::plan_root(
                crate::layout::RootRequest::ExactSize {
                    width: 800,
                    height: 600,
                },
                |stage| Root::prepare(RootSource::Plot(source.description()), stage),
                Root::measure,
                |prepared, _, plan, _| {
                    let theme = prepared.request();
                    let sizes =
                        crate::decorations::title_block::plan::RootTitleBlockPlan::appearances(
                            &theme,
                            source.description().prose(),
                        )
                        .map(|role| role.theme.get_font_size());
                    let clearance = crate::theme::composition::hard(
                        &theme
                            .root_title_lengths(sizes)?
                            .caption_to_content_clearance,
                    )?;
                    Ok((plan, clearance))
                },
            )
            .unwrap();
            let (plan, caption_clearance) = root.plan();
            let MeasuredRootContent::Single { plot, .. } = &plan.content else {
                panic!("measured single plot")
            };
            let body = plan.frame.panel_box;
            let panel = plot.panel_box;
            reserves.push((panel.y0 - body.y0).to_bits());
            assert!(
                panel.y0 > body.y0,
                "multiline axis requires its own extra gutter"
            );
            let required = plot.result.axes.as_ref().unwrap().requirements();
            let axis_ink_bottom = panel.y0 - required.bottom;
            let caption_top = if caption.is_empty() {
                0.0
            } else {
                8.0 + crate::theme::composition::hard(&plan.frame.root_text.reserves.bottom)
                    .unwrap()
                    - caption_clearance
            };
            assert!(
                axis_ink_bottom >= caption_top,
                "caption={caption:?}, body={body:?}, panel={panel:?}, required={required:?}, caption_top={caption_top}, axis_bottom={axis_ink_bottom}"
            );
        }
        assert_eq!(reserves[0], reserves[1]);
    }

    #[test]
    fn standalone_root_freezes_independent_alignment_before_null_and_svg() {
        for preset in [
            crate::Theme::publication(),
            crate::Theme::nature(),
            crate::Theme::science(),
            crate::Theme::chi(),
        ] {
            let composition = preset
                .composition()
                .clone()
                .root_caption_alignment(Center)
                .unwrap();
            let theme = preset.customize().composition(composition).build().unwrap();
            let plot = Plot::scatter(
                crate::NumericValues::new("x", [0.0, 1.0]),
                crate::NumericValues::new("y", [0.0, 1.0]),
            )
            .title("Title\nx")
            .subtitle("Subtitle\ny")
            .caption("Caption\nz")
            .size(333, 481)
            .theme(theme)
            .erase()
            .title_alignment(End)
            .subtitle_alignment(Start);
            let theme = plot.description().resolve_theme().unwrap();
            let plan = PlotPlan::new_with_resolved_theme(&plot, PlotKind::Scatter, theme).unwrap();
            let retained = plan.root.plan().retained_text();
            for (source, alignment) in [
                ("Title\nx", crate::layout::PhysicalTextAlignment::End),
                ("Subtitle\ny", crate::layout::PhysicalTextAlignment::Start),
                ("Caption\nz", crate::layout::PhysicalTextAlignment::Center),
            ] {
                let roles = retained
                    .iter()
                    .filter(|(_, text)| text.source == source)
                    .collect::<Vec<_>>();
                assert_eq!(roles.len(), 1);
                let (placement, text) = roles[0];
                assert_eq!(text.style.alignment, alignment);
                assert_eq!(text.lines.len(), 2);
                let anchor = match alignment {
                    crate::layout::PhysicalTextAlignment::Start => 0,
                    crate::layout::PhysicalTextAlignment::Center => {
                        crate::render::backend::nearest_even_text_half(text.width_quanta).unwrap()
                    }
                    crate::layout::PhysicalTextAlignment::End => text.width_quanta,
                };
                for (line, geometry) in text.lines.iter().zip(&placement.0.lines) {
                    assert_eq!(
                        geometry.origin.x.units(),
                        placement.0.center.x.units() - anchor + line.translation_x_quanta
                    );
                }
            }
            let mut backend = plan.take_svg_backend().unwrap();
            plan.render(&mut backend).unwrap();
            let svg = backend.into_bytes().unwrap();
            let xml = std::str::from_utf8(&svg).unwrap();
            let document = roxmltree::Document::parse(xml).unwrap();
            for source in ["Title", "x", "Subtitle", "y", "Caption", "z"] {
                assert!(document
                    .descendants()
                    .any(|node| node.has_tag_name("text") && node.text() == Some(source)));
            }
        }
    }
}

#[cfg(test)]
mod retained_legend_tests {
    use super::*;
    use crate::orchestrator::measurements::{self, Counter};

    #[test]
    fn standalone_capture_uses_selected_legend_operands_not_measured_style_or_position() {
        let source = Plot::scatter(
            crate::NumericValues::new("x", [0.0, 1.0, 2.0, 3.0]),
            crate::NumericValues::new("y", [1.0, 2.0, 1.5, 2.5]),
        )
        .hue(crate::CategoryValues::new("group", ["first", "first", "second", "second"]).unwrap())
        .size(800, 600)
        .legend(crate::LegendPolicy::Visible(
            crate::LegendPosition::OutsideRight,
        ))
        .erase();
        measurements::start();
        let root = crate::layout::plan_root(
            crate::layout::RootRequest::ExactSize {
                width: 800,
                height: 600,
            },
            |stage| Root::prepare(RootSource::Plot(source.description()), stage),
            Root::measure,
            |prepared, _, mut plan, stage| {
                let theme = prepared.request();
                let MeasuredRootContent::Single { plot, legend } = &mut plan.content else {
                    panic!("measured single plot")
                };
                assert_eq!(plot.plot_id, prepared.plots[0].scope.id);
                assert_eq!(plot.selected.plot_id, plot.plot_id);
                let (layer, draw) = legend.as_ref().unwrap();
                assert_eq!(layer, "legend");
                let mut geometry = crate::render::geometry::GeometrySnapshot::new();
                draw.geometry(&mut geometry)?;
                let mut backend = crate::render::geometry::GeometryBackend::new(&theme)?;
                backend.begin(800, 600)?;
                draw.draw(&mut backend)?;
                backend.end()?;
                assert_eq!(geometry.bounds(), backend.bounds());
                let measured = std::sync::Arc::get_mut(plot)
                    .unwrap()
                    .legend
                    .as_mut()
                    .unwrap();
                std::sync::Arc::make_mut(&mut measured.legend.metrics).font_size = f64::NAN;
                measured.bounds = measured.bounds.translated(10_000.0, 10_000.0);
                let verifier = prepared.sealed_verifier()?;
                capture_calls(plan.size, &verifier, stage, |backend| {
                    render_single_plot(
                        source.description(),
                        source.kind(),
                        plot,
                        legend.as_ref(),
                        &plan.frame,
                        backend,
                    )
                })
            },
        )
        .unwrap();
        let work = measurements::finish();
        assert!(work.counters[Counter::LegendSelection as usize] > 0);
        assert_eq!(work.capture_counters[Counter::LegendSelection as usize], 0);
        assert!(root
            .plan()
            .retained_text()
            .iter()
            .any(|(_, text)| text.source == "First"));
        let shapes = crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get);
        measurements::start();
        validate_text_replay_plan(800, 600, root.plan(), "retained standalone legend Null")
            .unwrap();
        let mut svg = root.plan().take_svg_backend().unwrap();
        root.plan()
            .replay(800, 600, &mut svg, "retained standalone legend SVG")
            .unwrap();
        assert!(!svg.into_bytes().unwrap().is_empty());
        assert_eq!(measurements::finish().counters, [0; Counter::ALL.len()]);
        assert_eq!(
            shapes,
            crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get)
        );
    }
}
