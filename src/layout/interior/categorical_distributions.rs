use super::{InteriorRoute, OneCategoricalAxis};
use crate::font::canonical::OperationContext;
use crate::layout::{AxisSide, PanelBox, TickSource};
use crate::orchestrator::measurements::{self, Counter};
use crate::plots::categorical_distribution::{extraction, retained, PreparedDistribution, STAGES};
use crate::spec::{AxisDomain, PlotRenderContext, PlotRenderResult};
use crate::{CategoryValues, NumericValues, Plot};

pub(crate) fn sources() -> [Plot; 3] {
    configured(false, false, None)
}

fn configured(annotated: bool, grouped: bool, limits: Option<(f64, f64)>) -> [Plot; 3] {
    fn finish<
        K: crate::kind::ExactFamily
            + crate::kind::Series
            + crate::kind::NumericY
            + crate::kind::Significance<crate::Pairwise>,
    >(
        mut plot: Plot<K>,
        annotated: bool,
        grouped: bool,
        limits: Option<(f64, f64)>,
    ) -> Plot {
        if grouped {
            plot = plot.series(
                CategoryValues::new(
                    "series",
                    [
                        "first", "first", "first", "first", "first", "first", "second", "second",
                    ],
                )
                .unwrap(),
            );
        }
        if let Some((lo, hi)) = limits {
            plot = plot.y_limits(lo, hi);
        }
        plot.significance(
            [crate::Significance::between(
                "after_visit",
                "before_visit",
                crate::SignificanceMark::PValue(0.0123),
            )]
            .into_iter()
            .filter(|_| annotated),
        )
        .erase()
    }
    let categories = CategoryValues::new(
        "category",
        [
            "after_visit",
            "after_visit",
            "after_visit",
            "after_visit",
            "before_visit",
            "before_visit",
            "after_visit",
            "after_visit",
        ],
    )
    .unwrap();
    let values = NumericValues::new("value", [1.25, 1.25, 1.25, 9.75, -2.5, 3.125, 2.25, 4.5]);
    [
        finish(
            Plot::boxplot(categories.clone(), values.clone()),
            annotated,
            grouped,
            limits,
        ),
        finish(
            Plot::violin(categories.clone(), values.clone()),
            annotated,
            grouped,
            limits,
        ),
        finish(
            Plot::raincloud(crate::RaincloudInput::samples(categories, values)),
            annotated,
            grouped,
            limits,
        ),
    ]
}

fn draw(result: &PlotRenderResult, plot: &Plot, theme: &crate::theme::ResolvedTheme) -> Vec<u8> {
    use crate::render::backend::Backend;
    let mut svg = crate::render::svg_backend::SvgBackend::in_memory(900, 700);
    svg.begin(900, 700).unwrap();
    result.marks.as_ref().unwrap().draw(&mut svg).unwrap();
    crate::significance::render_selected(plot, result, theme, &mut svg).unwrap();
    result.annotations.as_ref().unwrap().draw(&mut svg).unwrap();
    result.axes.as_ref().unwrap().draw(&mut svg).unwrap();
    svg.end().unwrap();
    svg.into_bytes().unwrap()
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one native contract covers prepared semantics and repeated assignments for each theme owner"
)]
fn native_categorical_distributions_keep_bottom_identity_sparse_groups_and_support() {
    for (source, family) in configured(false, true, None).into_iter().zip([
        OneCategoricalAxis::Boxplot,
        OneCategoricalAxis::Violin,
        OneCategoricalAxis::Raincloud,
    ]) {
        let source =
            source.labels(crate::Labels::humanized().alias("after_visit", "Follow-up visit"));
        let inputs =
            crate::plots::shared_x::prepare_plot(&source, &OperationContext::new(), None).unwrap();
        assert_eq!(inputs.route, InteriorRoute::OneCategoricalAxis(family));
        let plot = inputs.attach(source).unwrap();
        let prepared = retained(&plot).unwrap();
        let sample = match prepared {
            PreparedDistribution::Boxplot(input) => &input.sample,
            PreparedDistribution::Violin(input) => &input.sample,
            PreparedDistribution::Supplied(_) => unreachable!(),
        };
        assert_eq!(sample.x_levels, ["after_visit", "before_visit"]);
        assert_eq!(sample.group_levels, ["first", "second"]);
        assert_eq!(
            sample
                .groups
                .iter()
                .map(|group| (group.x_index, group.group_index, group.observations.len()))
                .collect::<Vec<_>>(),
            [(0, 0, 4), (1, 0, 2), (0, 1, 2)]
        );
        if family == OneCategoricalAxis::Boxplot {
            assert_eq!(prepared.range(), (-2.5, 9.75));
        } else {
            assert!(prepared.range().0 < -2.5 && prepared.range().1 > 9.75);
        }
        for preset in [
            crate::Theme::Publication,
            crate::Theme::Nature,
            crate::Theme::Science,
        ] {
            let theme = preset.resolve().unwrap();
            let plot = crate::plots::shared_x::prepare_plot(&plot, &OperationContext::new(), None)
                .unwrap()
                .attach(plot.clone())
                .unwrap();
            let reads = crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get);
            for panel in [
                PanelBox::new(80.25, 550.75, 70.5, 410.25),
                PanelBox::new(30.0, 810.0, 50.0, 600.0),
            ] {
                for range in [(-20.0, 30.0), (-1.0, 6.0)] {
                    let plot = plot
                        .clone()
                        .with_planned_x_limits((-10.0, 10.0))
                        .with_planned_y_limits(range);
                    measurements::start();
                    let selected = plot
                        .measure_selected_interior(PlotRenderContext::new(
                            plot.data(),
                            panel,
                            &theme,
                        ))
                        .unwrap();
                    let work = measurements::finish();
                    assert_eq!(
                        work.counters[Counter::OneCategoricalAxisMeasure as usize],
                        1
                    );
                    assert_eq!(work.counters[Counter::LegacyPanelConstructor as usize], 0);
                    assert_eq!(
                        (selected.x_domain, selected.y_domain),
                        (AxisDomain::Categorical, AxisDomain::Continuous)
                    );
                    assert_eq!(
                        selected.x_tick_keys.as_ref().unwrap(),
                        &["after_visit", "before_visit"]
                    );
                    assert_eq!(selected.layout.x_axis.source, TickSource::SUPPLIED);
                    assert_eq!(
                        selected
                            .layout
                            .x_axis
                            .ticks
                            .iter()
                            .map(|tick| (tick.position, tick.label.as_str()))
                            .collect::<Vec<_>>(),
                        [(0.5, "Follow-up visit"), (1.5, "Before visit")]
                    );
                    assert_eq!(
                        selected.axes.as_ref().unwrap().range(AxisSide::Bottom),
                        (0.0, 2.0)
                    );
                    assert_eq!(selected.axes.as_ref().unwrap().range(AxisSide::Left), range);
                    assert_eq!(selected.significance.as_ref().unwrap().y_range, range);
                    assert_eq!(selected.layout.projection.panel_box(), panel);
                    assert_eq!(selected.annotations.as_ref().unwrap().panel_clip, panel);
                    assert_eq!(
                        crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get),
                        reads
                    );
                }
            }
        }
    }
}

#[test]
fn native_distribution_significance_uses_root_feedback_and_final_assigned_geometry() {
    use crate::decorations::plan::{plan_panel_for_test, LegendRole};
    for grouped in [false, true] {
        for source in configured(true, grouped, None) {
            let source = source.with_suppressed_legend().size(900, 700);
            let inputs =
                crate::plots::shared_x::prepare_plot(&source, &OperationContext::new(), None)
                    .unwrap();
            let plot = inputs.attach(source).unwrap();
            for preset in [
                crate::Theme::Publication,
                crate::Theme::Nature,
                crate::Theme::Science,
            ] {
                let theme = preset.resolve().unwrap();
                let plot =
                    crate::plots::shared_x::prepare_plot(&plot, &OperationContext::new(), None)
                        .unwrap()
                        .attach(plot.clone())
                        .unwrap();
                for cell in [
                    PanelBox::new(16.0, 625.0, 8.0, 390.0),
                    PanelBox::new(30.25, 800.5, 40.75, 610.0),
                ] {
                    let range = retained(&plot).unwrap().range();
                    let plot = plot.clone().with_planned_y_limits(range);
                    let initial = plot
                        .measure_selected_interior(PlotRenderContext::new(
                            plot.data(),
                            cell,
                            &theme,
                        ))
                        .unwrap();
                    assert_eq!(initial.layout.projection.y_range(), range);
                    assert!(initial.significance.as_ref().unwrap().y_range.1 > range.1);
                    measurements::start();
                    let plan = plan_panel_for_test(
                        &plot,
                        plot.kind(),
                        cell,
                        LegendRole::Suppressed,
                        &theme,
                    )
                    .unwrap();
                    let work = measurements::finish();
                    assert!(work.roots.len() >= 2);
                    assert_eq!(work.counters[Counter::LegacyPanelConstructor as usize], 0);
                    let final_plot = plan.plot_with_planned_limits(&plot);
                    let selected = final_plot
                        .measure_selected_interior(PlotRenderContext::new(
                            final_plot.data(),
                            plan.panel_box,
                            &theme,
                        ))
                        .unwrap();
                    assert_eq!(
                        selected.layout.projection.y_range(),
                        plan.result.significance.as_ref().unwrap().y_range
                    );
                    assert_eq!(
                        draw(&selected, &final_plot, &theme),
                        draw(&plan.result, &final_plot, &theme)
                    );
                    let required = selected.axes.as_ref().unwrap().requirements();
                    assert_eq!(
                        (
                            plan.panel_box.x0 - cell.x0,
                            cell.x1 - plan.panel_box.x1,
                            plan.panel_box.y0 - cell.y0,
                            cell.y1 - plan.panel_box.y1
                        ),
                        (
                            (required.left + 4.0).ceil(),
                            (required.right + 4.0).ceil(),
                            (required.bottom + 4.0).ceil(),
                            (required.top + 4.0).ceil()
                        )
                    );
                }
            }
        }
    }
}

#[test]
fn native_root_mark_and_annotation_growth_freeze_the_final_domain() {
    use crate::decorations::plan::{plan_panel, LegendRole};
    use crate::layout::kernel::FeedbackKind;
    use crate::layout::{AddressRole, ComponentAddress};
    for source in configured(true, false, None) {
        let source = source
            .with_suppressed_legend()
            .size(900, 700)
            .title("Measured root title")
            .caption("Independent annotation caption");
        let theme = crate::Theme::Publication.resolve().unwrap();
        let owner = ComponentAddress::root()
            .child(AddressRole::PlotBody, 0)
            .unwrap();
        let mut ranges = Vec::new();
        let root = crate::layout::plan_exact_root(
            900,
            700,
            |stage| crate::plots::shared_x::prepare_plot(&source, &stage.operation(), None),
            |inputs, assigned, round| {
                let plot = inputs.attach(source.clone())?;
                let baseline = retained(&plot)?.range();
                let plot = plot.with_planned_y_limits(baseline);
                let consumed = round
                    .feedback_domain(&owner, FeedbackKind::AutomaticDomainY)
                    .unwrap_or(baseline);
                let canvas = plot.measured_canvas_layout(&theme)?;
                round.observe_canvas(&canvas)?;
                let plan = crate::layout::fit::measured(plan_panel(
                    &plot,
                    plot.kind(),
                    canvas.panel_box,
                    LegendRole::Suppressed,
                    &theme,
                    round,
                    &owner,
                ))
                .map_err(crate::layout::fit::PlanError::into_error)?;
                let mut requested = plan.result.significance.as_ref().unwrap().y_range;
                if let Some(marks) = plan.result.marks.as_deref() {
                    if let super::MarkPayload::Distribution(payload) = &marks.payload {
                        if let Some(expansion) = payload.domain_expansions[1] {
                            requested.0 = requested.0.min(expansion.limits.0);
                            requested.1 = requested.1.max(expansion.limits.1);
                        }
                    }
                }
                assert!(requested.0 <= consumed.0 && requested.1 >= consumed.1);
                ranges.push((consumed, requested));
                Ok(crate::layout::MeasuredLayout::new(assigned, plan))
            },
            |_, _, plan, _| Ok(plan),
        )
        .unwrap();
        assert!(ranges.len() >= 2);
        assert!(ranges[0].1 .1 > ranges[0].0 .1);
        assert!(ranges.windows(2).all(|pair| pair[1].0 == pair[0].1));
        let selected = &root.plan().result;
        assert_eq!(
            selected.layout.projection.y_range(),
            selected.significance.as_ref().unwrap().y_range
        );
        assert_eq!(ranges.last().unwrap().0, ranges.last().unwrap().1);
    }
}

#[test]
fn native_distribution_exact_limits_and_labels_are_indivisible_and_never_expand() {
    for sufficient in [false, true] {
        let range = if sufficient {
            (-30.0, 40.0)
        } else {
            (-1.0, 1.0)
        };
        for source in configured(true, false, Some(range)) {
            let source = source.with_test_axis_ticks(
                1,
                vec![
                    crate::AxisTick::new(range.0, "raw_lower"),
                    crate::AxisTick::new(0.0, ""),
                    crate::AxisTick::new(range.1, "raw_upper"),
                ],
            );
            let inputs =
                crate::plots::shared_x::prepare_plot(&source, &OperationContext::new(), None)
                    .unwrap();
            let plot = inputs.attach(source).unwrap();
            let theme = crate::Theme::Publication.resolve().unwrap();
            let result = plot.measure_selected_interior(PlotRenderContext::new(
                plot.data(),
                PanelBox::new(80.0, 600.0, 70.0, 450.0),
                &theme,
            ));
            if sufficient {
                let result = result.unwrap();
                assert_eq!(result.layout.y_axis.source, TickSource::EXACT);
                assert_eq!(
                    result
                        .layout
                        .y_axis
                        .ticks
                        .iter()
                        .map(|tick| tick.label.as_str())
                        .collect::<Vec<_>>(),
                    ["raw_lower", "", "raw_upper"]
                );
                assert_eq!(result.significance.as_ref().unwrap().y_range, range);
                assert_eq!(result.layout.projection.y_range(), range);
            } else {
                assert!(matches!(result, Err(crate::layout::fit::PlanError::Fit(_))));
            }
        }
    }
    for source in sources().into_iter().chain([extraction::supplied()]) {
        for ticks in [
            vec![],
            vec![crate::AxisTick::new(f64::NAN, "invalid")],
            vec![crate::AxisTick::new(1e100, "outside")],
        ] {
            let invalid = source.clone().with_test_axis_ticks(1, ticks);
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
            measurements::start();
            assert!(matches!(invalid.preflight(), Err(crate::Error::Data(_))));
            let work = measurements::finish();
            for counter in [
                Counter::StandardPanelIntrinsic,
                Counter::OneCategoricalAxisMeasure,
                Counter::LegacyPanelConstructor,
            ] {
                assert_eq!(work.counters[counter as usize], 0);
            }
            assert_eq!(
                crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
                0
            );
        }
    }
}

#[test]
fn native_distribution_facets_preserve_cache_off_bytes() {
    use crate::decorations::plan::children::DISABLE_CHILD_REUSE;
    for scales in [
        crate::FacetScales::Fixed,
        crate::FacetScales::FreeX,
        crate::FacetScales::FreeY,
        crate::FacetScales::Free,
    ] {
        for source in extraction::sources(Some(scales)) {
            let mut outputs = Vec::new();
            for disabled in [false, true] {
                STAGES.with(|counts| counts.set([[0; 4]; 3]));
                DISABLE_CHILD_REUSE.with(|flag| flag.set(disabled));
                measurements::start();
                let mut svg = crate::render::svg_backend::SvgBackend::in_memory(1100, 640);
                let result = source.render_with_backend(&mut svg);
                let work = measurements::finish();
                DISABLE_CHILD_REUSE.with(|flag| flag.set(false));
                result.unwrap();
                assert!(work.roots.len() >= 2);
                assert_eq!(work.counters[Counter::LegacyPanelConstructor as usize], 0);
                for counter in [
                    Counter::StandardPanelIntrinsic,
                    Counter::OneCategoricalAxisMeasure,
                    Counter::TickSource,
                    Counter::SignificanceBisection,
                ] {
                    assert_eq!(work.capture_counters[counter as usize], 0);
                }
                let counts = STAGES
                    .with(std::cell::Cell::get)
                    .into_iter()
                    .find(|counts| counts[0] > 0)
                    .unwrap();
                assert_eq!(counts[0], 2);
                assert_eq!(counts[1], counts[2]);
                assert_eq!(counts[3], 2);
                outputs.push(svg.into_bytes().unwrap());
            }
            assert_eq!(outputs[0], outputs[1]);
        }
    }
}

#[test]
fn native_distribution_shared_canvas_prepares_once_with_identical_cache_off_feedback() {
    use crate::decorations::plan::children::DISABLE_CHILD_REUSE;
    fn canvas<
        K: crate::kind::ExactFamily + crate::kind::Series + crate::kind::Significance<crate::Pairwise>,
    >(
        plot: Plot<K>,
    ) -> crate::Canvas {
        let plot = plot
            .series(CategoryValues::new("series", ["one", "one", "two", "two"]).unwrap())
            .significance([crate::Significance::between(
                "a",
                "b",
                crate::SignificanceMark::OneStar,
            )])
            .x_label("First axis line\nSecond axis line\nThird axis line");
        crate::Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, plot.clone())
            .unwrap()
            .panel(0, 1, plot)
            .unwrap()
            .legend_policy(crate::CanvasLegendPolicy::Shared)
            .size(1200, 640)
    }
    let categories = CategoryValues::new("category", ["a", "a", "b", "b"]).unwrap();
    let values = NumericValues::new("value", [1.25, 4.5, 2.25, 7.75]);
    for (family, canvas) in [
        canvas(Plot::boxplot(categories.clone(), values.clone())),
        canvas(Plot::violin(categories.clone(), values.clone())),
        canvas(Plot::raincloud(crate::RaincloudInput::samples(
            categories, values,
        ))),
    ]
    .into_iter()
    .enumerate()
    {
        let mut outputs = Vec::new();
        for disabled in [false, true] {
            STAGES.with(|counts| counts.set([[0; 4]; 3]));
            DISABLE_CHILD_REUSE.with(|flag| flag.set(disabled));
            measurements::start();
            let mut svg = crate::render::svg_backend::SvgBackend::in_memory(1200, 640);
            let result = canvas.render_with_backend(&mut svg);
            let work = measurements::finish();
            DISABLE_CHILD_REUSE.with(|flag| flag.set(false));
            result.unwrap();
            assert!(work.roots.len() >= 2);
            assert!(work
                .gutters
                .iter()
                .any(|change| change.consumed.map(f64::to_bits)
                    != change.requested.map(f64::to_bits)));
            assert_eq!(work.counters[Counter::LegacyPanelConstructor as usize], 0);
            let counts = STAGES.with(std::cell::Cell::get)[family];
            assert_eq!(counts[0], 2);
            assert_eq!(counts[1], counts[2]);
            assert_eq!(counts[3], 2);
            outputs.push(svg.into_bytes().unwrap());
        }
        assert_eq!(outputs[0], outputs[1]);
    }
}

#[test]
fn native_distribution_raw_categories_and_all_overlays_remain_complete_and_unclipped() {
    use crate::render::backend::Backend;
    for overlay in [
        crate::RawOverlay::Hidden,
        crate::RawOverlay::Points,
        crate::RawOverlay::Jittered,
    ] {
        let categories = CategoryValues::new(
            "category",
            [" raw_A ", " raw_A ", " raw_A ", " raw_A ", "raw_B", "raw_B"],
        )
        .unwrap();
        let values = NumericValues::new("value", [1.0, 1.0, 1.0, 9.0, 2.25, 4.5]);
        for (index, source) in [
            Plot::boxplot(categories.clone(), values.clone())
                .raw_overlay(overlay)
                .erase(),
            Plot::violin(categories.clone(), values.clone())
                .raw_overlay(overlay)
                .erase(),
            Plot::raincloud(crate::RaincloudInput::samples(categories, values)).erase(),
        ]
        .into_iter()
        .enumerate()
        {
            let source = source.labels(crate::Labels::raw());
            let inputs =
                crate::plots::shared_x::prepare_plot(&source, &OperationContext::new(), None)
                    .unwrap();
            let plot = inputs.attach(source).unwrap();
            let theme = crate::Theme::Publication.resolve().unwrap();
            let panel = PanelBox::new(80.25, 600.75, 70.5, 450.25);
            let selected = plot
                .measure_selected_interior(PlotRenderContext::new(plot.data(), panel, &theme))
                .unwrap();
            assert_eq!(
                selected.x_tick_keys.as_ref().unwrap(),
                &[" raw_A ", "raw_B"]
            );
            assert_eq!(
                selected
                    .layout
                    .x_axis
                    .ticks
                    .iter()
                    .map(|tick| tick.label.as_str())
                    .collect::<Vec<_>>(),
                [" raw_A ", "raw_B"]
            );
            let mut svg = crate::render::svg_backend::SvgBackend::in_memory(900, 700);
            svg.begin(900, 700).unwrap();
            selected.marks.as_ref().unwrap().draw(&mut svg).unwrap();
            svg.end().unwrap();
            let svg = String::from_utf8(svg.into_bytes().unwrap()).unwrap();
            let expected = if index == 2 || overlay != crate::RawOverlay::Hidden {
                6
            } else {
                usize::from(index == 0)
            };
            assert_eq!(svg.matches("<circle ").count(), expected);
            let axes = selected.axes.as_ref().unwrap().geometry_snapshot().unwrap();
            assert!(axes.bounds().iter().any(|ink| ink.bottom < panel.y0));
            // Fit cannot silently hide a category or put its text under the mark clip.
            let source = plot.labels(crate::Labels::raw().alias("raw_B", "W".repeat(100)));
            let inputs =
                crate::plots::shared_x::prepare_plot(&source, &OperationContext::new(), None)
                    .unwrap();
            let plot = inputs.attach(source).unwrap();
            let projections = crate::layout::Projection::test_projection_calls();
            assert!(matches!(
                plot.measure_selected_interior(PlotRenderContext::new(plot.data(), panel, &theme)),
                Err(crate::layout::fit::PlanError::Fit(_))
            ));
            assert_eq!(
                crate::layout::Projection::test_projection_calls(),
                projections
            );
        }
    }
}
