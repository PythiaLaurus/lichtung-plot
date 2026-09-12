//! Heterogeneous plot composition on one canvas.

use std::path::Path;
use std::sync::Arc;

use crate::Error;

use crate::decorations::legend::{Legend, LegendEntry};
use crate::decorations::plan::children::MeasurementPhase;
use crate::decorations::title_block::TitleBlock;
use crate::layout::canvas::{PanelRequirements, PANEL_HEADER_GAP};
use crate::layout::fit::{FitFailure, FitScope};
use crate::layout::kernel::{FeedbackKind, FitKind};
use crate::layout::{CanvasLayout, CanvasLayoutSpec, PanelBox, PhysicalTextAlignment, TextBlock};
use crate::render::backend::{with_layer, Backend, Point2D, SurfaceRole};
use crate::scene::RenderedScene;
use crate::spec::{AxisDomain, DecorationMode};
use crate::theme::ResolvedTheme;
use crate::{CanvasLegendPolicy, Labels, Output, Plot, PlotKind, Theme};

#[cfg(test)]
thread_local! {
    pub(crate) static CANVAS_THEME_EVIDENCE: std::cell::RefCell<Vec<(usize, &'static str, usize)>> = const { std::cell::RefCell::new(Vec::new()) };
    pub(crate) static CANVAS_SHARED_LEGEND_THEME_EVIDENCE: std::cell::RefCell<Vec<(usize, &'static str)>> = const { std::cell::RefCell::new(Vec::new()) };
    pub(crate) static CANVAS_SCENE_RENDER_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    pub(crate) static CANVAS_ASSIGNED_PANEL_EVIDENCE: std::cell::RefCell<Vec<(PanelBox, PanelBox)>> = const { std::cell::RefCell::new(Vec::new()) };
}

#[cfg(test)]
fn record_canvas_theme(owner: usize, stage: &'static str, theme: &ResolvedTheme) {
    CANVAS_THEME_EVIDENCE.with(|evidence| {
        evidence
            .borrow_mut()
            .push((owner, stage, std::ptr::from_ref(theme).addr()));
    });
}

#[cfg(test)]
fn record_shared_legend_theme(owner: usize, theme: &ResolvedPlotTheme) {
    let provenance = match theme {
        ResolvedPlotTheme::InheritedRoot => "inherited-root",
        ResolvedPlotTheme::ExplicitPlot(_) => "explicit-plot",
    };
    CANVAS_SHARED_LEGEND_THEME_EVIDENCE.with(|evidence| {
        evidence.borrow_mut().push((owner, provenance));
    });
}

struct PlannedCanvasPanel {
    index: usize,
    kind: PlotKind,
    canvas_cell: PanelBox,
    panel_plan: Arc<crate::decorations::plan::PanelPlan>,
    header: Vec<crate::render::backend::PlacedText>,
}

struct CanvasPlan {
    themes: Arc<ResolvedCanvasThemes>,
    plots: Vec<Option<Plot>>,
    canvas_layout: CanvasLayout,
    shared_legend: Option<crate::decorations::plan::ResolvedLegend>,
    panels: Vec<PlannedCanvasPanel>,
    text_panels: Vec<TextPanelPlan>,
    headers: Vec<CanvasHeaderPlan>,
    row_titles: Vec<CanvasHeaderPlan>,
}

struct PreparedCanvasInputs {
    themes: Arc<ResolvedCanvasThemes>,
    text_census: crate::layout::PreparedTextCensus,
    shared_x: Vec<Option<(f64, f64)>>,
    interiors: Vec<Option<crate::layout::interior::PreparedInteriorInputs>>,
}

struct PreparedCanvasRound<'a> {
    themes: Arc<ResolvedCanvasThemes>,
    canvas_layout: CanvasLayout,
    plots: Vec<Option<Plot>>,
    shared_legend: Option<(String, Arc<[LegendEntry]>)>,
    body_probes: Vec<Option<Arc<crate::spec::PlotRenderResult>>>,
    shared_x: &'a [Option<(f64, f64)>],
}

#[cfg(test)]
mod d1_tests {
    use super::*;

    #[test]
    fn shared_x_member_rejections_precede_font_discovery() {
        let numeric = Plot::scatter(
            crate::NumericValues::new("x", [0.0, 1.0]),
            crate::NumericValues::new("y", [1.0, 2.0]),
        );
        let categorical = Plot::boxplot(
            crate::CategoryValues::new("x", ["a", "b"]).unwrap(),
            crate::NumericValues::new("y", [1.0, 2.0]),
        )
        .erase();
        for (member, expected) in [
            (Some(categorical), "categorical x axis"),
            (
                Some(numeric.clone().x_limits(0.0, 1.0).erase()),
                "explicit x limits",
            ),
            (None, "unconfigured panel"),
        ] {
            let mut canvas = Canvas::grid(1, 2)
                .unwrap()
                .panel(0, 0, numeric.clone())
                .unwrap()
                .x_range_group([(0, 0), (0, 1)])
                .unwrap();
            if let Some(member) = member {
                canvas.panels[1] = Some(member);
            }
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
            let error = canvas.render_scene().unwrap_err();
            assert!(
                matches!(&error, Error::Data(message) if message.contains(expected)),
                "{error}"
            );
            assert_eq!(
                crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
                0
            );
        }
    }

    #[test]
    fn shared_x_domains_are_prepared_once_and_exact_containment_is_prefont() {
        let member = |values| {
            Plot::scatter(
                crate::NumericValues::new("x", values),
                crate::NumericValues::new("y", [1.0, 2.0]),
            )
            .legend(crate::LegendPolicy::Hidden)
        };
        let canvas = Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, member([0.0, 1.0]))
            .unwrap()
            .panel(0, 1, member([10.0, 20.0]))
            .unwrap()
            .x_range_group([(0, 0), (0, 1)])
            .unwrap()
            .size(1200, 800);
        crate::plots::shared_x::DOMAIN_PREPARATIONS.with(|count| count.set(0));
        crate::plots::scatter::STAGES.with(|stages| stages.set([0; 4]));
        let root = canvas.preflight_plan().unwrap();
        let shared = root.prepared().shared_x[0].unwrap();
        assert_eq!(root.prepared().shared_x[1], Some(shared));
        assert!(shared.0 <= 0.0 && shared.1 >= 20.0);
        assert_eq!(
            crate::plots::shared_x::DOMAIN_PREPARATIONS.with(std::cell::Cell::get),
            0
        );
        assert_eq!(
            crate::plots::scatter::STAGES.with(std::cell::Cell::get)[0],
            2
        );

        let cached = canvas.render_scene().unwrap();
        crate::decorations::plan::children::DISABLE_CHILD_REUSE.with(|disabled| disabled.set(true));
        let uncached = canvas.render_scene();
        crate::decorations::plan::children::DISABLE_CHILD_REUSE
            .with(|disabled| disabled.set(false));
        assert_eq!(cached.svg_bytes(), uncached.unwrap().svg_bytes());

        let mut invalid = canvas;
        invalid.panels[0] = Some(
            member([0.0, 1.0])
                .x_ticks(vec![crate::AxisTick::new(1000.0, "outside")])
                .erase(),
        );
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        let error = invalid.render_scene().unwrap_err();
        assert!(
            error.to_string().contains("outside resolved shared range"),
            "{error}"
        );
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }

    #[test]
    fn invalid_shared_x_family_domains_precede_font_discovery() {
        let invalid = [
            Plot::scatter(
                crate::NumericValues::new("x", [f64::NAN]),
                crate::NumericValues::new("y", [1.0]),
            )
            .erase(),
            Plot::density(crate::NumericValues::new("x", [1.0])).erase(),
            Plot::qq(crate::NumericValues::new("x", [1.0])).erase(),
            Plot::forest(
                crate::CategoryValues::new("term", ["a"]).unwrap(),
                crate::NumericValues::new("estimate", [0.0]),
                crate::Bounds::new(
                    crate::NumericValues::new("lower", [0.0]),
                    crate::NumericValues::new("upper", [1.0]),
                ),
                1.0,
            )
            .scale(crate::AxisScale::Log10)
            .erase(),
        ];
        for plot in invalid {
            let mut canvas = Canvas::grid(1, 2)
                .unwrap()
                .panel(
                    0,
                    1,
                    Plot::scatter(
                        crate::NumericValues::new("x", [1.0, 2.0]),
                        crate::NumericValues::new("y", [1.0, 2.0]),
                    ),
                )
                .unwrap()
                .x_range_group([(0, 0), (0, 1)])
                .unwrap();
            canvas.panels[0] = Some(plot);
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
            assert!(matches!(canvas.render_scene(), Err(Error::Data(_))));
            assert_eq!(
                crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
                0
            );
        }
    }

    #[test]
    fn shared_x_forest_union_is_not_log_transformed_twice() {
        let forest = || {
            Plot::forest(
                crate::CategoryValues::new("term", ["a"]).unwrap(),
                crate::NumericValues::new("estimate", [1.0]),
                crate::Bounds::new(
                    crate::NumericValues::new("lower", [0.1]),
                    crate::NumericValues::new("upper", [10.0]),
                ),
                1.0,
            )
            .scale(crate::AxisScale::Log10)
            .x_ticks(vec![
                crate::AxisTick::new(0.1, "low"),
                crate::AxisTick::new(10.0, "high"),
            ])
        };
        let canvas = Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, forest())
            .unwrap()
            .panel(0, 1, forest())
            .unwrap()
            .x_range_group([(0, 0), (0, 1)])
            .unwrap()
            .size(1200, 800);
        let root = canvas.preflight_plan().unwrap();
        assert_eq!(
            root.prepared().shared_x[0],
            Some(crate::layout::continuous_auto_range(
                (-1.0, 1.0),
                &crate::layout::AxisSpec::default()
            ))
        );
    }

    #[test]
    #[expect(
        clippy::too_many_lines,
        reason = "the production regression compares whole cached and uncached root lifecycles"
    )]
    fn dirty_canvas_child_reuses_the_unchanged_neighbor_with_exact_output() {
        use crate::decorations::plan::children::{
            ChildPlans, CHILD_MEASUREMENTS, DISABLE_CHILD_REUSE,
        };
        let plot = Plot::scatter(
            crate::NumericValues::new("x", [0.0, 1.0]),
            crate::NumericValues::new("y", [0.0, 1.0]),
        )
        .x_ticks(vec![
            crate::layout::AxisTick::new(0.0, "zero"),
            crate::layout::AxisTick::new(1.0, "one"),
        ])
        .panel_text_annotation(
            crate::PanelTextAnnotation::new(crate::PanelCorner::TopLeft, "annotation").unwrap(),
        )
        .legend(crate::LegendPolicy::Hidden);
        let canvas = Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, plot.clone())
            .unwrap()
            .panel(0, 1, plot)
            .unwrap()
            .size(1200, 800);
        let owner = crate::layout::ComponentAddress::root()
            .child(crate::layout::AddressRole::CanvasCell, 0)
            .unwrap();
        let neighbor = crate::layout::ComponentAddress::root()
            .child(crate::layout::AddressRole::CanvasCell, 1)
            .unwrap();
        let mut expected = None;
        for disabled in [true, false] {
            DISABLE_CHILD_REUSE.with(|value| value.set(disabled));
            CHILD_MEASUREMENTS.with(|events| events.borrow_mut().clear());
            let themes = Arc::new(ResolvedCanvasThemes::new(canvas.theme, &canvas.panels));
            let mut children = ChildPlans::canvas(&canvas);
            let mut calls = 0;
            let mut retained_neighbor = None;
            let root = crate::layout::plan_exact_root(
                canvas.width,
                canvas.height,
                |stage| {
                    let (interiors, _) = canvas.prepare_interiors(stage)?;
                    let (mut census, presence) =
                        canvas.foundational_text_census(&themes, &interiors, stage)?;
                    themes.prepare_font_environment(&mut census, presence, stage)?;
                    Ok(interiors)
                },
                |interiors, assigned, round| {
                    calls += 1;
                    let plan = canvas.build_canvas_plan(
                        Arc::clone(&themes),
                        assigned,
                        round,
                        &mut children,
                        &[],
                        interiors,
                    )?;
                    if calls == 2 {
                        retained_neighbor = Some(Arc::clone(&plan.panels[1].panel_plan));
                        round.request_domain(
                            &owner,
                            &owner,
                            FeedbackKind::AutomaticDomainX,
                            (-1.0, 2.0),
                        )?;
                    } else if calls == 3 {
                        assert_eq!(
                            Arc::ptr_eq(
                                retained_neighbor.as_ref().unwrap(),
                                &plan.panels[1].panel_plan
                            ),
                            !disabled
                        );
                    }
                    children.finish_round(round);
                    Ok(crate::layout::MeasuredLayout::new(assigned, plan))
                },
                |_, assigned, plan, stage| {
                    let (width, height) = assigned.dimensions();
                    let panels = plan
                        .panels
                        .iter()
                        .map(|panel| {
                            let plan = &panel.panel_plan;
                            let layout = &plan.result.layout;
                            let bounds = layout.projection.panel_box();
                            (
                                plan.panel_box,
                                layout.projection.x_range(),
                                layout.projection.y_range(),
                                layout.x_axis.ticks.clone(),
                                layout.y_axis.ticks.clone(),
                                plan.result
                                    .panel_text
                                    .iter()
                                    .map(|annotation| {
                                        (
                                            annotation.bounds_in(bounds),
                                            annotation.placement_in(bounds),
                                            annotation.text_plan.clone(),
                                        )
                                    })
                                    .collect::<Vec<_>>(),
                            )
                        })
                        .collect::<Vec<_>>();
                    let replay = crate::orchestrator::capture_text_replay_plan(
                        width,
                        height,
                        "child reuse test",
                        themes.root(),
                        stage,
                        |backend| canvas.render_planned_canvas(&plan, width, height, backend),
                    )?;
                    crate::orchestrator::validate_text_replay_plan(
                        width,
                        height,
                        &replay,
                        "child reuse test",
                    )?;
                    let text = replay
                        .retained_text()
                        .into_iter()
                        .map(|(placement, text)| (placement, text.clone()))
                        .collect::<Vec<_>>();
                    for source in ["annotation", "zero", "one"] {
                        assert!(text.iter().any(|(_, plan)| plan.source == source));
                    }
                    let mut svg = replay.take_svg_backend()?;
                    replay.replay(width, height, &mut svg, "child reuse test")?;
                    Ok((svg.into_bytes()?, text, panels))
                },
            )
            .unwrap();
            DISABLE_CHILD_REUSE.with(|value| value.set(false));
            assert_eq!(calls, 3);
            CHILD_MEASUREMENTS.with(|events| {
                let events = events.borrow();
                for phase in ["layout", "panel"] {
                    assert_eq!(
                        events
                            .iter()
                            .filter(|(address, stage)| address == &owner && *stage == phase)
                            .count(),
                        3,
                        "dirty {phase}"
                    );
                    assert_eq!(
                        events
                            .iter()
                            .filter(|(address, stage)| address == &neighbor && *stage == phase)
                            .count(),
                        if disabled { 3 } else { 2 },
                        "neighbor {phase}"
                    );
                }
            });
            if let Some(expected) = &expected {
                assert_eq!(root.plan(), expected);
            } else {
                expected = Some(root.plan().clone());
            }
        }
    }

    #[test]
    #[expect(
        clippy::too_many_lines,
        reason = "root-prepared mixed Canvas fixture checks every producer at first capture"
    )]
    fn canvas_producer_plans_survive_first_capture_without_shaping() {
        let source = format!(
            "{}AA\u{2029}A \u{202b}\u{5d0} A \u{5d0} \u{202c}A",
            "A\u{ad}".repeat(60)
        );
        for row_title in [false, true] {
            let plot = Plot::scatter(
                crate::NumericValues::new("x", [0.0, 1.0]),
                crate::NumericValues::new("y", [0.0, 1.0]),
            )
            .theme(Theme::Science);
            let plot = if row_title {
                plot
            } else {
                plot.title(&source).subtitle(&source)
            };
            let mut canvas = Canvas::grid(1, 2)
                .unwrap()
                .panel(0, 1, plot)
                .unwrap()
                .text_panel(0, 0, TextPanel::new(&source, &source).unwrap())
                .unwrap()
                .size(800, 1000)
                .row_header(0, 1, "A\u{301} shared row")
                .unwrap();
            if row_title {
                canvas = canvas.row_title(0, &source).unwrap();
            } else {
                canvas = canvas.column_header(0, 2, "A\u{301} shared").unwrap();
            }
            let themes = Arc::new(ResolvedCanvasThemes::new(canvas.theme, &canvas.panels));
            crate::layout::plan_exact_root(
                canvas.width,
                canvas.height,
                |stage| {
                    let (interiors, _) = canvas.prepare_interiors(stage)?;
                    let (mut census, presence) =
                        canvas.foundational_text_census(&themes, &interiors, stage)?;
                    themes.prepare_font_environment(&mut census, presence, stage)?;
                    Ok(interiors)
                },
                |interiors, assigned, round| {
                    Ok(crate::layout::MeasuredLayout::new(
                        assigned,
                        canvas.build_canvas_plan(
                            Arc::clone(&themes),
                            assigned,
                            round,
                            &mut crate::decorations::plan::children::ChildPlans::canvas(&canvas),
                            &[],
                            interiors,
                        )?,
                    ))
                },
                |_, assigned, plan, stage| {
                    let mut expected = Vec::new();
                    for panel in &plan.text_panels {
                        expected.push((panel.title_placement, panel.title.plan()));
                        expected.push((panel.body_placement, panel.body.plan()));
                    }
                    for text in plan.headers.iter().chain(&plan.row_titles) {
                        expected.push((text.text.placement, &text.text.plan));
                    }
                    for panel in &plan.panels {
                        for text in &panel.header {
                            expected.push((text.placement, &text.plan));
                        }
                    }
                    assert!(expected.iter().any(|(_, text)| text.source == source
                        && text
                            .lines
                            .iter()
                            .any(|line| line.selected_soft_hyphen.is_some())));
                    assert!(expected.iter().any(|(_, text)| text.source == source
                        && text
                            .lines
                            .iter()
                            .any(|line| line.runs.iter().any(|run| run.bidi_level % 2 == 1))));
                    crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| calls.set(0));
                    let (width, height) = assigned.dimensions();
                    let replay = crate::orchestrator::capture_text_replay_plan(
                        width,
                        height,
                        "Canvas producer closure",
                        themes.root(),
                        stage,
                        |backend| canvas.render_planned_canvas(&plan, width, height, backend),
                    )?;
                    crate::font::canonical::FONT_SHAPE_CALLS
                        .with(|calls| assert_eq!(calls.get(), 0, "first capture must not shape"));
                    let captured = replay.retained_text();
                    for (placement, text) in expected {
                        assert!(captured.iter().any(
                            |(actual_placement, actual)| *actual_placement == placement
                                && actual.shares_payload(text)
                                && *actual == text
                        ));
                    }
                    Ok(())
                },
            )
            .unwrap();
        }
    }

    #[test]
    fn shared_guide_recovery_measures_the_complete_canvas_body() {
        let plot = Plot::scatter(
            crate::NumericValues::new("x", [0.0, 1.0, 2.0, 3.0]),
            crate::NumericValues::new("y", [1.0, 2.0, 1.5, 2.5]),
        )
        .hue(
            crate::CategoryValues::new(
                "hue",
                [
                    "Complete explanatory first condition",
                    "Complete explanatory first condition",
                    "Complete explanatory second condition",
                    "Complete explanatory second condition",
                ],
            )
            .unwrap(),
        )
        .x_label("MMMMMMMMMMMMMMMMMMMMMMMMMMMM");
        let canvas = Canvas::grid(1, 1)
            .unwrap()
            .panel(0, 0, plot)
            .unwrap()
            .size(640, 480)
            .legend_policy(CanvasLegendPolicy::Shared);
        let themes = Arc::new(ResolvedCanvasThemes::new(canvas.theme, &canvas.panels));
        let root = crate::layout::plan_exact_root(
            canvas.width,
            canvas.height,
            |stage| {
                let (interiors, _) = canvas.prepare_interiors(stage)?;
                let (mut census, presence) =
                    canvas.foundational_text_census(&themes, &interiors, stage)?;
                themes.prepare_font_environment(&mut census, presence, stage)?;
                Ok(interiors)
            },
            |interiors, assigned, round| {
                Ok(crate::layout::MeasuredLayout::new(
                    assigned,
                    canvas.build_canvas_plan(
                        Arc::clone(&themes),
                        assigned,
                        round,
                        &mut crate::decorations::plan::children::ChildPlans::canvas(&canvas),
                        &[],
                        interiors,
                    )?,
                ))
            },
            |_, _, plan, _| Ok(plan),
        )
        .unwrap();
        let plan = root.plan();
        let guide = plan.shared_legend.as_ref().unwrap();
        assert!(guide.bounds.bottom > plan.panels[0].canvas_cell.y1);
        assert_eq!(
            plan.panels[0].panel_plan.result.layout.x_axis.label,
            "MMMMMMMMMMMMMMMMMMMMMMMMMMMM"
        );
    }

    #[test]
    #[expect(
        clippy::too_many_lines,
        reason = "exercise owned source variants and exact Canvas preparation budgets together"
    )]
    fn reference_canvas_owned_sources_obey_prepared_budgets() {
        use crate::{HistogramInput, HistogramKdePoint, NumericValues};

        for geometry in [false, true] {
            let mut previous = None;
            for count in [4_u32, 128] {
                let kde = (0..count)
                    .map(|index| HistogramKdePoint {
                        x: f64::from(index),
                        y: 0.1,
                    })
                    .collect();
                let plot = if geometry {
                    Plot::histogram(HistogramInput::geometry(crate::HistogramGeometry {
                        dimensions: crate::HistogramDimensions {
                            value: "sample".into(),
                            density: "density".into(),
                        },
                        bins: (0..count - 1)
                            .map(|index| crate::HistogramBin {
                                lower: f64::from(index),
                                upper: f64::from(index + 1),
                                density: 0.1,
                            })
                            .collect(),
                        kde,
                        rugs: (0..count)
                            .map(|index| crate::HistogramRug {
                                x: f64::from(index),
                                baseline: 0.0,
                                top: 0.01,
                            })
                            .collect(),
                    }))
                } else {
                    Plot::histogram(HistogramInput::samples(NumericValues::new(
                        "sample",
                        [0.0, 1.0, 2.0, 127.0],
                    )))
                    .stat(crate::HistogramStat::Density)
                    .composite(crate::HistogramComposite::KdeAndRug(
                        crate::HistogramKde::Precomputed(kde),
                    ))
                };
                let canvas = Canvas::grid(1, 2)
                    .unwrap()
                    .panel(
                        0,
                        0,
                        plot.x_ticks(vec![crate::AxisTick::new(150.0, "shared endpoint")]),
                    )
                    .unwrap()
                    .panel(
                        0,
                        1,
                        Plot::density(NumericValues::new("sample", [0.0, 100.0, 200.0])),
                    )
                    .unwrap()
                    .labels(crate::Labels::humanized().alias("sample", "Shared sample"))
                    .x_range_group([(0, 0), (0, 1)])
                    .unwrap();
                let prepare = |limits: Option<(u128, u64)>| {
                    crate::layout::plan_exact_root(
                        640,
                        480,
                        |stage| {
                            let resources = stage.generation_resources();
                            if let Some((visits, bytes)) = limits {
                                resources.set_test_limits(visits, bytes);
                            }
                            let (inputs, shared) = canvas.prepare_interiors(stage)?;
                            for (index, input) in inputs.iter().enumerate() {
                                let input = input.as_ref().unwrap();
                                assert_eq!(
                                    input.axes[0].0.as_deref(),
                                    Some(format!("panel-{index}").as_str())
                                );
                                assert_eq!(Some(input.axes[0].1.ranges[0]), shared[index]);
                                assert_eq!(input.axes[0].1.specs[0].limits, shared[index]);
                                assert_eq!(input.axes[0].1.labels[0], "Shared sample");
                            }
                            let usage = (
                                resources.usage().unwrap().1,
                                resources.byte_usage().unwrap(),
                            );
                            // This test bounds preparation, not the kernel's later feedback records.
                            resources.set_test_limits(u128::MAX, u64::MAX);
                            Ok(usage)
                        },
                        |_, assigned, _| Ok(crate::layout::MeasuredLayout::new(assigned, ())),
                        |usage, _, (), _| Ok(*usage),
                    )
                    .map(|root| *root.plan())
                };
                crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
                let (visits, bytes) = prepare(None).unwrap();
                for (limits, dimension) in [
                    ((visits - 1, bytes), "axis source work admission"),
                    ((visits, bytes - 1), "axis source storage admission"),
                ] {
                    let error = prepare(Some(limits)).unwrap_err();
                    assert!(matches!(error, Error::Render(_)), "{error}");
                    assert!(error.to_string().contains(dimension), "{error}");
                }
                assert_eq!(prepare(Some((visits, bytes))).unwrap(), (visits, bytes));
                if let Some((small_visits, small_bytes)) = previous {
                    assert!(visits > small_visits && bytes > small_bytes);
                    assert!(prepare(Some((small_visits, u64::MAX))).is_err());
                    assert!(prepare(Some((u128::MAX, small_bytes))).is_err());
                }
                previous = Some((visits, bytes));
                assert_eq!(
                    crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
                    0
                );
            }
        }
    }
}

struct ResolvedCanvasThemes {
    root: Box<ResolvedTheme>,
    plots: Vec<Option<ResolvedPlotTheme>>,
}

enum ResolvedPlotTheme {
    InheritedRoot,
    ExplicitPlot(Box<ResolvedTheme>),
}

impl ResolvedCanvasThemes {
    fn new(root: Theme, plots: &[Option<Plot>]) -> Self {
        let root = Box::new(root.resolve());
        let plots = plots
            .iter()
            .map(|plot| {
                plot.as_ref().map(|plot| {
                    plot.theme_spec()
                        .map_or(ResolvedPlotTheme::InheritedRoot, |theme| {
                            let mut theme = Box::new(theme.resolve());
                            theme.share_font_environment_from(&root);
                            ResolvedPlotTheme::ExplicitPlot(theme)
                        })
                })
            })
            .collect();
        Self { root, plots }
    }

    fn prepare_font_environment(
        &self,
        census: &mut crate::layout::PreparedTextCensus,
        presence: Vec<u64>,
        prepared_stage: &mut crate::layout::kernel::PreparedStage<'_>,
    ) -> Result<(), Error> {
        self.root
            .prepare_font_environment_from_census(census, presence, prepared_stage)
    }

    fn root(&self) -> &ResolvedTheme {
        self.root.as_ref()
    }

    fn plot(&self, index: usize) -> Option<&ResolvedTheme> {
        self.plot_selection(index).map(|theme| match theme {
            ResolvedPlotTheme::InheritedRoot => self.root(),
            ResolvedPlotTheme::ExplicitPlot(theme) => theme.as_ref(),
        })
    }

    fn plot_selection(&self, index: usize) -> Option<&ResolvedPlotTheme> {
        self.plots.get(index)?.as_ref()
    }
}

struct CanvasHeaderPlan {
    text: crate::render::backend::PlacedText,
}

/// Categorical tick-label policy for composed canvas grids.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CanvasCategoryLabels {
    /// Render categorical tick labels in every panel.
    #[default]
    EveryPlot,
    /// Render categorical tick labels only in the first column of each row.
    FirstColumnPerRow,
}

#[derive(Debug, Clone)]
struct CanvasHeader {
    start: usize,
    span: usize,
    text: String,
}

struct TextPanelPlan {
    index: usize,
    cell: PanelBox,
    title: crate::layout::TextBlock,
    body: crate::layout::TextBlock,
    title_placement: crate::render::backend::PlannedTextPlacement,
    body_placement: crate::render::backend::PlannedTextPlacement,
}

/// Validated prose content for one composed-canvas grid cell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextPanel {
    title: String,
    body: String,
}

impl TextPanel {
    /// Construct a text panel with nonempty title and body.
    pub fn new(title: impl Into<String>, body: impl Into<String>) -> Result<Self, Error> {
        let title = title.into();
        let body = body.into();
        validate_text_panel_value(&title, "title")?;
        validate_text_panel_value(&body, "body")?;
        Ok(Self { title, body })
    }

    /// Panel title rendered above the body.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Panel body rendered with deterministic wrapping.
    #[must_use]
    pub fn body(&self) -> &str {
        &self.body
    }
}

fn validate_text_panel_value(value: &str, role: &str) -> Result<(), Error> {
    if value.trim().is_empty() || value.trim() != value {
        return Err(Error::Data(format!(
            "text panel {role} must be nonempty and have no surrounding whitespace"
        )));
    }
    if value.chars().any(char::is_control) {
        return Err(Error::Data(format!(
            "text panel {role} must not contain control characters"
        )));
    }
    Ok(())
}

fn canvas_header(
    start: usize,
    span: usize,
    text: String,
    limit: usize,
    role: &str,
) -> Result<CanvasHeader, Error> {
    if span == 0 || start.checked_add(span).is_none_or(|end| end > limit) {
        return Err(Error::Data(format!(
            "canvas {role} header range {start}..{} is outside 0..{limit}",
            start.saturating_add(span)
        )));
    }
    if text.trim().is_empty() || text.trim() != text || text.chars().any(char::is_control) {
        return Err(Error::Data(format!(
            "canvas {role} header must be nonempty, trimmed, and free of control characters"
        )));
    }
    Ok(CanvasHeader { start, span, text })
}

fn validate_header_overlap(
    headers: &[CanvasHeader],
    candidate: &CanvasHeader,
    role: &str,
) -> Result<(), Error> {
    let end = candidate.start + candidate.span;
    if headers.iter().any(|header| {
        let other_end = header.start + header.span;
        candidate.start < other_end && header.start < end
    }) {
        return Err(Error::Data(format!(
            "canvas {role} headers must not overlap"
        )));
    }
    Ok(())
}

/// A typed grid of independently configured charts rendered on one canvas.
///
/// Canvas owns one root theme, which defaults to [`Theme::Publication`]. Plots
/// without an explicit theme inherit that root.
pub struct Canvas {
    rows: usize,
    columns: usize,
    panels: Vec<Option<Plot>>,
    text_panels: Vec<Option<TextPanel>>,
    theme: Theme,
    legend_policy: CanvasLegendPolicy,
    shared_legend_gap: Option<f64>,
    width: u32,
    height: u32,
    title: Option<String>,
    subtitle: Option<String>,
    // Stored independently of root headings; composition consumes these at T5b.
    #[allow(dead_code)]
    pub(crate) panel_title_alignment: Option<crate::theme::TitleAlignment>,
    #[allow(dead_code)]
    pub(crate) panel_subtitle_alignment: Option<crate::theme::TitleAlignment>,
    caption: Option<String>,
    labels: Option<Labels>,
    category_labels: CanvasCategoryLabels,
    column_headers: Vec<CanvasHeader>,
    row_headers: Vec<CanvasHeader>,
    row_titles: Vec<CanvasHeader>,
    x_range_groups: Vec<Vec<usize>>,
}

fn push_text_requirement(
    requirements: &mut Vec<crate::layout::PreparedTextRequirement>,
    address: crate::layout::ComponentAddress,
    role: crate::layout::PreparedTextRole,
    local_index: u32,
    source: &str,
    appearance: &crate::theme::TextAppearance,
    prepared_stage: &mut crate::layout::kernel::PreparedStage<'_>,
) -> Result<(), Error> {
    crate::layout::PreparedTextRequirement::push_into(
        requirements,
        address,
        role,
        local_index,
        source,
        &[],
        appearance,
        Some(prepared_stage),
    )
    .map_err(|error| Error::Render(format!("prepared canvas text census: {error}")))
}

impl Canvas {
    /// Construct a non-empty panel grid.
    pub fn grid(rows: usize, columns: usize) -> Result<Self, Error> {
        if rows == 0 || columns == 0 {
            return Err(Error::Data(
                "canvas grid rows and columns must both be greater than zero".into(),
            ));
        }
        let panel_count = rows
            .checked_mul(columns)
            .ok_or_else(|| Error::Data("canvas grid dimensions exceed allocation limits".into()))?;
        Ok(Self {
            rows,
            columns,
            panels: (0..panel_count).map(|_| None).collect(),
            text_panels: (0..panel_count).map(|_| None).collect(),
            theme: Theme::Publication,
            legend_policy: CanvasLegendPolicy::PerPlot,
            shared_legend_gap: None,
            width: 960,
            height: 640,
            title: None,
            subtitle: None,
            panel_title_alignment: None,
            panel_subtitle_alignment: None,
            caption: None,
            labels: None,
            category_labels: CanvasCategoryLabels::EveryPlot,
            column_headers: Vec::new(),
            row_headers: Vec::new(),
            row_titles: Vec::new(),
            x_range_groups: Vec::new(),
        })
    }

    /// Place one exact-family chart in an unoccupied zero-based grid cell.
    ///
    /// An already erased plot cannot be inserted:
    ///
    /// ```compile_fail
    /// use lichtung_plot::{Canvas, NumericValues, Plot};
    /// let erased: Plot = Plot::scatter(
    ///     NumericValues::new("x", [1.0]),
    ///     NumericValues::new("y", [2.0]),
    /// ).erase();
    /// let _ = Canvas::grid(1, 1).unwrap().panel(0, 0, erased);
    /// ```
    pub fn panel<K: crate::kind::ExactFamily>(
        mut self,
        row: usize,
        column: usize,
        plot: Plot<K>,
    ) -> Result<Self, Error> {
        if row >= self.rows || column >= self.columns {
            return Err(Error::Data(format!(
                "canvas panel ({row}, {column}) is outside {}x{} grid",
                self.rows, self.columns
            )));
        }
        let index = row * self.columns + column;
        if self.panels[index].is_some() || self.text_panels[index].is_some() {
            return Err(Error::Data(format!(
                "canvas panel ({row}, {column}) is already occupied"
            )));
        }
        self.panels[index] = Some(plot.erase());
        Ok(self)
    }

    /// Place one validated text panel in an unoccupied zero-based grid cell.
    pub fn text_panel(
        mut self,
        row: usize,
        column: usize,
        panel: TextPanel,
    ) -> Result<Self, Error> {
        if row >= self.rows || column >= self.columns {
            return Err(Error::Data(format!(
                "canvas panel ({row}, {column}) is outside {}x{} grid",
                self.rows, self.columns
            )));
        }
        let index = row * self.columns + column;
        if self.panels[index].is_some() || self.text_panels[index].is_some() {
            return Err(Error::Data(format!(
                "canvas panel ({row}, {column}) is already occupied"
            )));
        }
        self.text_panels[index] = Some(panel);
        Ok(self)
    }

    /// Set per-panel, shared, or hidden legend behavior.
    #[must_use]
    pub fn legend_policy(mut self, policy: CanvasLegendPolicy) -> Self {
        self.legend_policy = policy;
        self
    }

    /// Set the exact CSS-pixel gap between the panel grid and a shared legend.
    pub fn shared_legend_gap(mut self, gap: f64) -> Result<Self, Error> {
        if !gap.is_finite() || gap < 0.0 {
            return Err(Error::Data(
                "canvas shared legend gap must be finite and nonnegative".into(),
            ));
        }
        self.shared_legend_gap = Some(gap);
        Ok(self)
    }

    /// Add one measured header spanning contiguous canvas columns.
    pub fn column_header(
        mut self,
        start_column: usize,
        span: usize,
        text: impl Into<String>,
    ) -> Result<Self, Error> {
        let header = canvas_header(start_column, span, text.into(), self.columns, "column")?;
        validate_header_overlap(&self.column_headers, &header, "column")?;
        self.column_headers.push(header);
        Ok(self)
    }

    /// Add one measured header spanning contiguous canvas rows.
    pub fn row_header(
        mut self,
        start_row: usize,
        span: usize,
        text: impl Into<String>,
    ) -> Result<Self, Error> {
        let header = canvas_header(start_row, span, text.into(), self.rows, "row")?;
        validate_header_overlap(&self.row_headers, &header, "row")?;
        self.row_headers.push(header);
        Ok(self)
    }

    /// Add one measured title centered above every panel in one canvas row.
    pub fn row_title(mut self, row: usize, text: impl Into<String>) -> Result<Self, Error> {
        let title = canvas_header(row, 1, text.into(), self.rows, "row title")?;
        validate_header_overlap(&self.row_titles, &title, "row title")?;
        self.row_titles.push(title);
        Ok(self)
    }

    /// Configure duplicate categorical-label handling across the grid.
    #[must_use]
    pub const fn category_labels(mut self, policy: CanvasCategoryLabels) -> Self {
        self.category_labels = policy;
        self
    }

    /// Couple the automatic continuous horizontal range of explicit panel cells.
    ///
    /// Grouped cells must contain configured plots without explicit x limits or
    /// categorical x domains.
    pub fn x_range_group(
        mut self,
        cells: impl IntoIterator<Item = (usize, usize)>,
    ) -> Result<Self, Error> {
        let mut indices = Vec::new();
        for (row, column) in cells {
            if row >= self.rows || column >= self.columns {
                return Err(Error::Data(format!(
                    "canvas x-range group cell ({row}, {column}) is outside {}x{} grid",
                    self.rows, self.columns
                )));
            }
            let index = row * self.columns + column;
            if indices.contains(&index)
                || self
                    .x_range_groups
                    .iter()
                    .any(|group| group.contains(&index))
            {
                return Err(Error::Data(format!(
                    "canvas x-range group repeats panel ({row}, {column})"
                )));
            }
            indices.push(index);
        }
        if indices.is_empty() {
            return Err(Error::Data(
                "canvas x-range group must contain at least one panel".into(),
            ));
        }
        self.x_range_groups.push(indices);
        Ok(self)
    }

    /// Set the composed canvas dimensions.
    #[must_use]
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Set one canvas-level title.
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set one canvas-level subtitle.
    #[must_use]
    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// Set one canvas-level caption.
    #[must_use]
    pub fn caption(mut self, caption: impl Into<String>) -> Self {
        self.caption = Some(caption.into());
        self
    }

    /// Apply one reusable automatic-label policy to every configured panel.
    #[must_use]
    pub fn labels(mut self, labels: Labels) -> Self {
        self.labels = Some(labels);
        self
    }

    /// Set the Canvas root theme.
    ///
    /// Plots without an explicit theme inherit this theme. A theme selected with
    /// [`Plot::theme`] completely replaces it for that Plot. If this method is
    /// omitted, the Canvas root uses [`Theme::Publication`].
    #[must_use]
    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    fn render_preflighted(
        root: &crate::layout::FrozenRoot<crate::orchestrator::TextReplayPlan, PreparedCanvasInputs>,
        backend: &mut dyn Backend,
    ) -> Result<(), Error> {
        let _ = &root.prepared().text_census;
        let (width, height) = root.dimensions();
        root.plan()
            .replay(width, height, backend, "prepared canvas scene replay")
    }

    /// Validate the complete composed canvas without opening a backend or creating files.
    ///
    /// This checks canvas and grid layout, every chart's plot-specific data and
    /// representability preflight, channel and interval contracts, shared legend
    /// compatibility, and nested self-managed plot restrictions.
    pub fn preflight(&self) -> Result<(), Error> {
        self.preflight_plan().map(drop)
    }

    #[cfg(test)]
    pub(crate) fn render_with_backend(&self, backend: &mut dyn Backend) -> Result<(), Error> {
        let preflight = self.preflight_plan()?;
        Self::render_preflighted(&preflight, backend)
    }

    /// Render one immutable canonical composed SVG scene without creating a file.
    ///
    /// # Errors
    ///
    /// Returns an error when canvas preflight or SVG rendering fails.
    fn render_scene(&self) -> Result<RenderedScene, Error> {
        #[cfg(test)]
        CANVAS_SCENE_RENDER_CALLS.with(|calls| calls.set(calls.get() + 1));
        let preflight = self.preflight_plan()?;
        let (width, height) = preflight.dimensions();
        let mut backend = preflight.plan().take_svg_backend()?;
        Self::render_preflighted(&preflight, &mut backend)?;
        RenderedScene::new(width, height, backend.into_output())
    }

    /// Render once, publish an outlined SVG, and return a reusable output.
    /// The output filename is completed with `.svg` after conversion.
    pub fn svg(self, path: impl AsRef<Path>) -> Result<Output, Error> {
        let output = Output::new(self.render_scene()?);
        output.svg(path)
    }

    /// Render once, publish SVG with selectable text, and return a reusable output.
    /// The output filename is completed with `.svg` after rendering.
    pub fn svg_preserve_text(self, path: impl AsRef<Path>) -> Result<Output, Error> {
        let output = Output::new(self.render_scene()?);
        output.svg_preserve_text(path)
    }

    /// Render once, publish a balanced 300-DPI PNG, and return a reusable output.
    /// The output filename is completed with `.png` after rasterization.
    pub fn png(self, path: impl AsRef<Path>) -> Result<Output, Error> {
        let output = Output::new(self.render_scene()?);
        output.png(path)
    }

    /// Render once, publish a balanced source-sized PNG without density metadata.
    /// The output filename is completed with `.png` after rasterization.
    pub fn png_preview(self, path: impl AsRef<Path>) -> Result<Output, Error> {
        let output = Output::new(self.render_scene()?);
        output.png_preview(path)
    }

    /// Render once and publish a PNG using the exact caller-provided options.
    /// The output filename is completed with `.png` after rasterization.
    pub fn png_with_options(
        self,
        path: impl AsRef<Path>,
        options: crate::PngOptions,
    ) -> Result<Output, Error> {
        let output = Output::new(self.render_scene()?);
        output.png_with_options(path, options)
    }

    fn validate_structure(&self) -> Result<(), Error> {
        if self.width < 160 || self.height < 120 {
            return Err(Error::Data("canvas must be at least 160x120 pixels".into()));
        }
        if self.panels.iter().all(Option::is_none) && self.text_panels.iter().all(Option::is_none) {
            return Err(Error::Data(
                "canvas requires at least one configured panel".into(),
            ));
        }
        if self.shared_legend_gap.is_some() && self.legend_policy != CanvasLegendPolicy::Shared {
            return Err(Error::Data(
                "canvas shared legend gap requires CanvasLegendPolicy::Shared".into(),
            ));
        }
        for plot in self.panels.iter().flatten() {
            let kind = plot.kind();
            plot.validate_configuration(kind)?;
            crate::layout::interior::ParentCompositionAdapter::Canvas.validate_child(plot)?;
        }
        for group in &self.x_range_groups {
            for &index in group {
                let plot = self.panels[index].as_ref().ok_or_else(|| {
                    Error::Data(format!(
                        "canvas x-range group references unconfigured panel {index}"
                    ))
                })?;
                if plot.user_x_limits().is_some() {
                    return Err(Error::Data(format!(
                        "canvas x-range group panel {index} has explicit x limits"
                    )));
                }
                if plot
                    .automatic_axis_category_values(plot.kind(), true)
                    .is_some()
                {
                    return Err(Error::Data(format!(
                        "canvas x-range group panel {index} has a categorical x axis"
                    )));
                }
            }
        }
        for title in &self.row_titles {
            let start = title.start * self.columns;
            let end = start + self.columns;
            if self.panels[start..end].iter().flatten().any(|plot| {
                plot.title_text().is_some_and(|text| !text.is_empty())
                    || plot.subtitle_text().is_some_and(|text| !text.is_empty())
            }) {
                return Err(Error::Data(format!(
                    "canvas row title cannot coexist with a panel title or subtitle in row {}",
                    title.start
                )));
            }
        }
        Ok(())
    }

    fn prepare_shared_x(
        &self,
        stage: &mut crate::layout::kernel::PreparedStage<'_>,
        scatter: &[Option<crate::plots::scatter::PreparedScatter>],
        line: &[Option<crate::plots::line::PreparedLine>],
        reference: &[Option<crate::plots::shared_x::PreparedReference>],
    ) -> Result<Vec<Option<(f64, f64)>>, Error> {
        let mut ranges = Vec::new();
        if self.x_range_groups.is_empty() {
            return Ok(ranges);
        }
        stage.claim_topology_slots(self.panels.len() as u128)?;
        stage.reserve_prepared(&mut ranges, self.panels.len(), "shared x domains")?;
        ranges.resize(self.panels.len(), None);
        for group in &self.x_range_groups {
            let mut shared: Option<(f64, f64)> = None;
            for &index in group {
                let plot = self.panels[index]
                    .as_ref()
                    .expect("validated shared x member");
                let range = if let Some(marks) = &reference[index] {
                    marks.x_range(
                        plot,
                        plot.data()
                            .with_axis_resources(&stage.generation_resources()),
                    )?
                } else if plot.kind() == PlotKind::Scatter {
                    crate::layout::continuous_auto_range(
                        scatter[index]
                            .as_ref()
                            .ok_or_else(|| {
                                Error::Render(
                                    "shared Scatter range requires root-prepared inputs".into(),
                                )
                            })?
                            .ranges[0],
                        plot.x_axis_spec(),
                    )
                } else if matches!(
                    plot.kind(),
                    PlotKind::Line | PlotKind::Trajectory | PlotKind::LinePath
                ) {
                    let marks = line[index].as_ref().ok_or_else(|| {
                        Error::Render("shared line range requires root-prepared inputs".into())
                    })?;
                    if marks.conditions.is_some() {
                        return Err(Error::Data(
                            "canvas x-range group has a categorical x axis".into(),
                        ));
                    }
                    marks.ranges[0]
                } else {
                    crate::plots::shared_x::domain(plot, &stage.generation_resources())?
                };
                shared = Some(
                    shared.map_or(range, |prior| (prior.0.min(range.0), prior.1.max(range.1))),
                );
            }
            let shared = shared.expect("validated nonempty shared x group");
            for &index in group {
                let plot = self.panels[index]
                    .as_ref()
                    .expect("validated shared x member");
                if let Some(ticks) = plot.explicit_x_ticks() {
                    for tick in ticks {
                        let position = if plot.kind() == PlotKind::Forest
                            && plot.axis_scale_spec() == crate::AxisScale::Log10
                        {
                            tick.position.log10()
                        } else {
                            tick.position
                        };
                        if !position.is_finite() || position < shared.0 || position > shared.1 {
                            return Err(Error::Data(format!("x-axis explicit tick position {} is outside resolved shared range {}..{}", tick.position, shared.0, shared.1)));
                        }
                    }
                }
                ranges[index] = Some(shared);
            }
        }
        Ok(ranges)
    }

    fn row_title_plan(
        &self,
        title: &CanvasHeader,
        cells: &[PanelBox],
        theme: &crate::theme::ResolvedTheme,
    ) -> Result<crate::layout::ResolvedTextPlan, Error> {
        let first = title.start * self.columns;
        let last = first + self.columns - 1;
        let maximum_width = (cells[last].x1 - cells[first].x0 - 16.0).max(1.0);
        theme.plan_legacy_text(
            &title.text,
            theme.composition_title_font_size,
            maximum_width,
            PhysicalTextAlignment::Center,
            "canvas row title",
        )
    }

    fn add_row_title_requirements(
        &self,
        requirements: &mut Vec<(usize, usize, PanelRequirements)>,
        cells: &[PanelBox],
        theme: &crate::theme::ResolvedTheme,
    ) -> Result<(), Error> {
        for title in &self.row_titles {
            let plan = self.row_title_plan(title, cells, theme)?;
            let top = plan.height() + 8.0;
            requirements.push((
                title.start,
                0,
                PanelRequirements {
                    top,
                    ..PanelRequirements::default()
                },
            ));
        }
        Ok(())
    }

    #[expect(
        clippy::cast_precision_loss,
        reason = "wrapped row-title line indices are bounded by the finite canvas"
    )]
    fn plan_row_titles(
        &self,
        cells: &[PanelBox],
        theme: &crate::theme::ResolvedTheme,
    ) -> Result<Vec<CanvasHeaderPlan>, Error> {
        let mut plans = Vec::new();
        crate::render::backend::reserve_text_records(
            &mut plans,
            self.row_titles.len(),
            theme.prepared_font_environment()?,
        )?;
        for title in &self.row_titles {
            let first = title.start * self.columns;
            let last = first + self.columns - 1;
            let x = f64::midpoint(cells[first].x0, cells[last].x1);
            let plan = self.row_title_plan(title, cells, theme)?;
            let y = cells[first].y1
                + 14.0
                + plan.lines.len().saturating_sub(1) as f64 * plan.line_height();
            plans.push(CanvasHeaderPlan {
                text: crate::render::backend::PlacedText::from_deferred_first_line(
                    plan,
                    Point2D::new(x.round(), y.round()),
                    crate::render::backend::TextAnchor::Middle,
                    0.0,
                ),
            });
        }
        Ok(plans)
    }

    #[expect(
        clippy::type_complexity,
        clippy::too_many_lines,
        reason = "root component inputs and their coordinated x domains are prepared together before fonts"
    )]
    fn prepare_interiors(
        &self,
        stage: &mut crate::layout::kernel::PreparedStage<'_>,
    ) -> Result<
        (
            Vec<Option<crate::layout::interior::PreparedInteriorInputs>>,
            Vec<Option<(f64, f64)>>,
        ),
        Error,
    > {
        let mut interiors = Vec::new();
        stage.reserve_prepared(&mut interiors, self.panels.len(), "Canvas interior inputs")?;
        let resources = stage.generation_resources();
        let mut scatter = Vec::new();
        stage.reserve_prepared(&mut scatter, self.panels.len(), "Canvas Scatter semantics")?;
        let mut line = Vec::new();
        stage.reserve_prepared(&mut line, self.panels.len(), "Canvas line semantics")?;
        let mut reference = Vec::new();
        stage.reserve_prepared(
            &mut reference,
            self.panels.len(),
            "Canvas distribution/reference semantics",
        )?;
        for plot in &self.panels {
            reference.push(
                match plot.as_ref().filter(|plot| {
                    matches!(
                        plot.kind(),
                        PlotKind::Histogram
                            | PlotKind::Density
                            | PlotKind::Qq
                            | PlotKind::Regression
                            | PlotKind::Dumbbell
                            | PlotKind::PointInterval
                            | PlotKind::Coefficient
                            | PlotKind::Forest
                            | PlotKind::Bar
                            | PlotKind::HorizontalBar
                            | PlotKind::StackedBar
                            | PlotKind::StackedHorizontalBar
                            | PlotKind::Boxplot
                            | PlotKind::Violin
                            | PlotKind::Raincloud
                            | PlotKind::Interaction
                            | PlotKind::Heatmap
                            | PlotKind::Marginal
                    )
                }) {
                    Some(plot) => {
                        let data = plot
                            .configured_source_rows()?
                            .map_or_else(|| plot.data(), |rows| plot.data().map_source_rows(rows))
                            .with_axis_resources(&resources);
                        Some(crate::plots::shared_x::PreparedReference::new(plot, data)?)
                    }
                    None => None,
                },
            );
            line.push(
                match plot.as_ref().filter(|plot| {
                    matches!(
                        plot.kind(),
                        PlotKind::Line | PlotKind::Trajectory | PlotKind::LinePath
                    )
                }) {
                    Some(plot) => {
                        let data = plot
                            .configured_source_rows()?
                            .map_or_else(|| plot.data(), |rows| plot.data().map_source_rows(rows))
                            .with_axis_resources(&resources);
                        Some(crate::plots::line::PreparedLine::new(plot, data, None)?)
                    }
                    None => None,
                },
            );
            scatter.push(
                match plot
                    .as_ref()
                    .filter(|plot| plot.kind() == PlotKind::Scatter)
                {
                    Some(plot) => {
                        let data = plot
                            .configured_source_rows()?
                            .map_or_else(|| plot.data(), |rows| plot.data().map_source_rows(rows))
                            .with_axis_resources(&resources);
                        Some(crate::plots::scatter::PreparedScatter::new(plot, data)?)
                    }
                    None => None,
                },
            );
        }
        let shared_x = self.prepare_shared_x(stage, &scatter, &line, &reference)?;
        for (index, plot) in self.panels.iter().enumerate() {
            interiors.push(
                match plot.as_ref().filter(|plot| {
                    matches!(
                        plot.kind(),
                        PlotKind::Scatter
                            | PlotKind::Histogram
                            | PlotKind::Density
                            | PlotKind::Qq
                            | PlotKind::Regression
                            | PlotKind::Dumbbell
                            | PlotKind::PointInterval
                            | PlotKind::Coefficient
                            | PlotKind::Forest
                            | PlotKind::Bar
                            | PlotKind::HorizontalBar
                            | PlotKind::StackedBar
                            | PlotKind::StackedHorizontalBar
                            | PlotKind::Boxplot
                            | PlotKind::Violin
                            | PlotKind::Raincloud
                            | PlotKind::Interaction
                            | PlotKind::Heatmap
                            | PlotKind::Marginal
                            | PlotKind::Line
                            | PlotKind::Trajectory
                            | PlotKind::LinePath
                    )
                }) {
                    Some(plot) if reference[index].is_some() => {
                        let data = plot.data().with_axis_resources(&resources);
                        let scope_bytes = 6 + index
                            .checked_ilog10()
                            .map_or(1, |digits| digits as usize + 1);
                        let mut scope = String::new();
                        data.reserve_source_string(&mut scope, scope_bytes)?;
                        std::fmt::Write::write_fmt(&mut scope, format_args!("panel-{index}"))
                            .map_err(|error| Error::Render(format!("Canvas scope: {error}")))?;
                        Some(crate::plots::shared_x::prepare_reference_plot(
                            plot,
                            &resources,
                            reference[index]
                                .take()
                                .expect("prepared reference component"),
                            Some(&scope),
                            shared_x.get(index).copied().flatten(),
                            self.labels.as_ref(),
                        )?)
                    }
                    Some(plot) => {
                        let plot = self.labels.as_ref().map_or_else(
                            || plot.clone(),
                            |labels| plot.clone().labels(labels.clone()),
                        );
                        let mut plot = plot.with_render_scope(format!("panel-{index}"));
                        if let Some(Some(range)) = shared_x.get(index) {
                            plot = plot.with_planned_x_limits(*range);
                        }
                        Some(if plot.kind() == PlotKind::Scatter {
                            crate::plots::shared_x::prepare_scatter_plot(
                                &plot,
                                &resources,
                                scatter[index].take().expect("prepared Scatter component"),
                            )?
                        } else {
                            crate::plots::shared_x::prepare_line_plot(
                                &plot,
                                &resources,
                                line[index].take().expect("prepared line component"),
                            )?
                        })
                    }
                    None => None,
                },
            );
        }
        Ok((interiors, shared_x))
    }

    fn preflight_plan(
        &self,
    ) -> Result<
        crate::layout::FrozenRoot<crate::orchestrator::TextReplayPlan, PreparedCanvasInputs>,
        Error,
    > {
        let mut children = crate::decorations::plan::children::ChildPlans::canvas(self);
        let root = crate::layout::plan_exact_root(
            self.width,
            self.height,
            |prepared_stage| {
                let themes = Arc::new(ResolvedCanvasThemes::new(self.theme, &self.panels));
                #[cfg(test)]
                {
                    record_canvas_theme(usize::MAX, "resolved", themes.root());
                    for index in 0..themes.plots.len() {
                        if let Some(theme) = themes.plot(index) {
                            record_canvas_theme(index, "resolved", theme);
                        }
                    }
                }
                self.validate_structure()?;
                let (interiors, shared_x) = self.prepare_interiors(prepared_stage)?;
                let (mut census, scalar_presence) =
                    self.foundational_text_census(&themes, &interiors, prepared_stage)?;
                themes.prepare_font_environment(&mut census, scalar_presence, prepared_stage)?;
                Ok(PreparedCanvasInputs {
                    themes,
                    text_census: census,
                    shared_x,
                    interiors,
                })
            },
            |prepared, assigned, round| {
                let plan = self.build_canvas_plan(
                    Arc::clone(&prepared.themes),
                    assigned,
                    round,
                    &mut children,
                    &prepared.shared_x,
                    &prepared.interiors,
                )?;
                children.finish_round(round);
                Ok(crate::layout::MeasuredLayout::new(assigned, Box::new(plan)))
            },
            |_, assigned, plan, final_stage| {
                let (width, height) = assigned.dimensions();
                crate::orchestrator::capture_text_replay_plan(
                    width,
                    height,
                    "prepared canvas scene capture",
                    plan.themes.root(),
                    final_stage,
                    |backend| self.render_planned_canvas(&plan, width, height, backend),
                )
            },
        )?;
        let (width, height) = root.dimensions();
        crate::orchestrator::validate_text_replay_plan(
            width,
            height,
            root.plan(),
            "prepared canvas Null replay",
        )?;
        Ok(root)
    }

    #[expect(
        clippy::too_many_lines,
        reason = "the Canvas census keeps canonical root, composition, and panel preorder explicit"
    )]
    fn foundational_text_census(
        &self,
        themes: &ResolvedCanvasThemes,
        interiors: &[Option<crate::layout::interior::PreparedInteriorInputs>],
        prepared_stage: &mut crate::layout::kernel::PreparedStage<'_>,
    ) -> Result<(crate::layout::PreparedTextCensus, Vec<u64>), Error> {
        let root = crate::layout::ComponentAddress::root();
        let mut requirements = Vec::new();
        for (source, role, address_role, appearance) in [
            (
                self.title.as_deref(),
                crate::layout::PreparedTextRole::Title,
                crate::layout::AddressRole::RootTitle,
                &themes.root().typography.title,
            ),
            (
                self.subtitle.as_deref(),
                crate::layout::PreparedTextRole::Subtitle,
                crate::layout::AddressRole::RootSubtitle,
                &themes.root().typography.subtitle,
            ),
            (
                self.caption.as_deref(),
                crate::layout::PreparedTextRole::Caption,
                crate::layout::AddressRole::RootCaption,
                &themes.root().typography.caption,
            ),
        ] {
            if let Some(source) = source.filter(|source| !source.is_empty()) {
                let address = root.prepared_child(address_role, 0, prepared_stage)?;
                push_text_requirement(
                    &mut requirements,
                    address,
                    role,
                    0,
                    source,
                    appearance,
                    prepared_stage,
                )?;
            }
        }

        let composition_title = themes.root().legacy_text_appearance(
            themes.root().composition_title_font_size,
            crate::TextWeight::Normal,
        );
        let body = themes
            .root()
            .legacy_text_appearance(themes.root().label_font_size, crate::TextWeight::Normal);
        for (index, header) in self.column_headers.iter().enumerate() {
            let address = root.prepared_child(
                crate::layout::AddressRole::CanvasHeader,
                u32::try_from(index).map_err(|_| {
                    Error::Render("canvas column-header census index overflowed".into())
                })?,
                prepared_stage,
            )?;
            push_text_requirement(
                &mut requirements,
                address,
                crate::layout::PreparedTextRole::Header,
                0,
                &header.text,
                &composition_title,
                prepared_stage,
            )?;
        }
        for (index, header) in self.row_headers.iter().enumerate() {
            let address = root.prepared_child(
                crate::layout::AddressRole::CanvasHeader,
                u32::try_from(self.column_headers.len() + index).map_err(|_| {
                    Error::Render("canvas row-header census index overflowed".into())
                })?,
                prepared_stage,
            )?;
            push_text_requirement(
                &mut requirements,
                address,
                crate::layout::PreparedTextRole::Header,
                0,
                &header.text,
                &composition_title,
                prepared_stage,
            )?;
        }
        for (index, title) in self.row_titles.iter().enumerate() {
            let address = root.prepared_child(
                crate::layout::AddressRole::CanvasRowTitle,
                u32::try_from(index).map_err(|_| {
                    Error::Render("canvas row-title census index overflowed".into())
                })?,
                prepared_stage,
            )?;
            push_text_requirement(
                &mut requirements,
                address,
                crate::layout::PreparedTextRole::Header,
                0,
                &title.text,
                &composition_title,
                prepared_stage,
            )?;
        }
        for (index, panel) in self.text_panels.iter().enumerate() {
            let Some(panel) = panel else { continue };
            let address = root.prepared_child(
                crate::layout::AddressRole::TextPanelContent,
                u32::try_from(index)
                    .map_err(|_| Error::Render("text-panel census index overflowed".into()))?,
                prepared_stage,
            )?;
            for (role, local_index, source, appearance) in [
                (
                    crate::layout::PreparedTextRole::Header,
                    0,
                    panel.title(),
                    &composition_title,
                ),
                (
                    crate::layout::PreparedTextRole::Body,
                    0,
                    panel.body(),
                    &body,
                ),
            ] {
                let requirement_address = address.prepared_clone(prepared_stage)?;
                push_text_requirement(
                    &mut requirements,
                    requirement_address,
                    role,
                    local_index,
                    source,
                    appearance,
                    prepared_stage,
                )?;
            }
        }

        for (index, plot) in self.panels.iter().enumerate() {
            let Some(plot) = plot else { continue };
            let plot = self.labels.as_ref().map_or_else(
                || plot.clone(),
                |labels| plot.clone().labels(labels.clone()),
            );
            let plot = if self.legend_policy == CanvasLegendPolicy::Hidden {
                plot.with_suppressed_legend()
            } else {
                plot
            };
            let theme = themes
                .plot(index)
                .expect("configured canvas panel has a resolved theme");
            let cell = root.prepared_child(
                crate::layout::AddressRole::CanvasCell,
                u32::try_from(index)
                    .map_err(|_| Error::Render("canvas text census index overflowed".into()))?,
                prepared_stage,
            )?;
            let filter_point_label_x_to_limits = !self
                .x_range_groups
                .iter()
                .any(|group| group.contains(&index));
            crate::layout::PreparedTextRequirement::append_plot(
                interiors
                    .get(index)
                    .and_then(Option::as_ref)
                    .and_then(|inputs| inputs.heatmap.as_deref()),
                &mut requirements,
                &plot,
                plot.kind(),
                theme,
                &cell,
                false,
                filter_point_label_x_to_limits,
                true,
                Some(prepared_stage),
            )?;
        }
        let text_topology =
            crate::layout::PreparedTextCensus::structural_address_count(&requirements)
                .map_err(|error| Error::Render(format!("prepared canvas text census: {error}")))?;
        let mut scalar_presence = prepared_stage.reserve_scalar_presence_map()?;
        let census = crate::layout::PreparedTextCensus::materialize(
            requirements,
            Some(prepared_stage),
            Some(&mut scalar_presence),
        )
        .map_err(|error| Error::Render(format!("prepared canvas text census: {error}")))?;
        if census.counts.structural_addresses != text_topology {
            return Err(Error::Render(format!(
                "prepared canvas text topology counted {text_topology} addresses but materialized {}",
                census.counts.structural_addresses
            )));
        }
        Ok((census, scalar_presence))
    }

    fn build_canvas_plan(
        &self,
        themes: Arc<ResolvedCanvasThemes>,
        assigned: crate::layout::RootSize,
        round: &mut crate::layout::kernel::FeedbackRound,
        children: &mut crate::decorations::plan::children::ChildPlans,
        shared_x: &[Option<(f64, f64)>],
        interiors: &[Option<crate::layout::interior::PreparedInteriorInputs>],
    ) -> Result<CanvasPlan, Error> {
        use crate::layout::fit::FitOutcome;
        use crate::layout::kernel::TrialOutcome;
        let prepared =
            self.prepare_canvas_round(themes, assigned, round, children, shared_x, interiors)?;
        let mut compact_changes = true;
        if self.legend_policy != CanvasLegendPolicy::Shared {
            return match self.build_canvas_attempt(
                &prepared,
                assigned,
                round,
                None,
                &mut compact_changes,
                children,
            )? {
                FitOutcome::Measured(plan) => Ok(plan),
                FitOutcome::Miss(reason) => Err(reason.into_error()),
            };
        }
        let owner = crate::layout::ComponentAddress::root()
            .child(crate::layout::AddressRole::CanvasSharedLegend, 0)?;
        let mut first_miss = None;
        for (edge_index, edge) in [
            crate::LegendPosition::OutsideRight,
            crate::LegendPosition::OutsideTop,
            crate::LegendPosition::OutsideBottom,
            crate::LegendPosition::OutsideLeft,
        ]
        .into_iter()
        .enumerate()
        {
            compact_changes = true;
            for compact in [false, true] {
                if compact && !compact_changes {
                    continue;
                }
                round.claim_guide_trial(
                    &owner,
                    8,
                    u32::try_from(edge_index * 2 + usize::from(compact))
                        .map_err(|_| Error::Render("Canvas guide ordinal overflowed".into()))?,
                )?;
                let checkpoint = round.begin_trial()?;
                #[cfg(test)]
                let guide_trial = prepared.shared_legend.as_ref().map(|_| {
                    crate::orchestrator::measurements::GuideTrial::new(&owner, edge, compact)
                });
                round.observe_count(edge as usize)?;
                round.observe_count(usize::from(compact))?;
                let outcome = self.build_canvas_attempt(
                    &prepared,
                    assigned,
                    round,
                    Some((edge, compact)),
                    &mut compact_changes,
                    children,
                )?;
                match outcome {
                    FitOutcome::Measured(plan) => match round.finish_trial(checkpoint) {
                        _outcome @ (TrialOutcome::Fit | TrialOutcome::Provisional) => {
                            #[cfg(test)]
                            #[expect(
                                clippy::used_underscore_binding,
                                reason = "outcome is observed only in tests"
                            )]
                            if let Some(trial) = &guide_trial {
                                trial.complete(&_outcome);
                            }
                            return Ok(plan);
                        }
                        TrialOutcome::Miss(reason) => {
                            #[cfg(test)]
                            if let Some(trial) = &guide_trial {
                                trial.reject();
                            }
                            FitFailure::retain_least(&mut first_miss, reason);
                        }
                    },
                    FitOutcome::Miss(reason) => {
                        #[cfg(test)]
                        if let Some(trial) = &guide_trial {
                            trial.reject();
                        }
                        round.record_failure(reason)?;
                        if let TrialOutcome::Miss(reason) = round.finish_trial(checkpoint) {
                            FitFailure::retain_least(&mut first_miss, reason);
                        }
                    }
                }
            }
        }
        Err(first_miss
            .unwrap_or_else(|| {
                FitScope::new(&owner, 0).failure(
                    FitKind::LegendOrGuide,
                    "shared guide cannot fit its containing Canvas frame",
                )
            })
            .into_error())
    }

    fn prepare_canvas_round<'a>(
        &self,
        themes: Arc<ResolvedCanvasThemes>,
        assigned: crate::layout::RootSize,
        round: &mut crate::layout::kernel::FeedbackRound,
        children: &mut crate::decorations::plan::children::ChildPlans<'_>,
        shared_x: &'a [Option<(f64, f64)>],
        interiors: &[Option<crate::layout::interior::PreparedInteriorInputs>],
    ) -> Result<PreparedCanvasRound<'a>, Error> {
        let (width, height) = assigned.dimensions();
        #[cfg(test)]
        record_canvas_theme(usize::MAX, "layout", themes.root());
        let canvas_layout = CanvasLayout::measured_composed_canvas(
            width,
            height,
            themes.root(),
            CanvasLayoutSpec {
                has_title: self.title.as_ref().is_some_and(|value| !value.is_empty()),
                has_subtitle: self
                    .subtitle
                    .as_ref()
                    .is_some_and(|value| !value.is_empty()),
                has_caption: self.caption.as_ref().is_some_and(|value| !value.is_empty()),
                has_x_label: false,
                has_y_label: false,
            },
            (
                self.title.as_deref(),
                self.subtitle.as_deref(),
                self.caption.as_deref(),
            ),
        )?;
        let full_outer = canvas_layout.panel_box;
        round.observe_canvas(&canvas_layout)?;
        let mut plots = self
            .panels
            .iter()
            .enumerate()
            .map(|(index, plot)| {
                plot.as_ref()
                    .map(|plot| {
                        let plot = self.labels.as_ref().map_or_else(
                            || plot.clone(),
                            |labels| plot.clone().labels(labels.clone()),
                        );
                        let plot = plot.with_render_scope(format!("panel-{index}"));
                        match interiors.get(index).and_then(Option::as_ref) {
                            Some(inputs) => inputs.attach(plot),
                            None => Ok(plot),
                        }
                    })
                    .transpose()
            })
            .collect::<Result<Vec<_>, Error>>()?;
        let hue_groups = if self.legend_policy == CanvasLegendPolicy::Shared {
            self.canvas_hue_groups(&plots, full_outer, &themes, children, round)?
        } else {
            Vec::new()
        };
        plots = plots
            .into_iter()
            .enumerate()
            .map(|(index, plot)| {
                plot.map(|plot| {
                    let plot = hue_groups
                        .iter()
                        .find(|(dimension, _)| Some(dimension.as_str()) == plot.hue_dimension_key())
                        .map_or_else(
                            || plot.clone(),
                            |(_, levels)| plot.clone().with_global_hue_levels(levels.clone()),
                        );
                    plot.with_canvas_size(width, height)
                        .with_parent_axis_gutters()
                        .with_render_scope(format!("panel-{index}"))
                })
            })
            .collect::<Vec<_>>();
        let (shared_legend, body_probes) = if self.legend_policy == CanvasLegendPolicy::Shared {
            Self::preflight_shared_legend(&plots, &themes, full_outer, children, round)?
        } else {
            (None, Vec::new())
        };
        Ok(PreparedCanvasRound {
            themes,
            canvas_layout,
            plots,
            shared_legend: shared_legend.map(|(title, entries)| (title, entries.into())),
            body_probes,
            shared_x,
        })
    }

    #[expect(
        clippy::too_many_lines,
        clippy::cast_precision_loss,
        reason = "one Canvas trial allocates declared tracks and measures every affected child"
    )]
    fn build_canvas_attempt(
        &self,
        prepared: &PreparedCanvasRound,
        assigned: crate::layout::RootSize,
        round: &mut crate::layout::kernel::FeedbackRound,
        guide_trial: Option<(crate::LegendPosition, bool)>,
        compact_changes: &mut bool,
        children: &mut crate::decorations::plan::children::ChildPlans,
    ) -> Result<crate::layout::fit::FitOutcome<CanvasPlan>, Error> {
        use crate::layout::fit::{self, FitOutcome};
        let root_address = crate::layout::ComponentAddress::root();
        let guide_owner = root_address.child(crate::layout::AddressRole::CanvasSharedLegend, 0)?;
        let scope = FitScope::new(&root_address, round.attempt_ordinal(&guide_owner)?);
        let guide_scope = FitScope::new(&guide_owner, scope.ordinal);
        let (width, height) = assigned.dimensions();
        let themes = Arc::clone(&prepared.themes);
        let canvas_layout = prepared.canvas_layout.clone();
        let full_outer = canvas_layout.panel_box;
        let mut plots = prepared.plots.clone();
        let mut outer = full_outer;
        let shared_legend = if let Some((title, entries)) = &prepared.shared_legend {
            let (edge, compact) = guide_trial.expect("shared guide has an owned trial");
            let horizontal = matches!(
                edge,
                crate::LegendPosition::OutsideTop | crate::LegendPosition::OutsideBottom
            );
            let metrics = crate::decorations::legend::LegendMetrics::from_canvas(
                f64::from(width),
                f64::from(height),
            );
            let gap = self.shared_legend_gap.unwrap_or(metrics.pad * 2.0);
            let minimum_width =
                self.columns as f64 * 80.0 + self.columns.saturating_sub(1) as f64 * 12.0;
            let minimum_height =
                self.rows as f64 * 60.0 + self.rows.saturating_sub(1) as f64 * 12.0;
            let available = (
                full_outer.width() - if horizontal { 0.0 } else { minimum_width + gap },
                full_outer.height()
                    - if horizontal {
                        minimum_height + gap
                    } else {
                        0.0
                    },
            );
            let legend = match Legend::measure_outside(
                title,
                Arc::clone(entries),
                (width, height),
                available,
                horizontal,
                compact,
                themes.root(),
                compact_changes,
                &guide_scope,
            )? {
                FitOutcome::Measured(legend) => legend,
                FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
            };
            let w = legend.compute_box_width();
            #[cfg(test)]
            crate::orchestrator::measurements::guide_body();
            let h = legend.compute_box_height();
            let (left, bottom) = match edge {
                crate::LegendPosition::OutsideRight => {
                    outer.x1 -= w + gap;
                    (
                        outer.x1 + gap,
                        f64::midpoint(full_outer.y0, full_outer.y1) - h / 2.0,
                    )
                }
                crate::LegendPosition::OutsideLeft => {
                    outer.x0 += w + gap;
                    (
                        full_outer.x0,
                        f64::midpoint(full_outer.y0, full_outer.y1) - h / 2.0,
                    )
                }
                crate::LegendPosition::OutsideTop => {
                    outer.y1 -= h + gap;
                    (
                        f64::midpoint(full_outer.x0, full_outer.x1) - w / 2.0,
                        outer.y1 + gap,
                    )
                }
                crate::LegendPosition::OutsideBottom => {
                    outer.y0 += h + gap;
                    (
                        f64::midpoint(full_outer.x0, full_outer.x1) - w / 2.0,
                        full_outer.y0,
                    )
                }
                _ => unreachable!("closed shared guide edge order"),
            };
            let bounds = legend.box_at(left, bottom);
            Some(crate::decorations::plan::ResolvedLegend { legend, bounds })
        } else {
            None
        };
        let header_theme = themes.root();
        let header_outer = outer;
        let column_header_height = if self.column_headers.is_empty() {
            0.0
        } else {
            f64::from(header_theme.composition_title_font_size) + 18.0
        };
        let row_header_width = self
            .row_headers
            .iter()
            .map(|header| {
                header_theme
                    .measure_legacy_text(
                        &header.text,
                        header_theme.composition_title_font_size,
                        PhysicalTextAlignment::Center,
                        "canvas row header",
                    )
                    .map(|plan| plan.width())
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .fold(0.0_f64, f64::max)
            + if self.row_headers.is_empty() {
                0.0
            } else {
                20.0
            };
        outer.y1 -= column_header_height;
        outer.x0 += row_header_width;
        if outer.width() < 80.0 || outer.height() < 60.0 {
            return Ok(FitOutcome::Miss(scope.failure(
                FitKind::TrackOrResidual,
                "canvas shared headers leave insufficient panel space; enlarge the canvas",
            )));
        }
        let provisional = match crate::layout::canvas::allocate_composed_grid_fit(
            outer,
            self.rows,
            self.columns,
            &[],
            "canvas layout",
            &scope.child(crate::layout::AddressRole::CanvasGrid, 0)?,
        )? {
            FitOutcome::Measured(cells) => cells,
            FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
        };
        if self.category_labels == CanvasCategoryLabels::FirstColumnPerRow {
            for row in 0..self.rows {
                let first_index = row * self.columns;
                let first = plots[first_index].as_ref().ok_or_else(|| {
                    Error::Data(format!(
                        "canvas row {row} requires a configured first-column panel for shared category labels"
                    ))
                })?;
                let expected_theme = themes
                    .plot(first_index)
                    .expect("configured canvas panel has a resolved theme");
                #[cfg(test)]
                record_canvas_theme(
                    first_index,
                    "shared-category-label-validation",
                    expected_theme,
                );
                let expected = plot_category_domain(
                    first,
                    provisional[first_index],
                    expected_theme,
                    children,
                    first_index,
                    round,
                )?;
                for column in 1..self.columns {
                    let index = first_index + column;
                    let plot = plots[index].as_ref().ok_or_else(|| {
                        Error::Data(format!(
                            "canvas row {row} requires panel ({row}, {column}) for shared category labels"
                        ))
                    })?;
                    let theme = themes
                        .plot(index)
                        .expect("configured canvas panel has a resolved theme");
                    #[cfg(test)]
                    record_canvas_theme(index, "shared-category-label-validation", theme);
                    if plot_category_domain(
                        plot,
                        provisional[index],
                        theme,
                        children,
                        index,
                        round,
                    )? != expected
                    {
                        return Err(Error::Data(format!(
                            "canvas row {row} category domains differ across panels"
                        )));
                    }
                    plots[index] = Some(plot.clone().with_y_tick_labels_visible(false));
                }
            }
        }
        if !self.x_range_groups.is_empty() {
            for group in &self.x_range_groups {
                let mut shared: Option<(f64, f64)> = None;
                for &index in group {
                    let base = prepared.shared_x[index].expect("prepared shared x member");
                    let address = crate::layout::ComponentAddress::root().child(
                        crate::layout::AddressRole::CanvasCell,
                        u32::try_from(index)
                            .map_err(|_| Error::Render("canvas address overflowed".into()))?,
                    )?;
                    let range = round
                        .domain(&address, FeedbackKind::AutomaticDomainX)
                        .map_or(base, |range| (base.0.min(range.0), base.1.max(range.1)));
                    shared = Some(shared.map_or(range, |current| {
                        (current.0.min(range.0), current.1.max(range.1))
                    }));
                }
                let shared = shared.expect("validated nonempty x-range group");
                if !shared.0.is_finite() || !shared.1.is_finite() || shared.0 >= shared.1 {
                    return Err(Error::Render(
                        "canvas x-range group did not produce a finite ascending range".into(),
                    ));
                }
                for &index in group {
                    let plot = plots[index].as_ref().expect("validated grouped panel");
                    plots[index] = Some(plot.clone().with_resolved_facet_ranges(
                        plot.kind(),
                        Some(shared),
                        None,
                    ));
                }
            }
        }
        for (index, plot) in plots.iter_mut().enumerate() {
            let Some(plot) = plot else { continue };
            let address = crate::layout::ComponentAddress::root().child(
                crate::layout::AddressRole::CanvasCell,
                u32::try_from(index)
                    .map_err(|_| Error::Render("canvas address overflowed".into()))?,
            )?;
            if !self
                .x_range_groups
                .iter()
                .any(|group| group.contains(&index))
            {
                if let Some(range) = round.domain(&address, FeedbackKind::AutomaticDomainX) {
                    *plot = plot.clone().with_planned_x_limits(range);
                }
            }
            if let Some(range) = round.domain(&address, FeedbackKind::AutomaticDomainY) {
                *plot = plot.clone().with_planned_y_limits(range);
            }
        }
        let grid_address = crate::layout::ComponentAddress::root()
            .child(crate::layout::AddressRole::CanvasGrid, 0)?;
        round.begin_owner(&grid_address)?;
        round.observe_box(outer)?;
        let mut requirements = Vec::new();
        for index in 0..self.panels.len() {
            let address = grid_address.child(
                crate::layout::AddressRole::CanvasCell,
                u32::try_from(index)
                    .map_err(|_| Error::Render("canvas grid address overflowed".into()))?,
            )?;
            let [left, right, bottom, top] = round.insets(&address, FeedbackKind::AxisGutter);
            let owned = round.insets(&address, FeedbackKind::OwnedInk);
            let top = top + owned[3];
            requirements.push((
                index / self.columns,
                index % self.columns,
                PanelRequirements {
                    left,
                    right,
                    bottom,
                    top,
                },
            ));
        }
        for (group_index, group) in self.x_range_groups.iter().enumerate() {
            let address = grid_address.child(
                crate::layout::AddressRole::CanvasRangeGroup,
                u32::try_from(group_index)
                    .map_err(|_| Error::Render("canvas group address overflowed".into()))?,
            )?;
            let alignment = round.insets(&address, FeedbackKind::SharedAlignment);
            for &index in group {
                requirements[index].2.left = requirements[index].2.left.max(alignment[0]);
                requirements[index].2.right = requirements[index].2.right.max(alignment[1]);
            }
        }
        let cells = match crate::layout::canvas::allocate_composed_grid_fit(
            outer,
            self.rows,
            self.columns,
            &requirements,
            "canvas layout",
            &scope.child(crate::layout::AddressRole::CanvasGrid, 0)?,
        )? {
            FitOutcome::Measured(cells) => cells,
            FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
        };
        let mut measured = Vec::new();
        let mut requirements = Vec::new();
        let mut child_miss = None;
        for (index, plot) in plots.iter().enumerate() {
            let Some(plot) = plot else { continue };
            let row = index / self.columns;
            let column = index % self.columns;
            let kind = plot.kind();
            let theme = themes
                .plot(index)
                .expect("configured canvas panel has a resolved theme");
            #[cfg(test)]
            record_canvas_theme(index, "panel-measure", theme);
            if shared_legend.is_some() {
                let base = prepared.body_probes[index]
                    .as_ref()
                    .expect("shared guide body probe");
                if let FitOutcome::Miss(reason) = crate::decorations::plan::check_assigned_body(
                    plot,
                    base,
                    full_outer,
                    cells[index],
                    theme,
                    &scope.child(crate::layout::AddressRole::CanvasCell, index)?,
                )? {
                    FitFailure::retain_least(&mut child_miss, reason);
                }
            }
            let child_scope = scope.child(crate::layout::AddressRole::CanvasCell, index)?;
            let owner = crate::layout::ComponentAddress::root().child(
                crate::layout::AddressRole::CanvasCell,
                u32::try_from(index)
                    .map_err(|_| Error::Render("canvas address overflowed".into()))?,
            )?;
            let context = crate::spec::PlotRenderContext::new(plot.data(), cells[index], theme)
                .with_fit_scope(&child_scope);
            let result = match children.measure(
                plot,
                context,
                &owner,
                MeasurementPhase::Layout,
                round,
                || plot.preflight_measurement(kind, cells[index], theme, &child_scope),
            )? {
                FitOutcome::Measured(result) => result,
                FitOutcome::Miss(failure) => {
                    FitFailure::retain_least(&mut child_miss, failure);
                    continue;
                }
            };
            #[cfg(test)]
            CANVAS_ASSIGNED_PANEL_EVIDENCE.with(|evidence| {
                evidence
                    .borrow_mut()
                    .push((cells[index], result.layout.projection.panel_box()));
            });
            if result.decoration_mode == DecorationMode::SelfManaged {
                return Err(Error::Render(format!(
                    "{kind:?} manages its own canvas and cannot be nested in a composed canvas"
                )));
            }
            let title = plot.title_text().filter(|title| !title.is_empty());
            let subtitle = plot.subtitle_text().filter(|subtitle| !subtitle.is_empty());
            let requirement = match fit::outcome(PanelRequirements::measure_header(
                result.axes.as_ref().ok_or_else(|| {
                    Error::Render("composed panel is missing its measured axes".into())
                })?,
                theme,
                title,
                subtitle,
                cells[index],
                &scope.child(crate::layout::AddressRole::CanvasCell, index)?,
            ))? {
                FitOutcome::Measured(requirement) => requirement,
                FitOutcome::Miss(failure) => {
                    FitFailure::retain_least(&mut child_miss, failure);
                    continue;
                }
            };
            round.observe_layout(&result.layout)?;
            measured.push((index, kind));
            requirements.push((row, column, requirement));
        }
        if let Some(failure) = child_miss {
            return Ok(FitOutcome::Miss(failure));
        }
        for (group_index, group) in self.x_range_groups.iter().enumerate() {
            let address = grid_address.child(
                crate::layout::AddressRole::CanvasRangeGroup,
                u32::try_from(group_index)
                    .map_err(|_| Error::Render("canvas group address overflowed".into()))?,
            )?;
            let mut alignment = [0.0_f64; 4];
            for &index in group {
                if let Some((_, _, requirement)) = requirements
                    .iter()
                    .find(|(row, column, _)| row * self.columns + column == index)
                {
                    alignment[0] = alignment[0].max(requirement.left);
                    alignment[1] = alignment[1].max(requirement.right);
                }
            }
            if let Some(request) = round.request_insets(
                &address,
                &address,
                FeedbackKind::SharedAlignment,
                alignment,
            )? {
                round.miss(
                    &address,
                    FitKind::TrackOrResidual,
                    &[Some(request)],
                    "shared range group requires aligned panel edges",
                )?;
            }
        }
        let mut row_title_requirements = Vec::new();
        for (title_index, title) in self.row_titles.iter().enumerate() {
            let first = title.start * self.columns;
            let last = first + self.columns - 1;
            let appearance = header_theme.legacy_text_appearance(
                header_theme.composition_title_font_size,
                crate::TextWeight::Normal,
            );
            if let FitOutcome::Miss(reason) = fit::text(
                header_theme,
                &title.text,
                &appearance,
                (cells[last].x1 - cells[first].x0 - 16.0).max(0.0),
                PhysicalTextAlignment::Center,
                "Canvas row title",
                &scope.child(crate::layout::AddressRole::CanvasRowTitle, title_index)?,
            )? {
                return Ok(FitOutcome::Miss(reason));
            }
        }
        self.add_row_title_requirements(&mut row_title_requirements, &cells, header_theme)?;
        for (row, column, requirement) in row_title_requirements {
            let index = row * self.columns + column;
            let base_top = requirements
                .iter()
                .find(|(r, c, _)| *r == row && *c == column)
                .map_or(0.0, |(_, _, requirement)| requirement.top);
            let address = grid_address.child(
                crate::layout::AddressRole::CanvasCell,
                u32::try_from(index)
                    .map_err(|_| Error::Render("canvas grid address overflowed".into()))?,
            )?;
            if let Some(request) = round.request_insets(
                &address,
                &address,
                FeedbackKind::OwnedInk,
                [0.0, 0.0, 0.0, (requirement.top - base_top).max(0.0)],
            )? {
                round.miss(
                    &address,
                    FitKind::TrackOrResidual,
                    &[Some(request)],
                    "row title ink requires an owned grid reserve",
                )?;
            }
        }
        for (row, column, requirement) in requirements {
            let index = row * self.columns + column;
            let address = grid_address.child(
                crate::layout::AddressRole::CanvasCell,
                u32::try_from(index)
                    .map_err(|_| Error::Render("canvas grid address overflowed".into()))?,
            )?;
            round.observe_box(cells[index])?;
            if let Some(request) = round.request_insets(
                &address,
                &address,
                crate::layout::kernel::FeedbackKind::AxisGutter,
                [
                    requirement.left,
                    requirement.right,
                    requirement.bottom,
                    requirement.top,
                ],
            )? {
                round.miss(
                    &address,
                    crate::layout::kernel::FitKind::Tick,
                    &[Some(request)],
                    "canvas cell requires measured decoration gutters",
                )?;
            }
        }
        let mut headers = Vec::new();
        crate::render::backend::reserve_text_records(
            &mut headers,
            self.column_headers.len() + self.row_headers.len(),
            header_theme.prepared_font_environment()?,
        )?;
        for (header_index, header) in self.column_headers.iter().enumerate() {
            let left = cells[header.start].x0;
            let right = cells[header.start + header.span - 1].x1;
            let appearance = header_theme.legacy_text_appearance(
                header_theme.composition_title_font_size,
                crate::TextWeight::Normal,
            );
            let plan = match fit::text(
                header_theme,
                &header.text,
                &appearance,
                right - left,
                PhysicalTextAlignment::Center,
                "Canvas column header",
                &scope.child(crate::layout::AddressRole::CanvasHeader, header_index)?,
            )? {
                FitOutcome::Measured(plan)
                    if plan.lines.len() == 1 && plan.height() <= column_header_height =>
                {
                    plan
                }
                FitOutcome::Measured(_) => {
                    return Ok(FitOutcome::Miss(
                        scope
                            .child(crate::layout::AddressRole::CanvasHeader, header_index)?
                            .failure(
                                FitKind::Text,
                                "Canvas column header exceeds its assigned header band",
                            ),
                    ))
                }
                FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
            };
            headers.push(CanvasHeaderPlan {
                text: crate::render::backend::PlacedText::from_deferred_first_line(
                    plan,
                    Point2D::new(
                        f64::midpoint(left, right).round(),
                        (header_outer.y1 - column_header_height / 2.0).round(),
                    ),
                    crate::render::backend::TextAnchor::Middle,
                    0.0,
                ),
            });
        }
        for header in &self.row_headers {
            let indices = (header.start..header.start + header.span)
                .map(|row| row * self.columns)
                .collect::<Vec<_>>();
            let bottom = indices
                .iter()
                .map(|index| cells[*index].y0)
                .fold(f64::INFINITY, f64::min);
            let top = indices
                .iter()
                .map(|index| cells[*index].y1)
                .fold(f64::NEG_INFINITY, f64::max);
            headers.push(CanvasHeaderPlan {
                text: crate::render::backend::PlacedText::middle(
                    header_theme.measure_legacy_text(
                        &header.text,
                        header_theme.composition_title_font_size,
                        PhysicalTextAlignment::Center,
                        "canvas row header",
                    )?,
                    Point2D::new(
                        (header_outer.x0 + row_header_width / 2.0).round(),
                        f64::midpoint(bottom, top).round(),
                    ),
                    crate::render::backend::TextAnchor::Middle,
                    0.0,
                ),
            });
        }
        let row_titles = self.plan_row_titles(&cells, header_theme)?;
        #[cfg(test)]
        {
            if !headers.is_empty() {
                record_canvas_theme(usize::MAX, "shared-header-plan", header_theme);
            }
            if !row_titles.is_empty() {
                record_canvas_theme(usize::MAX, "row-title-plan", header_theme);
            }
        }

        let mut text_panels = Vec::new();
        crate::render::backend::reserve_text_records(
            &mut text_panels,
            self.text_panels.iter().flatten().count(),
            themes.root().prepared_font_environment()?,
        )?;
        for (index, text_panel) in self.text_panels.iter().enumerate() {
            let Some(text_panel) = text_panel else {
                continue;
            };
            #[cfg(test)]
            record_canvas_theme(usize::MAX, "text-plan", themes.root());
            let cell = cells[index];
            let maximum_width = (cell.width() - 32.0).max(1.0);
            let title = match fit::text(
                themes.root(),
                text_panel.title(),
                &themes.root().legacy_text_appearance(
                    themes.root().composition_title_font_size,
                    crate::TextWeight::Normal,
                ),
                maximum_width,
                PhysicalTextAlignment::Center,
                "text panel title",
                &scope
                    .child(crate::layout::AddressRole::TextPanelContent, index)?
                    .child(crate::layout::AddressRole::TextPanelTitle, 0)?,
            )? {
                FitOutcome::Measured(plan) => TextBlock::from_resolved(plan),
                FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
            };
            let body = match fit::text(
                themes.root(),
                text_panel.body(),
                &themes.root().legacy_text_appearance(
                    themes.root().label_font_size,
                    crate::TextWeight::Normal,
                ),
                maximum_width,
                PhysicalTextAlignment::Center,
                "text panel body",
                &scope
                    .child(crate::layout::AddressRole::TextPanelContent, index)?
                    .child(crate::layout::AddressRole::TextPanelBody, 0)?,
            )? {
                FitOutcome::Measured(plan) => TextBlock::from_resolved(plan),
                FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
            };
            let required_height = title.height() + 12.0 + body.height();
            if required_height > cell.height() - 32.0 {
                return Ok(FitOutcome::Miss(scope.child(crate::layout::AddressRole::TextPanelContent, index)?.failure(FitKind::Text, format!(
                    "text panel at ({}, {}) requires {required_height:.1}px but only {:.1}px is available; increase canvas size or shorten text",
                    index / self.columns,
                    index % self.columns,
                    (cell.height() - 32.0).max(0.0)
                ))));
            }
            text_panels.push(TextPanelPlan {
                index,
                cell,
                title_placement: crate::render::backend::PlacedText::from_deferred_first_line(
                    title.plan().clone(),
                    Point2D::new(
                        cell.x0.midpoint(cell.x1).round(),
                        (cell.y0.midpoint(cell.y1) + required_height / 2.0
                            - title.line_height() / 2.0)
                            .round(),
                    ),
                    crate::render::backend::TextAnchor::Middle,
                    0.0,
                )
                .placement,
                body_placement: crate::render::backend::PlacedText::from_deferred_first_line(
                    body.plan().clone(),
                    Point2D::new(
                        cell.x0.midpoint(cell.x1).round(),
                        (cell.y0.midpoint(cell.y1) + required_height / 2.0
                            - title.line_height() / 2.0
                            - title.height()
                            - 12.0)
                            .round(),
                    ),
                    crate::render::backend::TextAnchor::Middle,
                    0.0,
                )
                .placement,
                title,
                body,
            });
        }

        for text_panel in &text_panels {
            round.observe_box(text_panel.cell)?;
            round.observe_text_block(&text_panel.title)?;
            round.observe_text_block(&text_panel.body)?;
        }
        for header in headers.iter().chain(&row_titles) {
            round.observe_text(&header.text.plan)?;
            round.observe_number(header.text.placement.position.x)?;
            round.observe_number(header.text.placement.position.y)?;
        }
        round.observe_count(usize::from(shared_legend.is_some()))?;
        if let Some(legend) = &shared_legend {
            legend.observe_feedback(round)?;
        }

        let mut panels = Vec::with_capacity(measured.len());
        let mut child_miss = None;
        for (index, kind) in measured {
            let plot = plots[index]
                .as_ref()
                .expect("measured canvas panel remains configured");
            let cell = cells[index];
            let role = if self.legend_policy == CanvasLegendPolicy::PerPlot {
                crate::decorations::plan::LegendRole::Panel
            } else {
                crate::decorations::plan::LegendRole::Suppressed
            };
            let theme = themes
                .plot(index)
                .expect("configured canvas panel has a resolved theme");
            #[cfg(test)]
            record_canvas_theme(index, "panel-plan", theme);
            let address = crate::layout::ComponentAddress::root().child(
                crate::layout::AddressRole::CanvasCell,
                u32::try_from(index)
                    .map_err(|_| Error::Render("canvas address overflowed".into()))?,
            )?;
            let source_rows = plot.configured_source_rows()?;
            let context = source_rows.map_or_else(
                || crate::spec::PlotRenderContext::new(plot.data(), cell, theme),
                |rows| {
                    crate::spec::PlotRenderContext::with_source_rows(plot.data(), rows, cell, theme)
                },
            );
            let panel_plan = match children.plan(plot, role, context, None, round, &address)? {
                FitOutcome::Measured(plan) => plan,
                FitOutcome::Miss(failure) => {
                    FitFailure::retain_least(&mut child_miss, failure);
                    continue;
                }
            };
            if panel_plan.result.decoration_mode == DecorationMode::SelfManaged {
                return Err(Error::Render(format!(
                    "{kind:?} manages its own canvas and cannot be nested in a composed canvas"
                )));
            }
            panels.push(PlannedCanvasPanel {
                index,
                kind,
                canvas_cell: cell,
                panel_plan,
                header: plan_panel_header(plot, cell, theme)?,
            });
        }
        if let Some(failure) = child_miss {
            return Ok(FitOutcome::Miss(failure));
        }
        let plan = CanvasPlan {
            themes,
            plots,
            canvas_layout,
            shared_legend,
            panels,
            text_panels,
            headers,
            row_titles,
        };
        self.observe_arranged(&plan, round)?;
        Ok(FitOutcome::Measured(plan))
    }

    fn observe_arranged(
        &self,
        plan: &CanvasPlan,
        round: &mut crate::layout::kernel::FeedbackRound,
    ) -> Result<(), Error> {
        let mut geometry = crate::render::geometry::GeometryBackend::new(plan.themes.root())?;
        geometry.begin(self.width, self.height)?;
        let result = (|| {
            for panel in &plan.text_panels {
                draw_text_panel(panel, &mut geometry)?;
            }
            draw_canvas_row_titles(&plan.row_titles, &mut geometry)?;
            draw_canvas_headers(&plan.headers, &mut geometry)?;
            Ok(())
        })();
        crate::render::backend::finish_scope(result, geometry.end(), "arranged Canvas chrome")?;
        geometry.observe_feedback(round)?;
        for panel in &plan.panels {
            round.observe_box(panel.canvas_cell)?;
            let theme = plan.themes.plot(panel.index).expect("arranged panel theme");
            let plot = plan.plots[panel.index]
                .as_ref()
                .expect("arranged panel source");
            let mut geometry = crate::render::geometry::GeometryBackend::new(theme)?;
            geometry.begin(self.width, self.height)?;
            let result = draw_panel_header(plot, &panel.header, &mut geometry);
            crate::render::backend::finish_scope(
                result,
                geometry.end(),
                "arranged Canvas panel header",
            )?;
            geometry.observe_feedback(round)?;
        }
        Ok(())
    }

    #[expect(
        clippy::too_many_lines,
        reason = "the immutable Canvas plan keeps theme ownership and final draw ordering explicit"
    )]
    fn render_planned_canvas(
        &self,
        plan: &CanvasPlan,
        width: u32,
        height: u32,
        backend: &mut dyn Backend,
    ) -> Result<(), Error> {
        let theme = plan.themes.root();
        #[cfg(test)]
        record_canvas_theme(usize::MAX, "render", theme);
        backend.draw_surface(
            PanelBox::new(0.0, f64::from(width), 0.0, f64::from(height)),
            theme.surfaces.canvas_root,
            SurfaceRole::CanvasRoot,
        )?;
        for panel in &plan.panels {
            let theme = plan
                .themes
                .plot(panel.index)
                .expect("configured canvas panel has a resolved theme");
            backend.draw_surface(
                panel.panel_plan.panel_box,
                theme.surfaces.plot_panel,
                SurfaceRole::PlotPanel,
            )?;
        }
        for panel in &plan.panels {
            let plot = plan.plots[panel.index].as_ref().ok_or_else(|| {
                Error::Render("composed canvas panel disappeared during preflight".into())
            })?;
            #[cfg(test)]
            let theme = plan
                .themes
                .plot(panel.index)
                .expect("configured canvas panel has a resolved theme");
            #[cfg(test)]
            record_canvas_theme(panel.index, "grid-render", theme);
            crate::decorations::apply::apply_grid(plot, &panel.panel_plan, backend)?;
        }
        for panel in &plan.panels {
            let plot = plan.plots[panel.index].as_ref().ok_or_else(|| {
                Error::Render("composed canvas panel disappeared during render".into())
            })?;
            let theme = plan
                .themes
                .plot(panel.index)
                .expect("configured canvas panel has a resolved theme");
            #[cfg(test)]
            record_canvas_theme(panel.index, "panel-render", theme);
            let source_rows = plot.configured_source_rows()?;
            let plot_with_planned_limits = panel.panel_plan.plot_with_planned_limits(plot);
            let context = source_rows
                .map_or_else(
                    || {
                        crate::spec::PlotRenderContext::new(
                            plot_with_planned_limits.data(),
                            panel.panel_plan.panel_box,
                            theme,
                        )
                    },
                    |rows| {
                        crate::spec::PlotRenderContext::with_source_rows(
                            plot_with_planned_limits.data(),
                            rows,
                            panel.panel_plan.panel_box,
                            theme,
                        )
                    },
                )
                .with_retained_body(panel.panel_plan.result.retained_payload());
            plot_with_planned_limits.render_panel(panel.kind, context, backend)?;
            crate::significance::render_selected(plot, &panel.panel_plan.result, theme, backend)?;
            crate::decorations::apply::apply_axes(plot, &panel.panel_plan, backend)?;
            crate::decorations::apply::apply_planned_legend(
                plot,
                panel.panel_plan.legend.as_ref(),
                backend,
            )?;
            draw_panel_header(plot, &panel.header, backend)?;
        }
        for panel in &plan.text_panels {
            #[cfg(test)]
            record_canvas_theme(usize::MAX, "text-render", theme);
            draw_text_panel(panel, backend)?;
        }
        if !plan.row_titles.is_empty() {
            #[cfg(test)]
            record_canvas_theme(usize::MAX, "row-title-render", theme);
            draw_canvas_row_titles(&plan.row_titles, backend)?;
        }
        if !plan.headers.is_empty() {
            #[cfg(test)]
            record_canvas_theme(usize::MAX, "shared-header-render", theme);
            draw_canvas_headers(&plan.headers, backend)?;
        }
        if let Some(legend) = &plan.shared_legend {
            let (x, y) = legend.bounds.anchor();
            with_layer(backend, "canvas-shared-legend", |backend| {
                legend.legend.draw(backend, x, y)
            })?;
        }
        let mut titles = TitleBlock::new();
        if let Some(title) = &self.title {
            titles = titles.title(title.clone());
        }
        if let Some(subtitle) = &self.subtitle {
            titles = titles.subtitle(subtitle.clone());
        }
        if let Some(caption) = &self.caption {
            titles = titles.caption(caption.clone());
        }
        titles.draw(backend, width, &plan.canvas_layout)
    }

    fn canvas_hue_groups(
        &self,
        plots: &[Option<Plot>],
        panel: PanelBox,
        themes: &ResolvedCanvasThemes,
        children: &mut crate::decorations::plan::children::ChildPlans<'_>,
        round: &crate::layout::kernel::FeedbackRound,
    ) -> Result<Vec<(String, Vec<String>)>, Error> {
        let mut groups: Vec<(String, Vec<String>)> = Vec::new();
        for (index, plot) in plots.iter().enumerate() {
            let Some(plot) = plot else { continue };
            let plot = self.labels.as_ref().map_or_else(
                || plot.clone(),
                |labels| plot.clone().labels(labels.clone()),
            );
            if plot.legend_policy_spec() == crate::LegendPolicy::Hidden {
                continue;
            }
            let Some(dimension) = plot.hue_dimension_key() else {
                continue;
            };
            let theme = themes
                .plot(index)
                .expect("configured canvas panel has a resolved theme");
            #[cfg(test)]
            record_canvas_theme(index, "shared-hue-discovery", theme);
            let observed =
                children.canvas_probe(&plot, panel, theme, index, MeasurementPhase::Hue, round)?;
            let levels =
                if let Some((_, levels)) = groups.iter_mut().find(|(name, _)| name == dimension) {
                    levels
                } else {
                    let index = groups.len();
                    groups.push((dimension.to_owned(), Vec::new()));
                    &mut groups[index].1
                };
            for entry in &observed.legend_entries {
                if let Some(level) = entry.coordinated_hue_value_for(dimension) {
                    if !levels.iter().any(|value| value == level) {
                        levels.push(level.to_owned());
                    }
                }
            }
        }
        Ok(groups)
    }

    #[expect(
        clippy::type_complexity,
        reason = "shared descriptor preparation also retains the exact full-body probes used by frame attempts"
    )]
    fn preflight_shared_legend(
        plots: &[Option<Plot>],
        themes: &ResolvedCanvasThemes,
        panel: PanelBox,
        children: &mut crate::decorations::plan::children::ChildPlans<'_>,
        round: &crate::layout::kernel::FeedbackRound,
    ) -> Result<
        (
            Option<(String, Vec<LegendEntry>)>,
            Vec<Option<Arc<crate::spec::PlotRenderResult>>>,
        ),
        Error,
    > {
        let mut entries = Vec::new();
        let mut probes = Vec::new();
        crate::layout::fit::reserve(
            themes.root(),
            &mut probes,
            plots.len(),
            "shared guide body probes",
        )?;
        probes.resize_with(plots.len(), || None);
        for (index, plot) in plots.iter().enumerate() {
            let Some(plot) = plot else { continue };
            let selection = themes
                .plot_selection(index)
                .expect("configured canvas panel has a resolved theme");
            let theme = match selection {
                ResolvedPlotTheme::InheritedRoot => themes.root(),
                ResolvedPlotTheme::ExplicitPlot(theme) => theme.as_ref(),
            };
            #[cfg(test)]
            {
                if plot.legend_policy_spec() != crate::LegendPolicy::Hidden {
                    record_canvas_theme(index, "shared-legend-preflight", theme);
                    record_shared_legend_theme(index, selection);
                }
            }
            let result = children.canvas_probe(
                plot,
                panel,
                theme,
                index,
                MeasurementPhase::Legend,
                round,
            )?;
            if plot.legend_policy_spec() != crate::LegendPolicy::Hidden {
                entries.extend(result.legend_entries.iter().cloned());
            }
            probes[index] = Some(result);
        }
        let entries = crate::decorations::legend::merge_exact_entries(
            entries,
            "shared canvas legend",
            themes.root(),
        )?;
        if entries.is_empty() {
            Ok((None, probes))
        } else {
            Ok((
                Some((
                    crate::decorations::legend::resolved_title(&entries, themes.root())?,
                    entries,
                )),
                probes,
            ))
        }
    }
}

fn plot_category_domain(
    plot: &Plot,
    panel: PanelBox,
    theme: &crate::theme::ResolvedTheme,
    children: &mut crate::decorations::plan::children::ChildPlans<'_>,
    index: usize,
    round: &crate::layout::kernel::FeedbackRound,
) -> Result<Vec<String>, Error> {
    let kind = plot.kind();
    if !matches!(
        kind,
        PlotKind::HorizontalBar | PlotKind::StackedHorizontalBar
    ) {
        return Err(Error::Data(
            "shared category labels are supported only by horizontal bar families".into(),
        ));
    }
    let result =
        children.canvas_probe(plot, panel, theme, index, MeasurementPhase::Category, round)?;
    if result.y_domain != AxisDomain::Categorical {
        return Err(Error::Data(
            "shared category labels require a categorical y-axis".into(),
        ));
    }
    Ok(result.y_tick_keys.clone().unwrap_or_else(|| {
        result
            .layout
            .y_axis
            .ticks
            .iter()
            .map(|tick| tick.label.clone())
            .collect()
    }))
}

fn draw_canvas_headers(
    headers: &[CanvasHeaderPlan],
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    if headers.is_empty() {
        return Ok(());
    }
    with_layer(backend, "canvas-shared-headers", |backend| {
        for header in headers {
            header.text.draw(backend)?;
        }
        Ok(())
    })
}

fn draw_canvas_row_titles(
    titles: &[CanvasHeaderPlan],
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    if titles.is_empty() {
        return Ok(());
    }
    with_layer(backend, "canvas-row-titles", |backend| {
        for title in titles {
            title.text.draw(backend)?;
        }
        Ok(())
    })
}

fn draw_text_panel(panel: &TextPanelPlan, backend: &mut dyn Backend) -> Result<(), Error> {
    with_layer(
        backend,
        &format!("canvas-text-panel-{}", panel.index),
        |backend| {
            backend.draw_planned_text(panel.title_placement, panel.title.plan())?;
            backend.draw_planned_text(panel.body_placement, panel.body.plan())
        },
    )
}

#[expect(
    clippy::cast_precision_loss,
    reason = "small text-line indexes are projected into backend coordinates"
)]
fn plan_panel_header(
    plot: &Plot,
    cell: PanelBox,
    theme: &crate::theme::ResolvedTheme,
) -> Result<Vec<crate::render::backend::PlacedText>, Error> {
    let title = plot.title_text().filter(|title| !title.is_empty());
    let subtitle = plot.subtitle_text().filter(|subtitle| !subtitle.is_empty());
    if title.is_none() && subtitle.is_none() {
        return Ok(Vec::new());
    }
    let maximum_width = (cell.width() - 16.0).max(1.0);
    let title_plan = title
        .map(|text| {
            theme.plan_text(
                text,
                &theme.typography.title,
                maximum_width,
                PhysicalTextAlignment::Start,
                "panel title",
                false,
            )
        })
        .transpose()?;
    let subtitle_plan = subtitle
        .map(|text| {
            theme.plan_text(
                text,
                &theme.typography.subtitle,
                maximum_width,
                PhysicalTextAlignment::Start,
                "panel subtitle",
                false,
            )
        })
        .transpose()?;
    let subtitle_line_height = theme.typography.subtitle.line_height();
    let title_line_height = theme.typography.title.line_height();
    let title_base = cell.y1
        + 14.0
        + subtitle_plan.as_ref().map_or(0, |plan| plan.lines.len()) as f64 * subtitle_line_height
        + if title_plan.is_some() && subtitle_plan.is_some() {
            PANEL_HEADER_GAP
        } else {
            0.0
        };
    let mut plans = Vec::new();
    crate::render::backend::reserve_text_records(
        &mut plans,
        usize::from(title_plan.is_some()) + usize::from(subtitle_plan.is_some()),
        theme.prepared_font_environment()?,
    )?;
    let style = theme
        .typography
        .title
        .backend_style(crate::render::backend::TextAnchor::Start, 0.0);
    for (plan, base, pitch) in [
        (subtitle_plan, cell.y1 + 14.0, subtitle_line_height),
        (title_plan, title_base, title_line_height),
    ] {
        if let Some(plan) = plan {
            let first = base + plan.lines.len().saturating_sub(1) as f64 * pitch;
            plans.push(
                crate::render::backend::PlacedText::from_deferred_first_line(
                    plan,
                    Point2D::new((cell.x0 + 8.0).round(), first.round()),
                    style.anchor,
                    style.rotation,
                ),
            );
        }
    }
    Ok(plans)
}

fn draw_panel_header(
    plot: &Plot,
    plans: &[crate::render::backend::PlacedText],
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    if plans.is_empty() {
        return Ok(());
    }
    with_layer(backend, &plot.layer_name("title"), |backend| {
        for text in plans {
            text.draw(backend)?;
        }
        Ok(())
    })
}
