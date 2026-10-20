//! One-canvas chart lifecycle and immutable standalone scene planning.

use crate::Error;

use super::backend::{capture_text_replay_plan, validate_text_replay_plan, TextReplayPlan};
use crate::builder::Plot;
use crate::decorations::plan::PanelPlan;
use crate::layout::PanelBox;
use crate::render::backend::{Backend, SurfaceRole};
use crate::spec::{DecorationMode, PlotKind, PlotRenderContext, PlotRenderResult};
use crate::theme::ResolvedTheme;

pub(super) struct PlotPlan {
    root: crate::layout::FrozenRoot<TextReplayPlan, PreparedPlotInputs>,
}

struct PreparedPlotInputs {
    axes: crate::layout::interior::PreparedInteriorInputs,
    facet_guide: Option<super::facets::PreparedFacetGuide>,
    theme: Box<ResolvedTheme>,
    text_census: crate::layout::PreparedTextCensus,
    root_text: crate::decorations::title_block::plan::RootTitleBlockIntrinsics,
    root_frame: std::cell::RefCell<Option<CachedRootFrame>>,
}

struct CachedRootFrame {
    dimensions: (u32, u32),
    minimum: [u64; 2],
    frame: std::sync::Arc<crate::layout::RootFramePlan>,
}

impl PreparedPlotInputs {
    /// This prepared owner freezes its source, theme, fonts and instructions for
    /// one operation. Only root constraints and traced body minima can change.
    fn root_frame(
        &self,
        source: &Plot,
    ) -> Result<std::sync::Arc<crate::layout::RootFramePlan>, Error> {
        let dimensions = source.canvas_dimensions()?;
        let minimum = source.intrinsic_body_minimum(&self.theme)?;
        let minimum = [minimum.0.to_bits(), minimum.1.to_bits()];

        if let Some(cached) = self.root_frame.borrow().as_ref() {
            if cached.dimensions == dimensions && cached.minimum == minimum {
                return Ok(std::sync::Arc::clone(&cached.frame));
            }
        }
        let frame = source.measured_canvas_layout(&self.theme)?;

        let frame = frame.into_shared();
        *self.root_frame.borrow_mut() = Some(CachedRootFrame {
            dimensions,
            minimum,
            frame: std::sync::Arc::clone(&frame),
        });
        Ok(frame)
    }
}

enum PlotLayoutPlan {
    Unfaceted(Box<UnfacetedPlotPlan>),
    Faceted(Box<super::facets::FacetPlan>),
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
    let plan = PlotPlan::new(&source, source.kind()).unwrap();
    let prepared = plan.root.prepared();
    let source = prepared.axes.attach(source).unwrap();

    prepared.root_frame.borrow_mut().take();
    let original = prepared.root_frame(&source).unwrap();
    let shapes = crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get);

    let repeated = prepared.root_frame(&source).unwrap();
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
        .root_frame(&source.clone().with_canvas_size(641, 480))
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
        .root_frame(&source.with_canvas_size(641, 480))
        .unwrap();
    assert!(std::sync::Arc::ptr_eq(&changed, &repeated_changed));
}

#[cfg(all(test, unix, not(any(target_os = "macos", target_os = "android"))))]
mod soft_hyphen_tests {
    use super::*;

    #[test]
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
            theme.typography.root.title.font_size = 240;
            theme.typography.root.title.weight = crate::TextWeight::Bold;
            assert_ne!(
                theme.typography.root.title.weight,
                theme.typography.axes.tick_label.weight
            );
            let plan = PlotPlan::new_with_resolved_theme(&plot, PlotKind::Scatter, theme)
                .unwrap_or_else(|error| panic!("width {width}: {error}"));
            let prepared = plan.root.prepared();
            let captured = plan.root.plan().retained_text();
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
                .find(|requirement| requirement.text_role == crate::layout::PreparedTextRole::Title)
                .unwrap();
            assert_eq!(title.source, SOURCE);
            let pair = &census.pairs[title.pair_ordinal as usize];
            assert!(pair.required_scalars.contains(&'-'));
            assert!(census
                .pairs
                .iter()
                .filter(|pair| pair.pair_ordinal != title.pair_ordinal)
                .all(|pair| !pair.required_scalars.contains(&'-')));
            let environment = prepared.theme.prepared_font_environment().unwrap();
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

struct UnfacetedPlotPlan {
    plot: Plot,
    panel_plan: PanelPlan,
    canvas_layout: std::sync::Arc<crate::layout::RootFramePlan>,
    legend: Option<(String, crate::decorations::legend::LegendDrawPlan)>,
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
        let plan = PlotPlan::new_with_theme_factory(&source, source.kind(), || {
            let mut theme = crate::Theme::publication().resolve()?;
            edit(&mut theme)?;
            Ok(Box::new(theme))
        })?;
        let dimensions = plan.dimensions();
        let mut backend = plan.take_svg_backend()?;
        plan.render(&mut backend)?;
        Ok(NaturalScene {
            dimensions,
            svg: backend.into_bytes()?,
        })
    }

    fn ordinary_width(theme: &mut ResolvedTheme, width: f32) -> Result<(), Error> {
        let frame = theme.axes.frame().expect("preset ordinary frame");
        let changed = crate::theme::FrameTheme::new(
            frame.sides().clone(),
            frame.stroke().clone().with_width(width)?,
        )?;
        theme.axes = theme.axes.clone().with_frame(Some(changed))?;
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
                theme.axes = theme.axes.clone().with_frame(None)?;
                Ok(())
            })
            .unwrap();
            assert_eq!(absent, baseline, "width {width:?}, absent ordinary frame");
            for sides in [
                crate::theme::FrameSides::new(false, false, false, true).unwrap(),
                crate::theme::FrameSides::new(true, true, false, false).unwrap(),
            ] {
                let edited = natural_scene_with_theme_edit(&source, width, |theme| {
                    let frame = theme.axes.frame().unwrap().clone().with_sides(sides)?;
                    theme.axes = theme.axes.clone().with_frame(Some(frame))?;
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
                let frame = theme.axes.cell_frame().unwrap();
                let changed = crate::theme::FrameTheme::new(
                    frame.sides().clone(),
                    frame.stroke().clone().with_width(64.0)?,
                )?;
                theme.axes = theme.axes.clone().with_cell_frame(Some(changed))?;
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
                let frame = theme.axes.cell_frame().unwrap();
                let changed = crate::theme::FrameTheme::new(
                    frame.sides().clone(),
                    frame.stroke().clone().with_width(f32::MAX)?,
                )?;
                theme.axes = theme.axes.clone().with_cell_frame(Some(changed))?;
                Ok(())
            })
            .is_err());
        }
    }

    #[test]
    fn private_facet_gap_participates_in_containing_guide_recovery() {
        let source = Plot::scatter(
            crate::NumericValues::new("x", [0.0, 1.0, 0.0, 1.0]),
            crate::NumericValues::new("y", [0.0, 1.0, 1.0, 0.0]),
        )
        .hue(crate::CategoryValues::new("series", ["extraordinarycategory"; 4]).unwrap())
        .facet(crate::FacetSpec::column(
            crate::CategoryValues::new("facet", ["left", "left", "right", "right"]).unwrap(),
        ))
        .x_label("")
        .y_label("")
        .labels(
            crate::Labels::raw()
                .alias("facet", "f")
                .alias("left", "l")
                .alias("right", "r"),
        )
        .width(550)
        .erase();
        for gap in [12.0, 280.0] {
            let mut theme = crate::Theme::publication().resolve().unwrap();
            theme.composition = theme
                .composition
                .clone()
                .with_facet_gap(crate::theme::RelativeSpacing::from_pixels(gap, 15).unwrap())
                .unwrap();
            crate::orchestrator::measurements::start();
            let result =
                PlotPlan::new_with_theme_factory(&source, source.kind(), || Ok(Box::new(theme)));
            let work = crate::orchestrator::measurements::finish();
            let plan = result.unwrap_or_else(|error| panic!("gap {gap}: {error}"));
            assert_eq!(plan.dimensions().0, 550);
            let edge = if gap < 100.0 {
                crate::LegendPosition::OutsideRight
            } else {
                crate::LegendPosition::OutsideTop
            };
            assert!(
                work.guides.iter().any(|trial| trial.edge == edge
                    && trial.outcome == crate::orchestrator::measurements::GuideOutcome::Accepted),
                "gap {gap}: {:?}",
                work.guides
            );
        }
    }

    #[test]
    fn public_root_requests_measure_complete_content_and_replace_both_dimensions() {
        let source = Plot::scatter(crate::NumericValues::new("x", [0.0, 1.0, 2.0]),
            crate::NumericValues::new("y", [0.0, 1.0, 2.0])).caption(
            "A complete explanatory caption with legal word boundaries that must reflow at the assigned width and contribute its complete measured height.").erase();
        let exact_source = source.clone().width(300).size(640, 480);
        let exact = PlotPlan::new(&exact_source, exact_source.kind()).unwrap();
        assert_eq!(exact.root.dimensions(), (640, 480));
        let width_source = source.clone().size(0, 0).width(300);
        let width = PlotPlan::new(&width_source, width_source.kind()).unwrap();
        assert_eq!(width.root.dimensions().0, 300);
        assert!(width.root.dimensions().1 > 0);
        let automatic = PlotPlan::new(&source, source.kind()).unwrap();
        assert!(automatic.root.dimensions().0 > 0 && automatic.root.dimensions().1 > 0);
        assert!(width
            .root
            .plan()
            .retained_text()
            .iter()
            .any(|(_, text)| text.source == source.caption_text().unwrap()));
        let short_source = source.clone().caption("Short caption.").width(300);
        let short = PlotPlan::new(&short_source, short_source.kind()).unwrap();
        assert!(width.dimensions().1 > short.dimensions().1);
        assert!(
            width
                .root
                .plan()
                .retained_text()
                .iter()
                .find(|(_, text)| text.source == source.caption_text().unwrap())
                .unwrap()
                .1
                .lines
                .len()
                > 1
        );
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
    fn natural_prose_metrics_use_the_sealed_role_specific_font_pair() {
        let plot = Plot::scatter(
            crate::NumericValues::new("x", [1.0, 2.0]),
            crate::NumericValues::new("y", [3.0, 4.0]),
        )
        .title("A complete title with independently bold typography and legal boundaries")
        .erase();
        let mut theme = crate::Theme::publication().resolve().unwrap();
        theme.typography.root.title.weight = crate::TextWeight::Bold;
        let plan =
            PlotPlan::new_with_theme_factory(&plot, plot.kind(), || Ok(Box::new(theme))).unwrap();
        assert!(plan
            .root
            .prepared()
            .text_census
            .requirements
            .iter()
            .any(
                |entry| entry.text_role == crate::layout::PreparedTextRole::PreferenceMetric
                    && entry.weight == crate::TextWeight::Bold
                    && entry.source == "0"
            ));
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
    fn faceted_content_roots_keep_owned_sparse_wrap_and_complete_multiline_caption() {
        let plot = Plot::scatter(crate::NumericValues::new("x", [0.0, 1.0, 2.0, 3.0, 4.0, 5.0]),
            crate::NumericValues::new("y", [0.0, 1.0, 2.0, 3.0, 4.0, 5.0]))
            .facet(crate::FacetSpec::wrap(crate::CategoryValues::new("group", ["a", "a", "b", "b", "c", "c"]).unwrap(), 2).unwrap())
            .caption("First complete explanatory paragraph with legal breaks.\nSecond complete explanatory paragraph.")
            .erase();
        for width in [None, Some(800)] {
            let plot = width.map_or_else(|| plot.clone(), |width| plot.clone().width(width));
            let plan = PlotPlan::new(&plot, plot.kind()).unwrap();
            if let Some(width) = width {
                assert_eq!(plan.dimensions().0, width);
            }
            assert!(plan.dimensions().1 > 0);
            assert!(plan
                .root
                .plan()
                .retained_text()
                .iter()
                .any(|(_, text)| text.source == plot.caption_text().unwrap()));
            let strips = plan
                .root
                .plan()
                .retained_text()
                .iter()
                .filter(|(_, text)| text.source.contains(" = "))
                .count();
            assert_eq!(strips, 3);
        }
    }

    #[test]
    fn natural_outside_preferences_respect_explicit_horizontal_and_inside_placement() {
        let source = Plot::scatter(
            crate::NumericValues::new("x", [0.0, 1.0]),
            crate::NumericValues::new("y", [1.0, 2.0]),
        )
        .hue(crate::CategoryValues::new("series", ["one", "two"]).unwrap());
        let mut widths = Vec::new();
        for policy in [
            crate::LegendPolicy::Hidden,
            crate::LegendPolicy::Visible(crate::LegendPosition::OutsideTop),
            crate::LegendPolicy::Visible(crate::LegendPosition::InsideTopRight),
            crate::LegendPolicy::Visible(crate::LegendPosition::Auto),
        ] {
            let source = source.clone().legend(policy).erase();
            widths.push(
                PlotPlan::new(&source, source.kind())
                    .unwrap()
                    .dimensions()
                    .0,
            );
        }
        assert_eq!(widths[0], widths[1]);
        assert_eq!(widths[0], widths[2]);
        assert!(widths[3] > widths[0]);
    }

    #[test]
    fn matrix_content_roots_compose_the_native_hue_guide_and_variable_square() {
        let source = Plot::matrix(vec![
            crate::NumericValues::new("first_value", [1.0, 2.0, 3.0, 4.0, 5.0, 6.0]),
            crate::NumericValues::new("second_value", [2.0, 1.0, 4.0, 3.0, 6.0, 5.0]),
        ])
        .hue(
            crate::CategoryValues::new("series", ["one", "one", "one", "two", "two", "two"])
                .unwrap(),
        )
        .caption("A complete explanatory caption for the native variable matrix.")
        .erase();
        for width in [None, Some(800)] {
            let source = width.map_or_else(|| source.clone(), |width| source.clone().width(width));
            let plan = PlotPlan::new(&source, source.kind()).unwrap();
            if let Some(width) = width {
                assert_eq!(plan.dimensions().0, width);
            }
            let mut svg = plan.take_svg_backend().unwrap();
            plan.render(&mut svg).unwrap();
            let dimensions = plan.dimensions();
            let scene =
                crate::scene::RenderedScene::new(dimensions.0, dimensions.1, svg.into_output())
                    .unwrap();
            assert!(std::str::from_utf8(scene.svg_bytes())
                .unwrap()
                .contains("First value"));
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
            .with_suffix(" u")
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
                .any(|(_, text)| text.source == source.caption_text().unwrap()));
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
            .annotations(crate::HeatmapAnnotations::Values(crate::NumericFormat::new(crate::NumericNotation::Decimal, crate::NumericPrecision::AdaptiveFractional { min: 0, max: 2 }).unwrap().with_prefix("v=").unwrap()))
            .color_label_format(crate::NumericFormat::default().with_prefix("g=").unwrap())
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
        Self::new_with_theme_factory(source, kind, || Ok(Box::new(source.resolve_theme()?)))
    }

    #[cfg(test)]
    pub(super) fn new_with_resolved_theme(
        source: &Plot,
        kind: PlotKind,
        theme: ResolvedTheme,
    ) -> Result<Self, Error> {
        Self::new_with_theme_factory(source, kind, || Ok(Box::new(theme)))
    }

    #[expect(
        clippy::too_many_lines,
        reason = "ordered axis preparation, font preparation, measurement, and capture lifecycle"
    )]
    fn new_with_theme_factory(
        source: &Plot,
        kind: PlotKind,
        resolve_theme: impl FnOnce() -> Result<Box<ResolvedTheme>, Error>,
    ) -> Result<Self, Error> {
        let sizing = source.sizing_request().requested_dimensions();
        #[cfg(test)]
        super::PLOT_PLAN_CALLS.with(|calls| calls.set(calls.get() + 1));
        let mut children = crate::decorations::plan::children::ChildPlans::facets(source);
        let root = crate::layout::plan_exact_root(
            sizing.0,
            sizing.1,
            |prepared_stage| {
                let theme = resolve_theme()?;
                source.validate_configuration(kind)?;
                let axes = crate::plots::shared_x::prepare_plot(
                    source,
                    &prepared_stage.operation(),
                    Some(&theme),
                )?;
                axes.validate_categorical_scale(source, &theme, None)?;
                let facet_guide = source
                    .has_facets()
                    .then(|| {
                        super::facets::prepare_discrete_guide(
                            source,
                            kind,
                            &theme,
                            &axes,
                            &prepared_stage.operation(),
                        )
                    })
                    .transpose()?;
                if !source.has_facets() {
                    axes.prepare_discrete_guides(
                        source,
                        &theme,
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
                    theme.as_ref(),
                    &crate::layout::ComponentAddress::root(),
                    true,
                    true,
                    true,
                    Some(prepared_stage),
                )?;
                crate::layout::PreparedTextRequirement::append_preference_metrics(
                    &mut requirements,
                    &theme,
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
                theme.prepare_font_environment_from_census(
                    &mut census,
                    scalar_presence,
                    prepared_stage,
                )?;
                // Scientific preparation and the font census are complete. No
                // parent width has been allocated when these summaries are retained.
                axes.prepare_intrinsics(source, &theme)?;
                let root_text =
                    crate::decorations::title_block::plan::RootTitleBlockPlan::intrinsic_widths(
                        &theme,
                        [
                            source.title_text(),
                            source.subtitle_text(),
                            source.caption_text(),
                        ],
                        None,
                    )?;
                Ok(PreparedPlotInputs {
                    axes,
                    facet_guide,
                    theme,
                    text_census: census,
                    root_text,
                    root_frame: std::cell::RefCell::new(None),
                })
            },
            |prepared, assigned, round| {
                let assigned = if assigned.requested_dimensions().1.is_some() {
                    assigned
                } else {
                    let attached = prepared.axes.attach(source.clone())?;
                    if source.has_facets() {
                        super::facets::natural_size(
                            &attached,
                            &prepared.theme,
                            assigned,
                            round,
                            prepared.facet_guide.as_ref().expect("prepared facet guide"),
                            &prepared.axes,
                        )?
                    } else {
                        crate::layout::interior::sizing::plot_size(
                            &attached,
                            &prepared.axes,
                            &prepared.theme,
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
                let plan = if source.has_facets() {
                    PlotLayoutPlan::Faceted(Box::new(
                        super::facets::FacetPlan::new(
                            &source,
                            kind,
                            &prepared.theme,
                            round,
                            &mut children,
                            prepared
                                .facet_guide
                                .as_ref()
                                .expect("faceted root guide was prepared"),
                            sizing.1.is_none(),
                            Some(&prepared.axes),
                        )
                        .map_err(crate::layout::fit::PlanError::into_error)?,
                    ))
                } else {
                    PlotLayoutPlan::Unfaceted(Box::new(UnfacetedPlotPlan::new(
                        &source,
                        kind,
                        &prepared.theme,
                        prepared.root_frame(&source)?,
                        round,
                        sizing.1.is_none(),
                    )?))
                };
                children.finish_round(round);
                let assigned = match &plan {
                    PlotLayoutPlan::Unfaceted(plan) => {
                        let (width, height) = plan.plot.canvas_dimensions()?;
                        crate::layout::RootSize::new(width, height)
                    }
                    PlotLayoutPlan::Faceted(plan) => {
                        crate::layout::RootSize::new(plan.dimensions.0, plan.dimensions.1)
                    }
                };
                Ok(crate::layout::MeasuredLayout::new(assigned, plan))
            },
            |prepared, assigned, layout, final_stage| {
                let (width, height) = assigned.dimensions()?;
                let replay = capture_text_replay_plan(
                    width,
                    height,
                    "prepared plot scene capture",
                    &prepared.theme,
                    final_stage,
                    |backend| {
                        render_plot_scene(
                            source,
                            kind,
                            width,
                            height,
                            &prepared.theme,
                            &layout,
                            backend,
                        )
                    },
                )?;
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
        let _ = &self.root.prepared().text_census;
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
    theme: &ResolvedTheme,
    layout: &PlotLayoutPlan,
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    backend.draw_surface(
        PanelBox::new(0.0, f64::from(width), 0.0, f64::from(height)),
        theme.surfaces.canvas_root,
        SurfaceRole::CanvasRoot,
    )?;
    match layout {
        PlotLayoutPlan::Unfaceted(plan) => plan.render(source, kind, theme, backend),
        PlotLayoutPlan::Faceted(plan) => plan.render(source, kind, theme, backend),
    }
}

impl UnfacetedPlotPlan {
    fn new(
        source: &Plot,
        kind: PlotKind,
        theme: &ResolvedTheme,
        mut canvas_layout: std::sync::Arc<crate::layout::RootFramePlan>,
        round: &mut crate::layout::kernel::FeedbackRound,
        content_height: bool,
    ) -> Result<Self, Error> {
        let cell = canvas_layout.panel_box;
        let mut context = source.configured_source_rows()?.map_or_else(
            || PlotRenderContext::new(source.data(), cell, theme),
            |rows| PlotRenderContext::with_source_rows(source.data(), rows, cell, theme),
        );
        context.content_height = content_height;
        source.validate_interior_preparation()?;
        let discrete_guide = if kind == PlotKind::Matrix {
            None
        } else {
            source
                .prepared_discrete_guide()?
                .map(|descriptor| {
                    crate::decorations::legend::discrete::DiscreteGuideSlot::new(
                        descriptor,
                        &crate::layout::ComponentAddress::root()
                            .child(crate::layout::AddressRole::PlotBody, 0)?,
                        source.legend_position_spec(),
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
            let height = crate::layout::interior::sizing::integral(
                canvas_layout.height_for_body(panel_plan.frame_box.height(), theme)?,
            )?;
            plot = plot.with_canvas_size(source.canvas_dimensions()?.0, height);
            let mut resized = plot.measured_canvas_layout(theme)?;
            resized.panel_box = panel_plan.frame_box;
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
                let length = source
                    .axis_scope()
                    .map_or(Some(6), |scope| scope.len().checked_add(7))
                    .ok_or_else(|| {
                        Error::Render("standalone legend layer length overflow".into())
                    })?;
                crate::data::source::reserve_source_string(&mut layer, length)?;
                if let Some(scope) = source.axis_scope() {
                    layer.push_str(scope);
                    layer.push('-');
                }
                layer.push_str("legend");
                Ok::<_, Error>((layer, draw))
            })
            .transpose()?;
        Ok(Self {
            plot,
            panel_plan,
            canvas_layout,
            legend,
        })
    }

    fn render(
        &self,
        source: &Plot,
        kind: PlotKind,
        theme: &ResolvedTheme,
        backend: &mut dyn Backend,
    ) -> Result<(), Error> {
        backend.draw_surface(
            self.panel_plan.panel_box,
            theme.surfaces.plot_panel,
            SurfaceRole::PlotPanel,
        )?;
        if self.panel_plan.result.decoration_mode == DecorationMode::Standard {
            crate::decorations::apply::apply_grid(source, &self.panel_plan, backend)?;
        }
        let source_rows = self.plot.configured_source_rows()?;
        let context = source_rows
            .map_or_else(
                || PlotRenderContext::new(self.plot.data(), self.panel_plan.panel_box, theme),
                |rows| {
                    PlotRenderContext::with_source_rows(
                        self.plot.data(),
                        rows,
                        self.panel_plan.panel_box,
                        theme,
                    )
                },
            )
            .with_retained_body(&self.panel_plan.result);
        let rendered = self.plot.render_panel(kind, context, backend)?;

        crate::significance::render_selected(&self.panel_plan.result, backend)?;

        match self.panel_plan.result.decoration_mode {
            DecorationMode::Standard => {
                crate::decorations::apply::apply_axes(source, &self.panel_plan, backend)?;
                if let Some((layer, legend)) = &self.legend {
                    crate::render::backend::with_layer(backend, layer, |backend| {
                        legend.draw(backend)
                    })?;
                }
            }
            DecorationMode::SelfManaged => {}
        }
        crate::decorations::apply::apply_canvas_text(source, &self.canvas_layout, backend)?;
        debug_assert_eq!(rendered, self.panel_plan.result.decoration_mode);
        Ok(())
    }
}

impl Plot {
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
        panel: crate::layout::PanelBox,
        theme: &ResolvedTheme,
    ) -> Result<PlotRenderResult, Error> {
        crate::layout::fit::measured(self.preflight_measurement(
            kind,
            panel,
            theme,
            &crate::layout::fit::ROOT_SCOPE,
        ))
        .map_err(crate::layout::fit::PlanError::into_error)
    }

    pub(crate) fn preflight_measurement(
        &self,
        kind: PlotKind,
        panel: crate::layout::PanelBox,
        theme: &ResolvedTheme,
        scope: &crate::layout::fit::FitScope,
    ) -> Result<crate::layout::fit::FitOutcome<PlotRenderResult>, Error> {
        crate::layout::interior::InteriorRoute::classify(kind).validate(self)?;
        self.validate_category_order_configuration(kind)?;
        if !self.significance_annotations().is_empty() && !kind.supports_significance_annotations()
        {
            return Err(Error::Data(format!(
                "significance annotations are not supported for {kind:?}; use a supported categorical family or remove the annotations"
            )));
        }
        let source_rows = self.configured_source_rows()?;
        let context = source_rows.map_or_else(
            || PlotRenderContext::new(self.data(), panel, theme),
            |rows| PlotRenderContext::with_source_rows(self.data(), rows, panel, theme),
        );
        crate::layout::fit::outcome(self.measure_panel(kind, context.with_fit_scope(scope)))
    }

    pub(crate) fn configured_source_rows(&self) -> Result<Option<&[usize]>, Error> {
        if let Some(rows) = self.source_row_map() {
            if rows.len() != self.data().height() {
                return Err(Error::Data(format!(
                    "chart source-row map has length {}, but chart data has {} rows",
                    rows.len(),
                    self.data().height()
                )));
            }
        }
        Ok(self.source_row_map())
    }
}

#[cfg(test)]
mod root_title_tests {
    use super::*;
    use crate::theme::TitleAlignment::{Center, End, Start};

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
            let theme = crate::Theme::publication().resolve().unwrap();
            let root = crate::layout::plan_exact_root(
                800,
                600,
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
                    Ok(inputs)
                },
                |inputs, assigned, round| {
                    let plot = inputs.attach(source.clone())?;
                    let plan = UnfacetedPlotPlan::new(
                        &plot,
                        plot.kind(),
                        &theme,
                        plot.measured_canvas_layout(&theme)?.into_shared(),
                        round,
                        false,
                    )?;
                    Ok(crate::layout::MeasuredLayout::new(assigned, plan))
                },
                |_, _, plan, _| Ok(plan),
            )
            .unwrap();
            let plan = root.plan();
            let body = plan.canvas_layout.panel_box;
            let panel = plan.panel_plan.panel_box;
            reserves.push((panel.y0 - body.y0).to_bits());
            assert!(
                panel.y0 > body.y0,
                "multiline axis requires its own extra gutter"
            );
            let required = plan.panel_plan.result.axes.as_ref().unwrap().requirements();
            let axis_ink_bottom = panel.y0 - required.bottom;
            let caption_top = if caption.is_empty() {
                0.0
            } else {
                8.0 + crate::theme::composition::hard(&plan.canvas_layout.root_text.reserves.bottom)
                    .unwrap()
                    - crate::theme::composition::hard(
                        &theme
                            .root_title_lengths()
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
    fn matrix_root_uses_one_title_producer_with_independent_odd_size_alignments() {
        for preset in [crate::Theme::publication(), crate::Theme::science()] {
            for explicit in [false, true] {
                let mut plot = Plot::matrix(vec![
                    crate::NumericValues::new("x", [0.0, 1.0, 2.0, 3.0]),
                    crate::NumericValues::new("y", [2.0, 1.0, 3.0, 4.0]),
                ])
                .title("Title\nx")
                .subtitle("Subtitle\ny")
                .caption("Caption\nz")
                .size(533, 681)
                .theme(preset.clone())
                .erase();
                if explicit {
                    plot = plot.title_alignment(End).subtitle_alignment(Start);
                }
                let theme = plot.resolve_theme().unwrap();
                assert_eq!(theme.composition.root_title_alignment(), Center);
                assert_eq!(theme.composition.root_subtitle_alignment(), None);
                assert_eq!(theme.composition.root_caption_alignment(), Start);
                let prepared = crate::plots::shared_x::prepare_plot(
                    &plot,
                    &crate::font::canonical::OperationContext::new(),
                    None,
                )
                .unwrap()
                .attach(plot.clone())
                .unwrap();
                let canvas = prepared.measured_canvas_layout(&theme).unwrap();
                assert!(
                    crate::theme::composition::hard(&canvas.root_text.reserves.top).unwrap() > 0.0
                );
                assert!(
                    crate::theme::composition::hard(&canvas.root_text.reserves.bottom).unwrap()
                        > 0.0
                );
                let plan =
                    PlotPlan::new_with_resolved_theme(&plot, PlotKind::Matrix, theme).unwrap();
                let retained = plan.root.plan().retained_text();
                for (source, alignment) in [
                    (
                        "Title\nx",
                        if explicit {
                            crate::layout::PhysicalTextAlignment::End
                        } else {
                            crate::layout::PhysicalTextAlignment::Center
                        },
                    ),
                    (
                        "Subtitle\ny",
                        if explicit {
                            crate::layout::PhysicalTextAlignment::Start
                        } else {
                            crate::layout::PhysicalTextAlignment::Center
                        },
                    ),
                    ("Caption\nz", crate::layout::PhysicalTextAlignment::Start),
                ] {
                    let matching = retained
                        .iter()
                        .filter(|(_, text)| text.source == source)
                        .collect::<Vec<_>>();
                    assert_eq!(matching.len(), 1);
                    assert_eq!(matching[0].1.style.alignment, alignment);
                    assert_eq!(matching[0].1.lines.len(), 2);
                }
                let shapes = crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get);
                let mut backend = plan.take_svg_backend().unwrap();
                plan.render(&mut backend).unwrap();
                let svg = backend.into_bytes().unwrap();
                let document =
                    roxmltree::Document::parse(std::str::from_utf8(&svg).unwrap()).unwrap();
                assert_eq!(
                    document.root_element().attribute("viewBox"),
                    Some("0 0 533 681")
                );
                assert_eq!(
                    document
                        .descendants()
                        .filter(|node| node.has_tag_name("clipPath"))
                        .count(),
                    4
                );
                assert_eq!(
                    shapes,
                    crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get)
                );
            }
        }
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
                .with_root_caption_alignment(Center)
                .unwrap();
            let theme = preset
                .customize()
                .with_composition(composition)
                .build()
                .unwrap();
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
            let theme = plot.resolve_theme().unwrap();
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
                let canonical =
                    crate::render::backend::validate_planned_text(*placement, text).unwrap();
                assert!((canonical.position.x.value() - placement.position.x).abs() < 0.000_001);
                assert!((canonical.position.y.value() - placement.position.y).abs() < 0.000_001);
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
        .erase()
        .with_legend_position(crate::LegendPosition::OutsideRight);
        let theme = crate::Theme::publication().resolve().unwrap();
        measurements::start();
        let root = crate::layout::plan_exact_root(
            800,
            600,
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
                Ok(inputs)
            },
            |inputs, assigned, round| {
                let plot = inputs.attach(source.clone())?;
                Ok(crate::layout::MeasuredLayout::new(
                    assigned,
                    UnfacetedPlotPlan::new(
                        &plot,
                        plot.kind(),
                        &theme,
                        plot.measured_canvas_layout(&theme)?.into_shared(),
                        round,
                        false,
                    )?,
                ))
            },
            |_, _, mut plan, stage| {
                let (layer, draw) = plan.legend.as_ref().unwrap();
                assert_eq!(layer, "legend");
                let mut geometry = crate::render::geometry::GeometrySnapshot::new();
                draw.geometry(&mut geometry)?;
                let mut backend = crate::render::geometry::GeometryBackend::new(&theme)?;
                backend.begin(800, 600)?;
                draw.draw(&mut backend)?;
                backend.end()?;
                assert_eq!(geometry.bounds(), backend.bounds());
                let measured = plan.panel_plan.legend.as_mut().unwrap();
                std::sync::Arc::make_mut(&mut measured.legend.metrics).font_size = f64::NAN;
                measured.bounds = measured.bounds.translated(10_000.0, 10_000.0);
                capture_text_replay_plan(
                    800,
                    600,
                    "retained standalone legend",
                    &theme,
                    stage,
                    |backend| plan.render(&source, source.kind(), &theme, backend),
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
