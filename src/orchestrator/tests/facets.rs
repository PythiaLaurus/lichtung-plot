use super::{
    configured_chart, fixture, valid_grouped_bar_fixture, DataFrame, Plot, PlotKind,
    RecordingBackend, Series,
};

fn bar_chart(data: &DataFrame, kind: PlotKind) -> Plot {
    let category = crate::test_support::category(data, "group");
    let value = crate::test_support::numeric(data, "y");
    match kind {
        PlotKind::Bar => Plot::bar(category, crate::BarInput::samples(value)).erase(),
        PlotKind::HorizontalBar => {
            Plot::horizontal_bar(category, crate::BarInput::samples(value)).erase()
        }
        _ => panic!("unsupported bar kind {kind:?}"),
    }
}

fn estimate_bar_chart(data: &DataFrame, kind: PlotKind) -> Plot {
    let category = crate::test_support::category(data, "group");
    let value = crate::BarInput::estimates(crate::test_support::numeric(data, "y"));
    match kind {
        PlotKind::Bar => Plot::bar(category, value).erase(),
        PlotKind::HorizontalBar => Plot::horizontal_bar(category, value).erase(),
        _ => panic!("unsupported bar kind {kind:?}"),
    }
}

fn positioned_series_bar_chart(data: &DataFrame, kind: PlotKind) -> Plot {
    let category = crate::test_support::category(data, "group");
    let value = || crate::BarInput::samples(crate::test_support::numeric(data, "y"));
    let series = || crate::test_support::category(data, "hue");
    match kind {
        PlotKind::Bar => Plot::bar(category, value()).series(series()).erase(),
        PlotKind::HorizontalBar => Plot::horizontal_bar(category, value())
            .series(series())
            .erase(),
        _ => panic!("unsupported bar kind {kind:?}"),
    }
}

#[test]
fn composed_per_panel_outside_legends_remain_in_their_cells() {
    let data = fixture();
    let left = Plot::line(
        crate::test_support::numeric(&data, "x"),
        crate::test_support::numeric(&data, "y"),
    )
    .group(crate::test_support::category(&data, "hue"))
    .hue(crate::test_support::category(&data, "hue"))
    .legend(crate::LegendPolicy::Visible(
        crate::LegendPosition::OutsideRight,
    ))
    .size(900, 360);
    let right = Plot::line(
        crate::test_support::numeric(&data, "x"),
        crate::test_support::numeric(&data, "y"),
    )
    .group(crate::test_support::category(&data, "hue"))
    .hue(crate::test_support::category(&data, "hue"))
    .legend(crate::LegendPolicy::Visible(
        crate::LegendPosition::OutsideRight,
    ))
    .size(900, 360);
    let figure = crate::Canvas::grid(1, 2)
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
    let chart = Plot::scatter(
        crate::test_support::numeric(&data, "x"),
        crate::test_support::numeric(&data, "y"),
    )
    .hue(crate::test_support::category(&data, "hue"))
    .facet(crate::FacetSpec::column(crate::test_support::category(
        &data, "facet",
    )))
    .legend(crate::LegendPolicy::Visible(
        crate::LegendPosition::OutsideRight,
    ))
    .size(760, 420);
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
    let chart = Plot::scatter(
        crate::test_support::numeric(&data, "x"),
        crate::test_support::numeric(&data, "y"),
    )
    .hue(crate::test_support::category(&data, "hue"))
    .facet(crate::FacetSpec::column(crate::test_support::category(
        &data, "facet",
    )))
    .size(760, 420);
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

#[test]
fn singleton_positioned_series_bar_cells_render_without_error_bars() {
    let data = fixture();
    for kind in [PlotKind::Bar, PlotKind::HorizontalBar] {
        let chart = positioned_series_bar_chart(&data, kind);
        let mut backend = RecordingBackend::default();
        chart.render_kind(kind, &mut backend).unwrap();
        assert_eq!((backend.begins, backend.ends), (1, 1), "{kind:?}");
    }
}

#[test]
fn empty_and_nonfinite_ungrouped_cells_fail_before_backend_begin() {
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
                Series::new("group".into(), ["a", "a"]).into(),
                Series::new("y".into(), [1.0_f64, f64::NAN]).into(),
            ],
        )
        .unwrap(),
    ];
    for data in &fixtures {
        for kind in [PlotKind::Bar, PlotKind::HorizontalBar] {
            let chart = bar_chart(data, kind);
            let mut backend = RecordingBackend::default();
            assert!(chart.render_kind(kind, &mut backend).is_err());
            assert_eq!(backend.begins, 0, "{kind:?}");
        }
    }
}

#[test]
fn valid_positioned_series_bars_keep_provenance_off_canvas() {
    let data = valid_grouped_bar_fixture();
    for kind in [PlotKind::Bar, PlotKind::HorizontalBar] {
        let chart = positioned_series_bar_chart(&data, kind);
        let mut backend = RecordingBackend::default();
        chart.render_kind(kind, &mut backend).unwrap();
        let rendered_text = backend
            .texts
            .iter()
            .map(|(_, _, text)| text.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        assert!(!rendered_text.contains("bootstrap"));
    }
}

#[test]
fn singleton_generated_bar_samples_are_isolated_across_facets() {
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
        let category = crate::test_support::category(&data, "group");
        let value = || crate::BarInput::samples(crate::test_support::numeric(&data, "y"));
        let facet = || crate::test_support::category(&data, "facet");
        let chart = match kind {
            PlotKind::Bar => Plot::bar(category, value())
                .facet(crate::FacetSpec::column(facet()))
                .erase(),
            PlotKind::HorizontalBar => Plot::horizontal_bar(category, value())
                .facet(crate::FacetSpec::column(facet()))
                .erase(),
            _ => unreachable!(),
        };
        let mut backend = RecordingBackend::default();
        chart.render_kind(kind, &mut backend).unwrap();
        assert_eq!((backend.begins, backend.ends), (1, 1), "{kind:?}");
    }
}

#[test]
fn estimate_bars_accept_one_finite_row_and_reject_duplicates_before_begin() {
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
        let chart = estimate_bar_chart(&valid, kind);
        let mut backend = RecordingBackend::default();
        chart.render_kind(kind, &mut backend).unwrap();
        assert!(backend
            .texts
            .iter()
            .all(|(_, _, text)| !text.contains("mean")));

        let chart = estimate_bar_chart(&duplicate, kind);
        let mut backend = RecordingBackend::default();
        let error = chart.render_kind(kind, &mut backend).unwrap_err();
        assert!(error.to_string().contains("exactly one is required"));
        assert_eq!(backend.begins, 0);
    }
}
