//! One-canvas chart lifecycle and immutable standalone scene planning.

use crate::Error;

use super::backend::{capture_text_replay_plan, validate_text_replay_plan, TextReplayPlan};
use super::plan::{MeasuredOwnedPlan, MeasuredRootContent};
use super::root::{CachedRootFrame, RootLayoutCaches};
use crate::builder::Plot;
use crate::decorations::plan::MeasuredPlotPlan;
use crate::layout::PixelRect;
use crate::render::backend::{Backend, SurfaceRole};
use crate::spec::{DecorationMode, PlotKind, PlotRenderContext, PlotRenderResult};
use crate::text::{TextRequestContext, TextService};
use crate::theme::ResolvedTheme;

pub(super) struct PlotPlan {
    root: crate::layout::FrozenRoot<TextReplayPlan>,
}

struct PreparedPlotInputs<'src> {
    operation: crate::font::canonical::OperationContext,
    axes: crate::layout::interior::PreparedInteriorInputs<'src>,
    facet_guide: Option<super::facets::PreparedFacetGuide>,
    theme: Box<ResolvedTheme>,
    text: TextService,
    text_census: crate::layout::PreparedTextCensus,
    root_text: crate::decorations::title_block::plan::RootTitleBlockIntrinsics,
    caches: std::cell::RefCell<RootLayoutCaches>,
}

impl<'src> PreparedPlotInputs<'src> {
    fn prepare(
        source: &'src Plot,
        kind: PlotKind,
        resolve_theme: impl FnOnce() -> Result<Box<ResolvedTheme>, Error>,
        caches: std::cell::RefCell<RootLayoutCaches>,
        prepared_stage: &mut crate::layout::kernel::PreparedStage<'_>,
    ) -> Result<Self, Error> {
        let theme = resolve_theme()?;
        let text = TextService::default();
        let request = TextRequestContext::new(&theme, &text);
        source.description().validate_configuration(kind)?;
        let mut axes = crate::plots::shared_x::prepare_plot(
            source,
            &prepared_stage.operation(),
            Some(&theme),
        )?;
        axes.validate_categorical_scale(source, &request, None)?;
        let facet_guide = source
            .description()
            .has_facets()
            .then(|| {
                super::facets::prepare_discrete_guide(
                    source,
                    kind,
                    &request,
                    &mut axes,
                    &prepared_stage.operation(),
                )
            })
            .transpose()?;
        if !source.description().has_facets() {
            axes.prepare_discrete_guides(
                source,
                &request,
                &prepared_stage.operation(),
                &crate::layout::ComponentAddress::root()
                    .child(crate::layout::AddressRole::PlotBody, 0)?,
                false,
            )?;
        }
        let mut requirements = crate::layout::foundational_plot_requirements(
            Some(&axes),
            source,
            kind,
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
        let text_topology =
            crate::layout::PreparedTextCensus::structural_address_count(&requirements)
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
        axes.prepare_intrinsics(source, &request, &prepared_stage.operation())?;
        let root_text =
            crate::decorations::title_block::plan::RootTitleBlockPlan::intrinsic_widths(
                &request,
                source.description().prose(),
                None,
            )?;
        Ok(Self {
            operation: prepared_stage.operation(),
            axes,
            facet_guide,
            theme,
            text,
            text_census: census,
            root_text,
            caches,
        })
    }
}

/// Keep producer fixtures inside the production prepare/font/measure operation.
/// Only owning results, scalar observations and liveness witnesses may escape.
#[cfg(test)]
pub(crate) fn with_prepared_plot_for_test<T>(
    source: &Plot,
    appearance: &TextRequestContext<'_, '_>,
    visit: impl FnOnce(
        &Plot,
        &crate::layout::interior::PreparedInteriorInputs<'_>,
        &TextRequestContext<'_, '_>,
        &crate::font::canonical::OperationContext,
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
            PreparedPlotInputs::prepare(
                source,
                source.kind(),
                || Ok(Box::new(appearance.theme.clone())),
                std::cell::RefCell::new(RootLayoutCaches::new(
                    crate::decorations::plan::children::ChildPlans::facets(),
                )),
                stage,
            )
        },
        |prepared, assigned, _| {
            let (width, height) = assigned.assigned_size()?.dimensions()?;
            let plot = prepared
                .axes
                .attach(source.clone())?
                .with_canvas_size(width, height);
            let request = TextRequestContext::new(&prepared.theme, &prepared.text);
            result = Some(visit.take().expect("one feedback-free producer fixture")(
                &plot,
                &prepared.axes,
                &request,
                &prepared.operation,
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

impl PreparedPlotInputs<'_> {
    /// This prepared owner freezes its source, theme, fonts and instructions for
    /// one operation. Only root constraints and traced body minima can change.
    fn root_frame(
        &self,
        source: &Plot,
        frames: &mut Option<CachedRootFrame>,
    ) -> Result<std::sync::Arc<crate::layout::RootFramePlan>, Error> {
        let dimensions = source.canvas_dimensions()?;
        let request = TextRequestContext::new(&self.theme, &self.text);
        let intrinsic = self.axes.intrinsic(&self.axes.plots[0]);
        let minimum = source.intrinsic_body_minimum(
            self.axes.plots.first(),
            intrinsic,
            &self.operation,
            &request,
        )?;
        let minimum = [minimum.0.to_bits(), minimum.1.to_bits()];

        if let Some(cached) = frames.as_ref() {
            if cached.dimensions == dimensions
                && cached.minimum == minimum
                && source.description().same_prose(&cached.prose)
            {
                return Ok(std::sync::Arc::clone(&cached.frame));
            }
        }
        let frame = source.measured_canvas_layout(
            self.axes.plots.first(),
            intrinsic,
            &self.operation,
            &request,
        )?;

        let frame = frame.into_shared();
        *frames = Some(CachedRootFrame {
            prose: source.description().copy_prose()?,
            dimensions,
            minimum,
            frame: std::sync::Arc::clone(&frame),
        });
        Ok(frame)
    }
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
            let source = prepared.axes.attach(source.clone()).unwrap();

            let mut caches = prepared.caches.borrow_mut();
            let frames = &mut caches.frames;
            frames.take();
            let original = prepared.root_frame(&source, frames).unwrap();
            let shapes = crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get);

            let repeated = prepared.root_frame(&source, frames).unwrap();
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
            let changed = prepared
                .root_frame(&source.clone().with_canvas_size(641, 480), frames)
                .unwrap();
            assert_ne!(changed.panel_box, repeated.panel_box);

            assert_eq!(
                changed
                    .root_text
                    .placed_text()
                    .map(|(_, text)| text.source.as_str())
                    .collect::<Vec<_>>(),
                ["Root title", "Independent subtitle", "Immutable caption"]
            );

            let repeated_changed = prepared
                .root_frame(&source.clone().with_canvas_size(641, 480), frames)
                .unwrap();
            assert!(std::sync::Arc::ptr_eq(&changed, &repeated_changed));

            let styled = source.clone().title(
                crate::Text::new("Root title").style(
                    prepared
                        .theme
                        .typography
                        .root
                        .title
                        .clone()
                        .font_size(27.0)
                        .unwrap(),
                ),
            );
            let styled_frame = prepared.root_frame(&styled, frames).unwrap();
            assert!(!std::sync::Arc::ptr_eq(&original, &styled_frame));
            assert!(styled_frame.insets.top > original.insets.top);
            let repeated_style = prepared.root_frame(&styled, frames).unwrap();
            assert!(std::sync::Arc::ptr_eq(&styled_frame, &repeated_style));
            let reset =
                styled.title(crate::Text::new("Root title").layout(crate::TextLayout::default()));
            let reset_frame = prepared.root_frame(&reset, frames).unwrap();
            assert!(!std::sync::Arc::ptr_eq(&styled_frame, &reset_frame));
            assert_eq!(styled_frame.panel_box, reset_frame.panel_box);
            let error = styled_frame
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
                    let census = &prepared.text_census;
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
        .width(300)
        .erase();
        crate::orchestrator::measurements::start();
        let plan = PlotPlan::new(&plot, plot.kind()).unwrap();
        let observed = crate::orchestrator::measurements::finish();
        assert_eq!(plan.dimensions().0, 300);
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
                    .text_census
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

    #[expect(
        clippy::too_many_lines,
        reason = "ordered axis preparation, font preparation, measurement, and capture lifecycle"
    )]
    fn new_with_theme_factory(
        source: &Plot,
        kind: PlotKind,
        resolve_theme: impl FnOnce() -> Result<Box<ResolvedTheme>, Error>,
        #[cfg(test)] inspect: impl FnOnce(&PreparedPlotInputs, &TextReplayPlan),
    ) -> Result<Self, Error> {
        let sizing = source.description().sizing_request();
        #[cfg(test)]
        super::PLOT_PLAN_CALLS.with(|calls| calls.set(calls.get() + 1));
        let caches = std::cell::RefCell::new(RootLayoutCaches::new(
            crate::decorations::plan::children::ChildPlans::facets(),
        ));
        let root = crate::layout::plan_root(
            sizing,
            |prepared_stage| {
                PreparedPlotInputs::prepare(source, kind, resolve_theme, caches, prepared_stage)
            },
            |prepared, assigned, round| {
                let mut caches = prepared.caches.borrow_mut();
                let RootLayoutCaches {
                    children, frames, ..
                } = &mut *caches;
                let request = TextRequestContext::new(&prepared.theme, &prepared.text);
                let assigned =
                    if let crate::layout::RootRequest::ExactSize { width, height } = assigned {
                        crate::layout::RootSize::new(width, height)
                    } else {
                        let attached = prepared.axes.attach(source.clone())?;
                        if source.description().has_facets() {
                            super::facets::natural_size(
                                &attached,
                                &prepared.operation,
                                &request,
                                assigned,
                                round,
                                prepared.facet_guide.as_ref().expect("prepared facet guide"),
                                &prepared.axes,
                            )?
                        } else {
                            crate::layout::interior::sizing::plot_size(
                                &attached,
                                &prepared.axes,
                                &prepared.operation,
                                &request,
                                assigned,
                                round,
                            )?
                        }
                    };
                let (width, height) = assigned.dimensions()?;
                prepared
                    .root_text
                    .check_width(
                        prepared.theme.root_inner_width(f64::from(width))?,
                        &crate::layout::fit::FitScope::root(),
                    )
                    .map_err(crate::layout::fit::PlanError::into_error)?;

                let source = prepared
                    .axes
                    .attach(source.clone())?
                    .with_canvas_size(width, height);
                let plan = if source.description().has_facets() {
                    super::facets::OwnedFacetPlan::measure(
                        &source,
                        &prepared.operation,
                        kind,
                        &request,
                        round,
                        children,
                        prepared
                            .facet_guide
                            .as_ref()
                            .expect("faceted root guide was prepared"),
                        sizing.automatic_height(),
                        &prepared.axes,
                    )
                    .map_err(crate::layout::fit::PlanError::into_error)?
                } else {
                    measure_single_plot(
                        &source,
                        prepared.axes.plots.first(),
                        prepared.axes.intrinsic(&prepared.axes.plots[0]),
                        kind,
                        &request,
                        prepared.root_frame(&source, frames)?,
                        round,
                        sizing.automatic_height(),
                    )?
                };
                children.finish_round(round);
                let assigned = plan.size;
                Ok(crate::layout::MeasuredLayout::new(assigned, plan))
            },
            |prepared, assigned, layout, final_stage| {
                let request = TextRequestContext::new(&prepared.theme, &prepared.text);
                let _ = &prepared.text_census;
                let (width, height) = assigned.dimensions()?;
                #[cfg(test)]
                let fonts = crate::font::canonical::planning_counts();
                let replay = capture_text_replay_plan(
                    width,
                    height,
                    "prepared plot scene capture",
                    &request,
                    final_stage,
                    |backend| render_plot_scene(source, kind, width, height, &layout, backend),
                )?;
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
    source: &Plot,
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

#[expect(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    reason = "one selected standalone panel retains its original frame completion and legend checks"
)]
fn measure_single_plot(
    source: &Plot,
    registered: Option<&crate::orchestrator::registration::PlotRegistration<'_>>,
    intrinsic: &crate::orchestrator::root::PlotIntrinsics,
    kind: PlotKind,
    theme: &TextRequestContext<'_, '_>,
    mut canvas_layout: std::sync::Arc<crate::layout::RootFramePlan>,
    round: &mut crate::layout::kernel::FeedbackRound,
    content_height: bool,
) -> Result<MeasuredOwnedPlan, Error> {
    let cell = canvas_layout.panel_box;
    source.description().configured_source_rows()?;
    let operation = theme.text.prepared_font_environment()?.operation();
    let mut context = PlotRenderContext::new(
        registered
            .expect("prepared standalone plot")
            .data(&operation)?,
        cell,
        theme,
    )
    .with_registration(registered.expect("prepared standalone plot"), intrinsic);
    context.content_height = content_height;
    source.validate_interior_preparation(registered)?;
    let discrete_guide = if kind == PlotKind::Matrix {
        None
    } else {
        source
            .prepared_discrete_guide(registered)?
            .map(|descriptor| {
                crate::decorations::legend::discrete::GuideSlot::new(
                    descriptor,
                    &crate::layout::ComponentAddress::root()
                        .child(crate::layout::AddressRole::PlotBody, 0)?,
                    source.description().legend_position_spec(),
                )
            })
            .transpose()?
    };
    let panel_plan =
        crate::layout::fit::measured(crate::decorations::plan::plan_panel_with_context(
            source,
            kind,
            crate::decorations::plan::LegendRole::Panel,
            context,
            discrete_guide.as_ref(),
            round,
            &crate::layout::ComponentAddress::root()
                .child(crate::layout::AddressRole::PlotBody, 0)?,
        ))
        .map_err(crate::layout::fit::PlanError::into_error)?;

    let mut plot = panel_plan.plot_with_planned_limits(source);
    if content_height {
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
        plot = plot.with_canvas_size(
            panel_plan.selected.dimensions.canvas_size().dimensions()?.0,
            height,
        );
        let mut resized = plot.measured_canvas_layout(registered, intrinsic, &operation, theme)?;
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
                    crate::decorations::legend::address_role(
                        source.description().legend_position_spec(),
                    ),
                    0,
                )?;
            let (width, height) = plot.canvas_dimensions()?;
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
            let length = super::registration::render_name(registered)
                .map_or(Some(6), |scope| scope.len().checked_add(7))
                .ok_or_else(|| Error::Render("standalone legend layer length overflow".into()))?;
            crate::data::source::reserve_source_string(&mut layer, length)?;
            if let Some(scope) = super::registration::render_name(registered) {
                layer.push_str(scope);
                layer.push('-');
            }
            layer.push_str("legend");
            Ok::<_, Error>((layer, draw))
        })
        .transpose()?;
    let (width, height) = plot.canvas_dimensions()?;
    Ok(MeasuredOwnedPlan {
        size: crate::layout::RootSize::new(width, height),
        frame: canvas_layout,
        root_paint: theme.surfaces.canvas_root,
        content: MeasuredRootContent::Single {
            plot: std::sync::Arc::new(panel_plan),
            legend,
        },
    })
}

fn render_single_plot(
    source: &Plot,
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
    source.description().configured_source_rows()?;
    crate::layout::interior::InteriorRoute::classify(kind).validate(source.description())?;
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
    frame
        .root_text
        .draw(source.description().prose(), backend)?;
    debug_assert_eq!(rendered, plot.result.decoration_mode);
    Ok(())
}
impl Plot {
    #[cfg(test)]
    pub(crate) fn intrinsic_for_test(
        &self,
        theme: &TextRequestContext<'_, '_>,
    ) -> Result<(f64, f64), Error> {
        with_prepared_plot_for_test(self, theme, |plot, inputs, request, operation| {
            plot.intrinsic_body_minimum(
                inputs.plots.first(),
                inputs.intrinsic(&inputs.plots[0]),
                operation,
                request,
            )
        })?
    }

    #[cfg(test)]
    pub(super) fn render_kind_erased(
        &self,
        kind: PlotKind,
        backend: &mut dyn Backend,
    ) -> Result<(), Error> {
        PlotPlan::new(self, kind)?.render(backend)
    }

    #[cfg(test)]
    pub(crate) fn preflight_panel(
        &self,
        kind: PlotKind,
        panel: crate::layout::PixelRect,
        theme: &TextRequestContext<'_, '_>,
    ) -> Result<PlotRenderResult, Error> {
        with_prepared_plot_for_test(self, theme, |plot, inputs, request, _operation| {
            crate::layout::fit::measured(plot.preflight_measurement(
                inputs.plots.first(),
                Some(inputs.intrinsic(&inputs.plots[0])),
                None,
                kind,
                panel,
                request,
                &crate::layout::fit::ROOT_SCOPE,
            ))
            .map_err(crate::layout::fit::PlanError::into_error)
        })?
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "the probe borrows its occurrence, intrinsic slot and containing Canvas without storing owner data in Plot"
    )]
    pub(crate) fn preflight_measurement(
        &self,
        registered: Option<&crate::orchestrator::registration::PlotRegistration<'_>>,
        intrinsic: Option<&crate::orchestrator::root::PlotIntrinsics>,
        canvas: Option<&crate::Canvas>,
        kind: PlotKind,
        panel: crate::layout::PixelRect,
        theme: &TextRequestContext<'_, '_>,
        scope: &crate::layout::fit::FitScope,
    ) -> Result<crate::layout::fit::FitOutcome<PlotRenderResult>, Error> {
        crate::layout::interior::InteriorRoute::classify(kind).validate(self.description())?;
        self.description()
            .validate_category_order_configuration(kind)?;
        if !self.description().significance_annotations().is_empty()
            && !kind.supports_significance_annotations()
        {
            return Err(Error::Data(format!(
                "significance annotations are not supported for {kind:?}; use a supported categorical family or remove the annotations"
            )));
        }
        self.description().configured_source_rows()?;
        let operation = theme.text.prepared_font_environment()?.operation();
        let registered = registered.expect("prepared plot measurement");
        let mut context = PlotRenderContext::new(registered.data(&operation)?, panel, theme)
            .with_registration(registered, intrinsic.expect("prepared plot intrinsic slot"))
            .with_fit_scope(scope);
        context.canvas = canvas;
        crate::layout::fit::outcome(self.measure_panel(kind, context))
    }
}

#[cfg(test)]
mod root_title_tests {
    use super::*;
    use crate::TextAlignment::{Center, End, Start};

    #[test]
    #[expect(
        clippy::too_many_lines,
        reason = "the same root operation prepares fonts and measures independent caption and native-axis reserves"
    )]
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
            let theme = crate::Theme::publication().resolve().unwrap();
            let text = crate::text::TextService::default();
            let theme = crate::text::TextRequestContext::new(&theme, &text);
            let root = crate::layout::plan_root(
                crate::layout::RootRequest::ExactSize {
                    width: 800,
                    height: 600,
                },
                |stage| {
                    let inputs = crate::plots::shared_x::prepare_plot(
                        &source,
                        &stage.operation(),
                        Some(&theme),
                    )?;
                    inputs.prepare_discrete_guides(
                        &source,
                        &theme,
                        &stage.operation(),
                        &crate::layout::ComponentAddress::root(),
                        false,
                    )?;
                    let requirements = crate::layout::foundational_plot_requirements(
                        Some(&inputs),
                        &source,
                        source.kind(),
                        &theme,
                        &crate::layout::ComponentAddress::root(),
                        true,
                        true,
                        true,
                        Some(stage),
                    )?;
                    let mut presence = stage.reserve_scalar_presence_map()?;
                    let mut census = crate::layout::PreparedTextCensus::materialize(
                        requirements,
                        Some(stage),
                        Some(&mut presence),
                    )
                    .map_err(|error| Error::Render(error.to_string()))?;
                    text.prepare_font_environment_from_census(&mut census, presence, stage)?;
                    Ok((inputs, stage.operation()))
                },
                |(inputs, operation), assigned, round| {
                    let plot = inputs.attach(source.clone())?;
                    let plan = measure_single_plot(
                        &plot,
                        inputs.plots.first(),
                        inputs.intrinsic(&inputs.plots[0]),
                        plot.kind(),
                        &theme,
                        plot.measured_canvas_layout(
                            inputs.plots.first(),
                            inputs.intrinsic(&inputs.plots[0]),
                            operation,
                            &theme,
                        )?
                        .into_shared(),
                        round,
                        false,
                    )?;
                    Ok(crate::layout::MeasuredLayout::new(
                        assigned.assigned_size()?,
                        plan,
                    ))
                },
                |_, _, plan, _| Ok(plan),
            )
            .unwrap();
            let plan = root.plan();
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
            let text_sizes =
                crate::decorations::title_block::plan::RootTitleBlockPlan::appearances(
                    &theme,
                    source.description().prose(),
                )
                .map(|role| role.theme.get_font_size());
            let caption_top = if caption.is_empty() {
                0.0
            } else {
                8.0 + crate::theme::composition::hard(&plan.frame.root_text.reserves.bottom)
                    .unwrap()
                    - crate::theme::composition::hard(
                        &theme
                            .root_title_lengths(text_sizes)
                            .unwrap()
                            .caption_to_content_clearance,
                    )
                    .unwrap()
            };
            assert!(axis_ink_bottom >= caption_top, "caption={caption:?}, body={body:?}, panel={panel:?}, required={required:?}, caption_top={caption_top}, axis_bottom={axis_ink_bottom}");
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
    #[expect(
        clippy::too_many_lines,
        reason = "one prepared font operation carries selected legend operands through mutation, capture and replay"
    )]
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
        let theme = crate::Theme::publication().resolve().unwrap();
        let text = crate::text::TextService::default();
        let theme = crate::text::TextRequestContext::new(&theme, &text);
        measurements::start();
        let root = crate::layout::plan_root(
            crate::layout::RootRequest::ExactSize {
                width: 800,
                height: 600,
            },
            |stage| {
                let inputs =
                    crate::plots::shared_x::prepare_plot(&source, &stage.operation(), None)?;
                inputs.prepare_discrete_guides(
                    &source,
                    &theme,
                    &stage.operation(),
                    &crate::layout::ComponentAddress::root()
                        .child(crate::layout::AddressRole::PlotBody, 0)?,
                    false,
                )?;
                let requirements = crate::layout::foundational_plot_requirements(
                    Some(&inputs),
                    &source,
                    source.kind(),
                    &theme,
                    &crate::layout::ComponentAddress::root(),
                    true,
                    true,
                    true,
                    Some(stage),
                )?;
                let mut presence = stage.reserve_scalar_presence_map()?;
                let mut census = crate::layout::PreparedTextCensus::materialize(
                    requirements,
                    Some(stage),
                    Some(&mut presence),
                )
                .map_err(|error| Error::Render(error.to_string()))?;
                theme
                    .text
                    .prepare_font_environment_from_census(&mut census, presence, stage)?;
                Ok((inputs, stage.operation()))
            },
            |(inputs, operation), assigned, round| {
                let plot = inputs.attach(source.clone())?;
                Ok(crate::layout::MeasuredLayout::new(
                    assigned.assigned_size()?,
                    measure_single_plot(
                        &plot,
                        inputs.plots.first(),
                        inputs.intrinsic(&inputs.plots[0]),
                        plot.kind(),
                        &theme,
                        plot.measured_canvas_layout(
                            inputs.plots.first(),
                            inputs.intrinsic(&inputs.plots[0]),
                            operation,
                            &theme,
                        )?
                        .into_shared(),
                        round,
                        false,
                    )?,
                ))
            },
            |(inputs, _), _, mut plan, stage| {
                let MeasuredRootContent::Single { plot, legend } = &mut plan.content else {
                    panic!("measured single plot")
                };
                assert_eq!(plot.plot_id, inputs.plots[0].scope.id);
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
                capture_text_replay_plan(
                    800,
                    600,
                    "retained standalone legend",
                    &theme,
                    stage,
                    |backend| {
                        render_single_plot(
                            &source,
                            source.kind(),
                            plot,
                            legend.as_ref(),
                            &plan.frame,
                            backend,
                        )
                    },
                )
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
