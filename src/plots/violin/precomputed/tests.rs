use super::*;
use crate::font::canonical::OperationGenerationResources;

fn source(with_hues: bool) -> Plot {
    source_with_limits(with_hues, None)
}

fn source_with_limits(with_hues: bool, limits: Option<(f64, f64)>) -> Plot {
    let chart = Plot::raincloud(crate::RaincloudInput::geometry(crate::RaincloudGeometry {
        dimensions: crate::RaincloudDimensions {
            category: "category".into(),
            value: "value".into(),
            hue: with_hues.then(|| "color".into()),
        },
        groups: ["z", "a", "z", "m"]
            .into_iter()
            .enumerate()
            .map(|(index, hue)| RaincloudGroup {
                category: format!("category {index}"),
                hue: with_hues.then(|| hue.into()),
                density: vec![
                    crate::RaincloudDensityPoint {
                        y: 0.0,
                        normalized_width: 0.0,
                    },
                    crate::RaincloudDensityPoint {
                        y: 3.0,
                        normalized_width: 1.0,
                    },
                    crate::RaincloudDensityPoint {
                        y: 6.0,
                        normalized_width: 0.0,
                    },
                ],
                q1: 1.0,
                median: 3.0,
                q3: 5.0,
                points: vec![crate::RaincloudPoint {
                    y: 3.0,
                    normalized_strip: 0.5,
                }],
            })
            .collect(),
    }));
    if let Some((lo, hi)) = limits {
        chart.y_limits(lo, hi).erase()
    } else {
        chart.erase()
    }
}

#[test]
fn supplied_raincloud_cloud_and_strip_ink_fit_automatic_y_at_root() {
    let source = source(true).size(800, 600);
    let mut theme = crate::Theme::Publication.resolve();
    theme.marks.violin_body.outline_width = 120.0;
    theme.marks.raw_overlay_radius = 50;
    let resources = theme
        .prepared_font_environment()
        .unwrap()
        .generation_resources();
    let inputs = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
    let chart = inputs.attach(source).unwrap();
    let narrow = crate::layout::PanelBox::new(80.0, 700.0, 80.0, 520.0);
    assert!(chart.intrinsic_body_minimum(&theme).unwrap().0 > narrow.width());
    let error = crate::decorations::plan::plan_panel_for_test(
        &chart,
        crate::PlotKind::Raincloud,
        narrow,
        crate::decorations::plan::LegendRole::Suppressed,
        &theme,
    )
    .err()
    .expect("painted strip cannot fit this declared category slot")
    .to_string();
    assert!(error.contains("TrackOrResidual"), "{error}");

    let source = self::source(true).size(5000, 1400);
    let resources = theme
        .prepared_font_environment()
        .unwrap()
        .generation_resources();
    let inputs = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
    let chart = inputs.attach(source).unwrap();
    let cell = crate::layout::PanelBox::new(80.0, 4900.0, 80.0, 1300.0);
    let selected = crate::decorations::plan::plan_panel_for_test(
        &chart,
        crate::PlotKind::Raincloud,
        cell,
        crate::decorations::plan::LegendRole::Suppressed,
        &theme,
    )
    .unwrap();
    let crate::layout::interior::MarkPayload::Distribution(payload) =
        &selected.result.marks.as_ref().unwrap().payload
    else {
        panic!("selected Raincloud")
    };
    assert!(payload.domain_expansions.iter().all(Option::is_none));
    let panel = selected.result.layout.projection.panel_box();
    let mut ink = crate::render::geometry::GeometrySnapshot::new(&theme).unwrap();
    for mark in &payload.marks {
        match mark {
            crate::plots::categorical_distribution::Mark::Polygon(points, style) => {
                ink.polygon(points, style, None).unwrap();
            }
            crate::plots::categorical_distribution::Mark::Line(points, style) => {
                ink.line(points, style, None).unwrap();
            }
            crate::plots::categorical_distribution::Mark::Scatter(points, style) => {
                ink.scatter_with_clip(points, style, None).unwrap();
            }
        }
    }
    assert!(!ink.bounds().is_empty());
    for bound in ink.bounds() {
        assert!(bound.left >= panel.x0 && bound.right <= panel.x1);
        assert!(bound.bottom >= panel.y0 && bound.top <= panel.y1);
    }
    let exact_source = source_with_limits(true, Some((1.0, 5.0))).size(5000, 1400);
    let resources = theme
        .prepared_font_environment()
        .unwrap()
        .generation_resources();
    let inputs = crate::plots::shared_x::prepare_plot(&exact_source, &resources).unwrap();
    let exact_chart = inputs.attach(exact_source).unwrap();
    let exact = crate::decorations::plan::plan_panel_for_test(
        &exact_chart,
        crate::PlotKind::Raincloud,
        cell,
        crate::decorations::plan::LegendRole::Suppressed,
        &theme,
    )
    .unwrap();
    assert!(exact.planned_y_limits().is_none());
}

#[test]
fn prepared_hue_indices_preserve_order_and_obey_exact_work_and_storage_caps() {
    let source = source(true);
    HUE_LOOKUP_VISITS.with(|count| count.set(0));
    let resources = OperationGenerationResources::new();
    let prepared = prepare(&source, source.data().with_axis_resources(&resources)).unwrap();
    assert_eq!(prepared.hues, ["z", "a", "m"]);
    assert_eq!(
        prepared
            .groups
            .iter()
            .map(|group| group.local_color_index)
            .collect::<Vec<_>>(),
        [0, 1, 0, 2]
    );
    // Nonzero evidence distinguishes retained semantic work from an unused probe.
    assert_eq!(HUE_LOOKUP_VISITS.with(std::cell::Cell::get), 5);
    let visits = resources.usage().unwrap().1;
    let bytes = resources.byte_usage().unwrap();
    assert!(visits >= 5);
    assert!(bytes >= u64::try_from(4 * std::mem::size_of::<PreparedRaincloudGroup>()).unwrap());
    for (visits, bytes, diagnostic) in [
        (visits - 1, bytes, "work admission"),
        (visits, bytes - 1, "storage admission"),
    ] {
        let limited = OperationGenerationResources::with_test_limits(visits, bytes);
        let error = prepare(&source, source.data().with_axis_resources(&limited)).unwrap_err();
        assert!(error.to_string().contains(diagnostic), "{error}");
    }
    let exact = OperationGenerationResources::with_test_limits(visits, bytes);
    prepare(&source, source.data().with_axis_resources(&exact)).unwrap();
    assert_eq!(exact.usage().unwrap().1, visits);
    assert_eq!(exact.byte_usage().unwrap(), bytes);
}

#[test]
fn repeated_measurement_uses_retained_indices_and_charges_candidate_work() {
    for with_hues in [true, false] {
        let source = source(with_hues);
        HUE_LOOKUP_VISITS.with(|count| count.set(0));
        let inputs =
            crate::plots::shared_x::prepare_plot(&source, &OperationGenerationResources::new())
                .unwrap();
        let source = inputs.attach(source).unwrap();
        let lookup_visits = HUE_LOOKUP_VISITS.with(std::cell::Cell::get);
        assert_eq!(lookup_visits, if with_hues { 5 } else { 0 });
        let crate::plots::categorical_distribution::PreparedDistribution::Supplied(prepared) =
            crate::plots::categorical_distribution::retained(&source).unwrap()
        else {
            unreachable!()
        };
        let theme = crate::Theme::Publication.resolve();
        for (index, group) in prepared.groups.iter().enumerate() {
            let expected = if with_hues {
                [0, 1, 0, 2][index]
            } else {
                index
            };
            assert_eq!(group.local_color_index, expected);
            assert_eq!(
                group_color(&source, &group.geometry, group.local_color_index, &theme).unwrap(),
                theme.colors[expected % theme.colors.len()]
            );
        }
        let resources = theme
            .prepared_font_environment()
            .unwrap()
            .generation_resources();
        inputs.prepare_intrinsics(&source, &theme).unwrap();
        let measure = || {
            source.measure_selected_interior(PlotRenderContext::new(
                source.data(),
                crate::layout::PanelBox::new(20.25, 1420.75, 30.5, 350.25),
                &theme,
            ))
        };
        let mut candidate_visits = 0;
        for _ in 0..2 {
            let before = resources.usage().unwrap().1;
            crate::orchestrator::measurements::start();
            measure().unwrap();
            let work = crate::orchestrator::measurements::finish();
            assert_eq!(
                work.counters
                    [crate::orchestrator::measurements::Counter::StandardPanelIntrinsic as usize],
                0
            );
            candidate_visits = resources.usage().unwrap().1 - before;
            assert!(candidate_visits > 0);
            assert_eq!(HUE_LOOKUP_VISITS.with(std::cell::Cell::get), lookup_visits);
        }
        let limit = resources.usage().unwrap().1 + candidate_visits;
        resources.set_test_limits(limit, u64::MAX);
        measure().unwrap();
        assert_eq!(resources.usage().unwrap().1, limit);
        assert_eq!(HUE_LOOKUP_VISITS.with(std::cell::Cell::get), lookup_visits);
        let projections = crate::layout::Projection::test_projection_calls();
        let shapes = crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get);
        let calls = resources.call_cache_usage().0;
        let error = measure().unwrap_err().into_error();
        // The first owned variable scan now counts every supplied category
        // slot before the cached numeric tick-pitch lookup can begin.
        let expected = crate::font::canonical::TextGenerationClaimError::Limit {
            resource: "root processing visits",
            limit,
            cumulative_claim: limit + prepared.groups.len() as u128,
        };
        assert!(
            matches!(&error, Error::Render(message) if message == &format!("supplied Raincloud slot admission: {expected:?}")),
            "{error}; with_hues={with_hues}"
        );
        assert_eq!(resources.usage().unwrap().1, limit);
        // Interior and cached axis intrinsics are admitted; numeric/text
        // lookups never start after the earlier source-slot work refusal.
        assert_eq!(resources.call_cache_usage().0 - calls, 2);
        assert_eq!(
            crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get),
            shapes
        );
        assert_eq!(
            crate::layout::Projection::test_projection_calls(),
            projections
        );
        assert_eq!(HUE_LOOKUP_VISITS.with(std::cell::Cell::get), lookup_visits);
    }
}
