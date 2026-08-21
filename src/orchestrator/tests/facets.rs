use polars::prelude::NamedFrom;

use super::{
    configured_chart, fixture, valid_grouped_bar_fixture, Chart, DataFrame, PlotKind,
    RecordingBackend, Series,
};

#[test]
fn composed_per_panel_outside_legends_remain_in_their_cells() {
    let data = fixture();
    let mut left = Chart::new(&data).group("hue").hue("hue");
    let _ = left
        .x("x")
        .y("y")
        .legend_position(crate::LegendPosition::OutsideRight)
        .size(900, 360)
        .line();
    let mut right = Chart::new(&data).group("hue").hue("hue");
    let _ = right
        .x("x")
        .y("y")
        .legend_position(crate::LegendPosition::OutsideRight)
        .size(900, 360)
        .line();
    let figure = crate::Figure::grid(1, 2)
        .unwrap()
        .panel(0, 0, left)
        .unwrap()
        .panel(0, 1, right)
        .unwrap()
        .size(900, 360);
    let mut backend = RecordingBackend::default();

    figure.render_with_backend(&mut backend).unwrap();

    assert_eq!(backend.rounded_rects.len(), 2);
    assert!(backend.rounded_rects[0].x1 < 450.0);
    assert!(backend.rounded_rects[1].x0 > 450.0);
}

#[test]
fn facets_reserve_one_complete_outside_legend_band() {
    let data = DataFrame::new(
        4,
        vec![
            Series::new("facet".into(), ["left", "left", "right", "right"]).into(),
            Series::new("hue".into(), ["h1", "h1", "h2", "h2"]).into(),
            Series::new("x".into(), [1.0_f64, 2.0, 1.0, 2.0]).into(),
            Series::new("y".into(), [1.0_f64, 2.0, 4.0, 5.0]).into(),
        ],
    )
    .unwrap();
    let mut chart = Chart::new(&data).hue("hue").facet_col("facet");
    let _ = chart
        .x("x")
        .y("y")
        .legend_position(crate::LegendPosition::OutsideRight)
        .size(760, 420)
        .scatter();
    let mut backend = RecordingBackend::default();

    chart.render_with_backend(&mut backend).unwrap();

    assert_eq!(backend.rounded_rects.len(), 1);
    assert!(backend.rounded_rects[0].x1 <= 760.0);
    assert_eq!(
        backend
            .texts
            .iter()
            .filter(|(_, _, text)| text == "H 1")
            .count(),
        1
    );
    assert_eq!(
        backend
            .texts
            .iter()
            .filter(|(_, _, text)| text == "H 2")
            .count(),
        1
    );
}

#[test]
fn dense_fixed_facets_share_planner_headroom() {
    let data = DataFrame::new(
        12,
        vec![
            Series::new(
                "facet".into(),
                [
                    "left", "left", "left", "left", "left", "left", "left", "left", "right",
                    "right", "right", "right",
                ],
            )
            .into(),
            Series::new(
                "hue".into(),
                [
                    "h1", "h1", "h1", "h1", "h2", "h2", "h2", "h2", "h1", "h1", "h2", "h2",
                ],
            )
            .into(),
            Series::new(
                "x".into(),
                [
                    0.0_f64, 10.0, 0.0, 10.0, 0.5, 9.5, 0.5, 9.5, 1.0, 2.0, 3.0, 4.0,
                ],
            )
            .into(),
            Series::new(
                "y".into(),
                [
                    0.0_f64, 0.0, 10.0, 9.0, 0.5, 0.5, 9.5, 10.0, 9.0, 2.0, 1.0, 3.0,
                ],
            )
            .into(),
        ],
    )
    .unwrap();
    let mut chart = Chart::new(&data).hue("hue").facet_col("facet");
    let _ = chart.x("x").y("y").size(760, 420).scatter();
    let mut backend = RecordingBackend::default();

    chart.render_with_backend(&mut backend).unwrap();

    let legend = backend.rounded_rects[0];
    assert!(backend.scatter_points[..8].iter().all(|point| {
        point.x < legend.x0 || point.x > legend.x1 || point.y < legend.y0 || point.y > legend.y1
    }));
    assert!((backend.scatter_points[3].y - backend.scatter_points[10].y).abs() < 0.01);
}

#[test]
fn valid_ungrouped_generated_bar_intervals_render_before_and_after_orientation() {
    let data = fixture();
    for kind in [PlotKind::Bar, PlotKind::HorizontalBar] {
        let chart = configured_chart(&data, kind);
        let mut backend = RecordingBackend::default();
        chart.render_kind(kind, &mut backend).unwrap();
        assert_eq!((backend.begins, backend.ends), (1, 1));
    }
}

fn caller_interval_semantics() -> crate::IntervalSemantics {
    crate::IntervalSemantics::new(
        crate::IntervalKind::Confidence,
        0.95,
        crate::InferentialScale::Response,
        "caller-provided interval",
    )
    .unwrap()
}

#[test]
fn caller_interval_semantics_cannot_forge_generated_bar_claims() {
    let data = fixture();
    for kind in [PlotKind::Bar, PlotKind::HorizontalBar] {
        let chart = configured_chart(&data, kind).interval_semantics(caller_interval_semantics());
        let mut backend = RecordingBackend::default();
        let error = chart.render_kind(kind, &mut backend).unwrap_err();
        assert!(error.to_string().contains("owned by BarAggregation"));
        assert_eq!(backend.begins, 0, "{kind:?}");
    }
}

#[test]
fn every_bar_family_rejects_ignored_bounds_before_backend_begin() {
    let data = fixture();
    for kind in [
        PlotKind::Bar,
        PlotKind::HorizontalBar,
        PlotKind::StackedBar,
        PlotKind::StackedHorizontalBar,
    ] {
        let chart = configured_chart(&data, kind).interval_bounds("lower", "upper");
        let mut backend = RecordingBackend::default();
        assert!(chart.render_kind(kind, &mut backend).is_err(), "{kind:?}");
        assert_eq!(backend.begins, 0, "{kind:?}");
    }
}

#[test]
fn unsupported_bar_subfamilies_reject_semantics_before_backend_begin() {
    let data = fixture();
    for kind in [PlotKind::StackedBar, PlotKind::StackedHorizontalBar] {
        let chart = configured_chart(&data, kind).interval_semantics(caller_interval_semantics());
        let mut backend = RecordingBackend::default();
        assert!(chart.render_kind(kind, &mut backend).is_err(), "{kind:?}");
        assert_eq!(backend.begins, 0, "{kind:?}");
    }
}

#[test]
fn singleton_grouped_generated_bar_cells_fail_before_backend_begin() {
    let data = fixture();
    for kind in [PlotKind::Bar, PlotKind::HorizontalBar] {
        let mut chart = Chart::new(&data).group("hue").hue("hue");
        let _ = chart.x("group");
        let _ = chart.y("y");
        let _ = chart.select(kind);
        let mut backend = RecordingBackend::default();
        let error = chart.render_kind(kind, &mut backend).unwrap_err();
        assert!(error.to_string().contains("at least two are required"));
        assert_eq!(backend.begins, 0, "{kind:?}");
    }
}

#[test]
fn empty_singleton_and_nonfinite_ungrouped_cells_fail_before_backend_begin() {
    let fixtures = [
        DataFrame::new(
            0,
            vec![
                Series::new("group".into(), Vec::<String>::new()).into(),
                Series::new("y".into(), Vec::<f64>::new()).into(),
            ],
        )
        .unwrap(),
        DataFrame::new(
            2,
            vec![
                Series::new("group".into(), ["a", "b"]).into(),
                Series::new("y".into(), [1.0_f64, 2.0]).into(),
            ],
        )
        .unwrap(),
        DataFrame::new(
            2,
            vec![
                Series::new("group".into(), ["a", "a"]).into(),
                Series::new("y".into(), [1.0_f64, f64::NAN]).into(),
            ],
        )
        .unwrap(),
    ];
    for data in &fixtures {
        for kind in [PlotKind::Bar, PlotKind::HorizontalBar] {
            let mut chart = Chart::new(data);
            let _ = chart.x("group");
            let _ = chart.y("y");
            let _ = chart.select(kind);
            let mut backend = RecordingBackend::default();
            assert!(chart.render_kind(kind, &mut backend).is_err());
            assert_eq!(backend.begins, 0, "{kind:?}");
        }
    }
}

#[test]
fn valid_grouped_vertical_and_horizontal_bars_keep_provenance_off_canvas() {
    let data = valid_grouped_bar_fixture();
    for kind in [PlotKind::Bar, PlotKind::HorizontalBar] {
        let mut chart = Chart::new(&data).group("hue").hue("hue");
        let _ = chart.x("group");
        let _ = chart.y("y");
        let _ = chart.select(kind);
        let mut backend = RecordingBackend::default();
        chart.render_kind(kind, &mut backend).unwrap();
        let rendered_text = backend
            .texts
            .iter()
            .map(|(_, _, text)| text.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        assert!(!rendered_text.contains(crate::BarAggregation::Mean.provenance()));
    }
}

#[test]
fn generated_bar_sample_sizes_are_not_pooled_across_facets() {
    let data = DataFrame::new(
        2,
        vec![
            Series::new("group".into(), ["a", "a"]).into(),
            Series::new("facet".into(), ["left", "right"]).into(),
            Series::new("y".into(), [1.0_f64, 2.0]).into(),
        ],
    )
    .unwrap();
    for kind in [PlotKind::Bar, PlotKind::HorizontalBar] {
        let mut chart = Chart::new(&data).facet_col("facet");
        let _ = chart.x("group");
        let _ = chart.y("y");
        let _ = chart.select(kind);
        let mut backend = RecordingBackend::default();
        let error = chart.render_kind(kind, &mut backend).unwrap_err();
        assert!(error.to_string().contains("facet facet=left"));
        assert_eq!(backend.begins, 0, "{kind:?}");
    }
}

#[test]
fn identity_bars_accept_one_finite_row_and_reject_duplicates_before_begin() {
    let valid = DataFrame::new(
        2,
        vec![
            Series::new("group".into(), ["a", "b"]).into(),
            Series::new("y".into(), [1.0_f64, 2.0]).into(),
        ],
    )
    .unwrap();
    let duplicate = DataFrame::new(
        2,
        vec![
            Series::new("group".into(), ["a", "a"]).into(),
            Series::new("y".into(), [1.0_f64, 2.0]).into(),
        ],
    )
    .unwrap();
    for kind in [PlotKind::Bar, PlotKind::HorizontalBar] {
        let mut chart = Chart::new(&valid).bar_aggregation(crate::BarAggregation::Identity);
        let _ = chart.x("group").y("y").select(kind);
        let mut backend = RecordingBackend::default();
        chart.render_kind(kind, &mut backend).unwrap();
        assert!(backend
            .texts
            .iter()
            .all(|(_, _, text)| !text.contains(crate::BarAggregation::Identity.provenance())));

        let mut chart = Chart::new(&duplicate).bar_aggregation(crate::BarAggregation::Identity);
        let _ = chart.x("group").y("y").select(kind);
        let mut backend = RecordingBackend::default();
        let error = chart.render_kind(kind, &mut backend).unwrap_err();
        assert!(error.to_string().contains("exactly one is required"));
        assert_eq!(backend.begins, 0);
    }
}
