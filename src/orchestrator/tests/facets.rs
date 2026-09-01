use super::{valid_grouped_bar_values, Plot, PlotKind, RecordingBackend};
use crate::{CategoryValues, NumericValues};

fn bar_plot(category: &CategoryValues, value: &NumericValues, kind: PlotKind) -> Plot {
    match kind {
        PlotKind::Bar => {
            Plot::bar(category.clone(), crate::BarInput::samples(value.clone())).erase()
        }
        PlotKind::HorizontalBar => {
            Plot::horizontal_bar(category.clone(), crate::BarInput::samples(value.clone())).erase()
        }
        _ => panic!("unsupported bar kind {kind:?}"),
    }
}

fn estimate_bar_plot(category: &CategoryValues, value: &NumericValues, kind: PlotKind) -> Plot {
    let value = crate::BarInput::estimates(value.clone());
    match kind {
        PlotKind::Bar => Plot::bar(category.clone(), value).erase(),
        PlotKind::HorizontalBar => Plot::horizontal_bar(category.clone(), value).erase(),
        _ => panic!("unsupported bar kind {kind:?}"),
    }
}

fn positioned_series_bar_plot(
    category: &CategoryValues,
    value: &NumericValues,
    series: &CategoryValues,
    kind: PlotKind,
) -> Plot {
    let value = || crate::BarInput::samples(value.clone());
    match kind {
        PlotKind::Bar => Plot::bar(category.clone(), value())
            .series(series.clone())
            .erase(),
        PlotKind::HorizontalBar => Plot::horizontal_bar(category.clone(), value())
            .series(series.clone())
            .erase(),
        _ => panic!("unsupported bar kind {kind:?}"),
    }
}

#[test]
fn composed_per_panel_outside_legends_remain_in_their_cells() {
    let x = NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]);
    let y = NumericValues::new("y", [1.0, 3.0, 2.0, 5.0]);
    let hue = CategoryValues::new("hue", ["h1", "h2", "h1", "h2"]).unwrap();
    let left = Plot::line(x.clone(), y.clone())
        .group(hue.clone())
        .hue(hue.clone())
        .legend(crate::LegendPolicy::Visible(
            crate::LegendPosition::OutsideRight,
        ))
        .size(900, 360);
    let right = Plot::line(x, y)
        .group(hue.clone())
        .hue(hue)
        .legend(crate::LegendPolicy::Visible(
            crate::LegendPosition::OutsideRight,
        ))
        .size(900, 360);
    let canvas = crate::Canvas::grid(1, 2)
        .unwrap()
        .panel(0, 0, left)
        .unwrap()
        .panel(0, 1, right)
        .unwrap()
        .size(900, 360);
    let mut backend = RecordingBackend::default();

    canvas.render_with_backend(&mut backend).unwrap();

    assert_eq!(backend.rounded_rects.len(), 2);
    assert!(backend.rounded_rects[0].x1 < 450.0);
    assert!(backend.rounded_rects[1].x0 > 450.0);
}

#[test]
fn facets_reserve_one_complete_outside_legend_band() {
    let facet = CategoryValues::new("facet", ["left", "left", "right", "right"]).unwrap();
    let plot = Plot::scatter(
        NumericValues::new("x", [1.0, 2.0, 1.0, 2.0]),
        NumericValues::new("y", [1.0, 2.0, 4.0, 5.0]),
    )
    .hue(CategoryValues::new("hue", ["h1", "h1", "h2", "h2"]).unwrap())
    .facet(crate::FacetSpec::column(facet))
    .legend(crate::LegendPolicy::Visible(
        crate::LegendPosition::OutsideRight,
    ))
    .size(760, 420);
    let mut backend = RecordingBackend::default();

    plot.render_with_backend(&mut backend).unwrap();

    assert_eq!(backend.surfaces.len(), 3);
    assert_eq!(
        backend.surfaces[0].role,
        crate::render::backend::SurfaceRole::CanvasRoot
    );
    assert!(backend.surfaces[1..]
        .iter()
        .all(|surface| surface.role == crate::render::backend::SurfaceRole::PlotPanel));
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
fn facet_strips_preserve_each_preset_foreground_and_header_size() {
    for theme in [
        crate::Theme::Publication,
        crate::Theme::Nature,
        crate::Theme::Science,
        crate::Theme::CHI,
    ] {
        let expected = theme.resolve();
        let facet = CategoryValues::new("facet", ["left", "left", "right", "right"]).unwrap();
        let plot = Plot::scatter(
            NumericValues::new("x", [1.0, 2.0, 1.0, 2.0]),
            NumericValues::new("y", [1.0, 2.0, 4.0, 5.0]),
        )
        .facet(crate::FacetSpec::column(facet))
        .theme(theme)
        .size(760, 420);
        let mut backend = RecordingBackend::default();

        plot.render_with_backend(&mut backend).unwrap();

        let strip_styles = backend
            .text_draws
            .iter()
            .filter(|(_, _, text, _)| text.contains(" = "))
            .map(|(_, _, _, style)| style)
            .collect::<Vec<_>>();
        assert_eq!(strip_styles.len(), 2, "{theme:?}");
        assert!(strip_styles.iter().all(|style| {
            style.color == expected.foreground && style.font_size == expected.facet_header_font_size
        }));
    }
}

#[test]
fn dense_fixed_facets_share_planner_headroom() {
    let facet = CategoryValues::new(
        "facet",
        [
            "left", "left", "left", "left", "left", "left", "left", "left", "right", "right",
            "right", "right",
        ],
    )
    .unwrap();
    let plot = Plot::scatter(
        NumericValues::new(
            "x",
            [0.0, 10.0, 0.0, 10.0, 0.5, 9.5, 0.5, 9.5, 1.0, 2.0, 3.0, 4.0],
        ),
        NumericValues::new(
            "y",
            [0.0, 0.0, 10.0, 9.0, 0.5, 0.5, 9.5, 10.0, 9.0, 2.0, 1.0, 3.0],
        ),
    )
    .hue(
        CategoryValues::new(
            "hue",
            [
                "h1", "h1", "h1", "h1", "h2", "h2", "h2", "h2", "h1", "h1", "h2", "h2",
            ],
        )
        .unwrap(),
    )
    .facet(crate::FacetSpec::column(facet))
    .size(760, 420);
    let mut backend = RecordingBackend::default();

    plot.render_with_backend(&mut backend).unwrap();

    let legend = backend.rounded_rects[0];
    assert!(backend.scatter_points[..8].iter().all(|point| {
        point.x < legend.x0 || point.x > legend.x1 || point.y < legend.y0 || point.y > legend.y1
    }));
    assert!((backend.scatter_points[3].y - backend.scatter_points[10].y).abs() < 0.01);
}

#[test]
fn valid_ungrouped_generated_bar_intervals_render_before_and_after_orientation() {
    let category = CategoryValues::new("group", ["a", "a", "b", "b"]).unwrap();
    let value = NumericValues::new("y", [1.0, 3.0, 2.0, 5.0]);
    for kind in [PlotKind::Bar, PlotKind::HorizontalBar] {
        let plot = bar_plot(&category, &value, kind);
        let mut backend = RecordingBackend::default();
        plot.render_kind(kind, &mut backend).unwrap();
        assert_eq!((backend.begins, backend.ends), (1, 1));
    }
}

#[test]
fn singleton_positioned_series_bar_cells_render_without_error_bars() {
    let category = CategoryValues::new("group", ["a", "a", "b", "b"]).unwrap();
    let value = NumericValues::new("y", [1.0, 3.0, 2.0, 5.0]);
    let series = CategoryValues::new("hue", ["h1", "h2", "h1", "h2"]).unwrap();
    for kind in [PlotKind::Bar, PlotKind::HorizontalBar] {
        let plot = positioned_series_bar_plot(&category, &value, &series, kind);
        let mut backend = RecordingBackend::default();
        plot.render_kind(kind, &mut backend).unwrap();
        assert_eq!((backend.begins, backend.ends), (1, 1), "{kind:?}");
    }
}

#[test]
fn empty_and_nonfinite_ungrouped_cells_fail_before_backend_begin() {
    let cases = [
        (
            CategoryValues::new("group", Vec::<String>::new()).unwrap(),
            NumericValues::new("y", []),
        ),
        (
            CategoryValues::new("group", ["a", "a"]).unwrap(),
            NumericValues::new("y", [1.0, f64::NAN]),
        ),
    ];
    for (category, value) in &cases {
        for kind in [PlotKind::Bar, PlotKind::HorizontalBar] {
            let plot = bar_plot(category, value, kind);
            let mut backend = RecordingBackend::default();
            assert!(plot.render_kind(kind, &mut backend).is_err());
            assert_eq!(backend.begins, 0, "{kind:?}");
        }
    }
}

#[test]
fn valid_positioned_series_bars_keep_provenance_off_canvas() {
    let data = valid_grouped_bar_values();
    for kind in [PlotKind::Bar, PlotKind::HorizontalBar] {
        let plot = positioned_series_bar_plot(
            &data.category,
            &data.value,
            data.series.as_ref().unwrap(),
            kind,
        );
        let mut backend = RecordingBackend::default();
        plot.render_kind(kind, &mut backend).unwrap();
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
    let category = CategoryValues::new("group", ["a", "a"]).unwrap();
    let value = NumericValues::new("y", [1.0, 2.0]);
    let facet = CategoryValues::new("facet", ["left", "right"]).unwrap();
    for kind in [PlotKind::Bar, PlotKind::HorizontalBar] {
        let value = || crate::BarInput::samples(value.clone());
        let plot = match kind {
            PlotKind::Bar => Plot::bar(category.clone(), value())
                .facet(crate::FacetSpec::column(facet.clone()))
                .erase(),
            PlotKind::HorizontalBar => Plot::horizontal_bar(category.clone(), value())
                .facet(crate::FacetSpec::column(facet.clone()))
                .erase(),
            _ => unreachable!(),
        };
        let mut backend = RecordingBackend::default();
        plot.render_kind(kind, &mut backend).unwrap();
        assert_eq!((backend.begins, backend.ends), (1, 1), "{kind:?}");
    }
}

#[test]
fn estimate_bars_accept_one_finite_row_and_reject_duplicates_before_begin() {
    let valid = CategoryValues::new("group", ["a", "b"]).unwrap();
    let duplicate = CategoryValues::new("group", ["a", "a"]).unwrap();
    let value = NumericValues::new("y", [1.0, 2.0]);
    for kind in [PlotKind::Bar, PlotKind::HorizontalBar] {
        let plot = estimate_bar_plot(&valid, &value, kind);
        let mut backend = RecordingBackend::default();
        plot.render_kind(kind, &mut backend).unwrap();
        assert!(backend
            .texts
            .iter()
            .all(|(_, _, text)| !text.contains("mean")));

        let plot = estimate_bar_plot(&duplicate, &value, kind);
        let mut backend = RecordingBackend::default();
        let error = plot.render_kind(kind, &mut backend).unwrap_err();
        assert!(error.to_string().contains("exactly one is required"));
        assert_eq!(backend.begins, 0);
    }
}
