use super::{assert_rejected_before_begin, HeatmapLayout, Plot, PlotKind, RecordingBackend};
use crate::{CategoryValues, NumericValues, RawOverlay, SourceRows};

fn assert_strict_numeric_faults<K, F>(field: &str, chart_with_fault: F)
where
    K: crate::kind::Marker,
    F: Fn(Option<f64>) -> Plot<K>,
{
    for fault in [
        None,
        Some(f64::NAN),
        Some(f64::INFINITY),
        Some(f64::NEG_INFINITY),
    ] {
        let chart = chart_with_fault(fault);
        let kind = chart.kind();
        let error = assert_rejected_before_begin(&chart, kind);
        assert!(error.contains(field), "{kind:?}: {error}");
        assert!(error.contains("source row 1"), "{kind:?}: {error}");
    }
}

fn assert_aligned_numeric_faults<K, F>(field: &str, chart_with_fault: F)
where
    K: crate::kind::Marker,
    F: Fn(Option<f64>) -> Plot<K>,
{
    for fault in [None, Some(f64::NAN)] {
        let chart = chart_with_fault(fault);
        let kind = chart.kind();
        let mut backend = RecordingBackend::default();
        chart.render_kind(kind, &mut backend).unwrap();
        assert_eq!((backend.begins, backend.ends), (1, 1), "{kind:?}");
    }
    for fault in [Some(f64::INFINITY), Some(f64::NEG_INFINITY)] {
        let chart = chart_with_fault(fault);
        let kind = chart.kind();
        let error = assert_rejected_before_begin(&chart, kind);
        assert!(error.contains(field), "{kind:?}: {error}");
        assert!(error.contains("source row 1"), "{kind:?}: {error}");
    }
}

fn assert_category_null_rejected<K: crate::kind::Marker>(column: &str, chart: &Plot<K>) {
    let kind = chart.kind();
    let error = assert_rejected_before_begin(chart, kind);
    assert!(error.contains(column), "{kind:?}: {error}");
    assert!(error.contains("null"), "{kind:?}: {error}");
    assert!(error.contains("source row 1"), "{kind:?}: {error}");
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "each family constructs its malformed native channel explicitly"
)]
fn strict_numeric_families_reject_null_nan_and_infinities_before_begin() {
    assert_strict_numeric_faults("x", |fault| {
        let mut x = [1.0, 2.0, 3.0, 4.0].map(Some);
        x[1] = fault;
        Plot::histogram(crate::HistogramInput::samples(NumericValues::nullable(
            "x", x,
        )))
    });
    assert_strict_numeric_faults("x", |fault| {
        let mut x = [1.0, 2.0, 3.0, 4.0].map(Some);
        x[1] = fault;
        Plot::density(NumericValues::nullable("x", x))
    });
    assert_strict_numeric_faults("x", |fault| {
        let mut x = [1.0, 2.0, 3.0, 4.0].map(Some);
        x[1] = fault;
        Plot::qq(NumericValues::nullable("x", x))
    });
    assert_strict_numeric_faults("lower", |fault| {
        let mut lower = [0.5, 1.0, 2.0, 3.0].map(Some);
        lower[1] = fault;
        Plot::point_interval(
            CategoryValues::new("interval_group", ["a", "b", "c", "d"]).unwrap(),
            NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
            crate::Bounds::new(
                NumericValues::nullable("lower", lower),
                NumericValues::new("upper", [1.5, 3.0, 4.0, 5.0]),
            ),
        )
    });
    assert_strict_numeric_faults("y", |fault| {
        let mut y = [1.0, 3.0, 2.0, 5.0].map(Some);
        y[1] = fault;
        Plot::line(
            NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
            NumericValues::nullable("y", y),
        )
    });
    assert_strict_numeric_faults("upper", |fault| {
        let group = CategoryValues::new("group", ["a", "a", "b", "b"]).unwrap();
        let hue = CategoryValues::new("hue", ["h1", "h2", "h1", "h2"]).unwrap();
        let mut upper = [1.5, 3.0, 4.0, 5.0].map(Some);
        upper[1] = fault;
        Plot::interaction(
            group,
            NumericValues::new("y", [1.0, 3.0, 2.0, 5.0]),
            hue.clone(),
            crate::Bounds::new(
                NumericValues::new("lower", [0.5, 1.0, 2.0, 3.0]),
                NumericValues::nullable("upper", upper),
            ),
        )
        .hue(hue)
    });
    assert_strict_numeric_faults("x", |fault| {
        let mut x = [1.0, 2.0, 3.0, 4.0].map(Some);
        x[1] = fault;
        Plot::coefficient(
            CategoryValues::new("interval_group", ["a", "b", "c", "d"]).unwrap(),
            crate::CoefficientInput::estimates_with_intervals(
                NumericValues::nullable("x", x),
                crate::Bounds::new(
                    NumericValues::new("lower", [0.5, 1.0, 2.0, 3.0]),
                    NumericValues::new("upper", [1.5, 3.0, 4.0, 5.0]),
                ),
            ),
        )
    });
    assert_strict_numeric_faults("lower", |fault| {
        let mut lower = [0.5, 1.0, 2.0, 3.0].map(Some);
        lower[1] = fault;
        Plot::forest(
            CategoryValues::new("interval_group", ["a", "b", "c", "d"]).unwrap(),
            NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
            crate::Bounds::new(
                NumericValues::nullable("lower", lower),
                NumericValues::new("upper", [1.5, 3.0, 4.0, 5.0]),
            ),
            0.0,
        )
    });
    assert_strict_numeric_faults("y", |fault| {
        let mut y = [1.0, 3.0, 2.0, 5.0].map(Some);
        y[1] = fault;
        Plot::bar(
            CategoryValues::new("group", ["a", "a", "b", "b"]).unwrap(),
            crate::BarInput::samples(NumericValues::nullable("y", y)),
        )
    });
    assert_strict_numeric_faults("y", |fault| {
        let mut y = [1.0, 3.0, 2.0, 5.0].map(Some);
        y[1] = fault;
        Plot::boxplot(
            CategoryValues::new("group", ["a", "a", "b", "b"]).unwrap(),
            NumericValues::nullable("y", y),
        )
    });
    assert_strict_numeric_faults("x", |fault| {
        let mut x = [1.0, 2.0, 3.0, 4.0].map(Some);
        x[1] = fault;
        Plot::matrix(vec![
            NumericValues::nullable("x", x),
            NumericValues::new("y", [1.0, 3.0, 2.0, 5.0]),
        ])
    });
    assert_strict_numeric_faults("color", |fault| {
        let mut color = [0.1, 0.3, 0.7, 1.0].map(Some);
        color[1] = fault;
        Plot::beeswarm(
            CategoryValues::new("group", ["a", "a", "b", "b"]).unwrap(),
            NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
            NumericValues::nullable("color", color),
        )
    });
    assert_strict_numeric_faults("delta", |fault| {
        let mut delta = [1.0, -1.0, 2.0, 0.0].map(Some);
        delta[1] = fault;
        Plot::waterfall(
            CategoryValues::new("group", ["a", "a", "b", "b"]).unwrap(),
            NumericValues::nullable("delta", delta),
            10.0,
        )
    });
}

#[test]
fn aligned_drop_families_drop_null_or_nan_pairs_and_reject_infinity() {
    assert_aligned_numeric_faults("x", |fault| {
        let mut x = [1.0, 2.0, 3.0, 4.0].map(Some);
        x[1] = fault;
        Plot::scatter(
            NumericValues::nullable("x", x),
            NumericValues::new("y", [1.0, 3.0, 2.0, 5.0]),
        )
    });
    assert_aligned_numeric_faults("y", |fault| {
        let mut y = [1.0, 3.0, 2.0, 5.0].map(Some);
        y[1] = fault;
        Plot::scatter(
            NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
            NumericValues::nullable("y", y),
        )
    });
    assert_aligned_numeric_faults("x", |fault| {
        let mut x = [1.0, 2.0, 3.0, 4.0].map(Some);
        x[1] = fault;
        Plot::regression(
            NumericValues::nullable("x", x),
            NumericValues::new("y", [1.0, 3.0, 2.0, 5.0]),
        )
    });
    assert_aligned_numeric_faults("y", |fault| {
        let mut y = [1.0, 3.0, 2.0, 5.0].map(Some);
        y[1] = fault;
        Plot::regression(
            NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
            NumericValues::nullable("y", y),
        )
    });
    assert_aligned_numeric_faults("x", |fault| {
        let mut x = [1.0, 2.0, 3.0, 4.0].map(Some);
        x[1] = fault;
        Plot::marginal(
            NumericValues::nullable("x", x),
            NumericValues::new("y", [1.0, 3.0, 2.0, 5.0]),
        )
    });
    assert_aligned_numeric_faults("y", |fault| {
        let mut y = [1.0, 3.0, 2.0, 5.0].map(Some);
        y[1] = fault;
        Plot::marginal(
            NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
            NumericValues::nullable("y", y),
        )
    });
}

#[test]
fn two_point_regression_requires_hidden_confidence_band() {
    let default = Plot::regression(
        NumericValues::new("x", [1.0, 2.0]),
        NumericValues::new("y", [3.0, 5.0]),
    );
    let mut backend = RecordingBackend::default();
    let error = default
        .render_kind(PlotKind::Regression, &mut backend)
        .unwrap_err();
    assert!(error.to_string().contains("at least 3 points"));
    assert_eq!(backend.begins, 0);

    let hidden = Plot::regression(
        NumericValues::new("x", [1.0, 2.0]),
        NumericValues::new("y", [3.0, 5.0]),
    )
    .band(crate::RegressionBand::Hidden);
    let mut backend = RecordingBackend::default();
    hidden
        .render_kind(PlotKind::Regression, &mut backend)
        .unwrap();
    assert_eq!((backend.begins, backend.ends), (1, 1));
}

#[test]
fn required_categorical_channels_reject_null_with_source_row() {
    let point_interval = Plot::point_interval(
        CategoryValues::nullable("interval_group", [Some("a"), None, Some("c"), Some("d")])
            .unwrap(),
        NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
        crate::Bounds::new(
            NumericValues::new("lower", [0.5, 1.0, 2.0, 3.0]),
            NumericValues::new("upper", [1.5, 3.0, 4.0, 5.0]),
        ),
    );
    assert_category_null_rejected("interval_group", &point_interval);

    let hue = CategoryValues::new("hue", ["h1", "h2", "h1", "h2"]).unwrap();
    let interaction = Plot::interaction(
        CategoryValues::nullable("group", [Some("a"), None, Some("b"), Some("b")]).unwrap(),
        NumericValues::new("y", [1.0, 3.0, 2.0, 5.0]),
        hue.clone(),
        crate::Bounds::new(
            NumericValues::new("lower", [0.5, 1.0, 2.0, 3.0]),
            NumericValues::new("upper", [1.5, 3.0, 4.0, 5.0]),
        ),
    )
    .hue(hue);
    assert_category_null_rejected("group", &interaction);

    let heatmap = Plot::heatmap(
        CategoryValues::nullable("interval_group", [Some("a"), None, Some("c"), Some("d")])
            .unwrap(),
        CategoryValues::new("group", ["a", "a", "b", "b"]).unwrap(),
        NumericValues::new("color", [0.1, 0.3, 0.7, 1.0]),
    );
    assert_category_null_rejected("interval_group", &heatmap);

    let bar = Plot::bar(
        CategoryValues::nullable("group", [Some("a"), None, Some("b"), Some("b")]).unwrap(),
        crate::BarInput::samples(NumericValues::new("y", [1.0, 3.0, 2.0, 5.0])),
    );
    assert_category_null_rejected("group", &bar);

    let stacked_bar = Plot::stacked_bar(
        CategoryValues::new("group", ["a", "a", "b", "b"]).unwrap(),
        NumericValues::new("y", [1.0, 3.0, 2.0, 5.0]),
        CategoryValues::nullable("hue", [Some("h1"), None, Some("h1"), Some("h2")]).unwrap(),
    );
    assert_category_null_rejected("hue", &stacked_bar);

    let boxplot = Plot::boxplot(
        CategoryValues::nullable("group", [Some("a"), None, Some("b"), Some("b")]).unwrap(),
        NumericValues::new("y", [1.0, 3.0, 2.0, 5.0]),
    );
    assert_category_null_rejected("group", &boxplot);

    let beeswarm = Plot::beeswarm(
        CategoryValues::nullable("group", [Some("a"), None, Some("b"), Some("b")]).unwrap(),
        NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
        NumericValues::new("color", [0.1, 0.3, 0.7, 1.0]),
    );
    assert_category_null_rejected("group", &beeswarm);

    let waterfall = Plot::waterfall(
        CategoryValues::nullable("group", [Some("a"), None, Some("b"), Some("b")]).unwrap(),
        NumericValues::new("delta", [1.0, -1.0, 2.0, 0.0]),
        10.0,
    );
    assert_category_null_rejected("group", &waterfall);
}

#[test]
fn heatmap_preserves_null_cells_but_rejects_nonfinite_values() {
    let null_chart = Plot::heatmap(
        CategoryValues::new("interval_group", ["a", "b", "c", "d"]).unwrap(),
        CategoryValues::new("group", ["a", "a", "b", "b"]).unwrap(),
        NumericValues::nullable("color", [Some(0.1), None, Some(0.7), Some(1.0)]),
    );
    null_chart
        .render_kind(PlotKind::Heatmap, &mut RecordingBackend::default())
        .unwrap();

    for fault in [Some(f64::NAN), Some(f64::INFINITY), Some(f64::NEG_INFINITY)] {
        let chart = Plot::heatmap(
            CategoryValues::new("interval_group", ["a", "b", "c", "d"]).unwrap(),
            CategoryValues::new("group", ["a", "a", "b", "b"]).unwrap(),
            NumericValues::nullable("color", [Some(0.1), fault, Some(0.7), Some(1.0)]),
        );
        let error = assert_rejected_before_begin(&chart, PlotKind::Heatmap);
        assert!(error.contains("color") && error.contains("source row 1"));
    }
}

#[test]
fn structural_triangle_rejects_source_cells_outside_active_half() {
    let chart = Plot::heatmap(
        CategoryValues::new("x", ["b", "a"]).unwrap(),
        CategoryValues::new("y", ["a", "b"]).unwrap(),
        NumericValues::new("color", [1.0, 2.0]),
    )
    .layout(HeatmapLayout::LowerTriangle { diagonal: false });

    let error = assert_rejected_before_begin(&chart, PlotKind::Heatmap);
    assert!(error.contains("outside the active"));
}

#[test]
fn source_rows_drive_standalone_facet_and_canvas_diagnostics() {
    let source_rows = SourceRows::new([4]).unwrap();
    let chart = Plot::scatter(
        NumericValues::new("x", [1.0]),
        NumericValues::new("y", [f64::INFINITY]),
    )
    .source_rows(source_rows.clone());
    assert!(assert_rejected_before_begin(&chart, PlotKind::Scatter).contains("source row 4"));

    let faceted = chart.clone().facet(crate::FacetSpec::row(
        CategoryValues::new("facet", ["a"]).unwrap(),
    ));
    assert!(assert_rejected_before_begin(&faceted, PlotKind::Scatter).contains("source row 4"));

    let canvas = crate::Canvas::grid(1, 1)
        .unwrap()
        .panel(0, 0, chart)
        .unwrap();
    let error = canvas
        .render_with_backend(&mut RecordingBackend::default())
        .unwrap_err();
    assert!(error.to_string().contains("source row 4"));
}

#[test]
fn plot_source_rows_drive_raw_overlay_diagnostics() {
    let chart = Plot::boxplot(
        CategoryValues::new("category", ["a"]).unwrap(),
        NumericValues::new("value", [f64::INFINITY]),
    )
    .source_rows(SourceRows::new([9]).unwrap())
    .raw_overlay(RawOverlay::Points);

    let error = assert_rejected_before_begin(&chart, PlotKind::Boxplot);
    assert!(error.contains("source row 9"));
}
