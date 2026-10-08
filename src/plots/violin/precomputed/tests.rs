use super::*;
use crate::font::canonical::OperationContext;

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
    let mut theme = crate::Theme::Publication.resolve().unwrap();
    theme.marks.violin_body.outline = Some(crate::theme::DataOutlineTheme::fixed(
        theme
            .marks
            .violin_body
            .outline()
            .unwrap()
            .fixed_stroke()
            .unwrap()
            .clone()
            .with_width(120.0)
            .unwrap(),
    ));
    theme.marks.raw_overlay_radius = 50;
    let resources = theme.prepared_font_environment().unwrap().operation();
    let inputs = crate::plots::shared_x::prepare_plot(&source, &resources, None).unwrap();
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
    let resources = theme.prepared_font_environment().unwrap().operation();
    let inputs = crate::plots::shared_x::prepare_plot(&source, &resources, None).unwrap();
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
    let mut ink = crate::render::geometry::GeometrySnapshot::new();
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
    let resources = theme.prepared_font_environment().unwrap().operation();
    let inputs = crate::plots::shared_x::prepare_plot(&exact_source, &resources, None).unwrap();
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
fn prepared_hue_domain_preserves_order_and_obeys_exact_work_and_storage_caps() {
    let source = source(true);
    HUE_LOOKUP_VISITS.with(|count| count.set(0));

    let prepared = prepare(&source).unwrap();
    assert_eq!(prepared.hues, ["z", "a", "m"]);
    assert_eq!(
        prepared
            .groups
            .iter()
            .map(|group| group.geometry.hue.as_deref())
            .collect::<Vec<_>>(),
        [Some("z"), Some("a"), Some("z"), Some("m")]
    );
    // Nonzero evidence distinguishes retained semantic work from an unused probe.
    assert_eq!(HUE_LOOKUP_VISITS.with(std::cell::Cell::get), 5);

    prepare(&source).unwrap();
}

#[test]
fn repeated_measurement_uses_retained_raw_keys() {
    for with_hues in [true, false] {
        let source = source(with_hues);
        HUE_LOOKUP_VISITS.with(|count| count.set(0));
        let inputs =
            crate::plots::shared_x::prepare_plot(&source, &OperationContext::new(), None).unwrap();
        let source = inputs.attach(source).unwrap();
        let lookup_visits = HUE_LOOKUP_VISITS.with(std::cell::Cell::get);
        assert_eq!(lookup_visits, if with_hues { 5 } else { 0 });
        let crate::plots::categorical_distribution::PreparedDistribution::Supplied(prepared) =
            crate::plots::categorical_distribution::retained(&source).unwrap()
        else {
            unreachable!()
        };
        let theme = crate::Theme::Publication.resolve().unwrap();
        let resources = theme.prepared_font_environment().unwrap().operation();
        let colors = crate::plots::categorical_distribution::retained(&source)
            .unwrap()
            .resolved_colors(&source, &theme, source.data().with_operation(&resources))
            .unwrap();
        for (index, group) in prepared.groups.iter().enumerate() {
            let expected = if with_hues {
                [0, 1, 0, 2][index]
            } else {
                index
            };
            assert_eq!(
                colors
                    .color(
                        group
                            .geometry
                            .hue
                            .as_deref()
                            .unwrap_or(&group.geometry.category)
                    )
                    .unwrap(),
                theme.color_scales.qualitative().colors()[expected].as_rgb()
            );
        }
        inputs.prepare_intrinsics(&source, &theme).unwrap();
        let measure = || {
            source.measure_selected_interior(PlotRenderContext::new(
                source.data(),
                crate::layout::PanelBox::new(20.25, 1420.75, 30.5, 350.25),
                &theme,
            ))
        };
        for _ in 0..2 {
            crate::orchestrator::measurements::start();
            measure().unwrap();
            let work = crate::orchestrator::measurements::finish();
            assert_eq!(
                work.counters
                    [crate::orchestrator::measurements::Counter::StandardPanelIntrinsic as usize],
                0
            );
            assert_eq!(HUE_LOOKUP_VISITS.with(std::cell::Cell::get), lookup_visits);
        }

        measure().unwrap();

        assert_eq!(HUE_LOOKUP_VISITS.with(std::cell::Cell::get), lookup_visits);
        assert_eq!(HUE_LOOKUP_VISITS.with(std::cell::Cell::get), lookup_visits);
    }
}
