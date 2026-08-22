use super::{
    assert_rejected_before_begin, categorical_fault_fixture, configured_chart, fixture,
    numeric_fault_fixture, DataFrame, HeatmapLayout, Plot, PlotKind, RecordingBackend, Series,
};
use crate::{RawOverlay, SourceRows};

#[test]
fn strict_numeric_families_reject_null_nan_and_infinities_before_begin() {
    let cases = [
        (PlotKind::Histogram, "x"),
        (PlotKind::Density, "x"),
        (PlotKind::Qq, "x"),
        (PlotKind::PointInterval, "lower"),
        (PlotKind::Line, "y"),
        (PlotKind::Interaction, "upper"),
        (PlotKind::Coefficient, "x"),
        (PlotKind::Forest, "lower"),
        (PlotKind::Bar, "y"),
        (PlotKind::Boxplot, "y"),
        (PlotKind::Matrix, "x"),
        (PlotKind::Beeswarm, "color"),
        (PlotKind::Waterfall, "delta"),
    ];
    for (kind, column) in cases {
        for fault in [
            None,
            Some(f64::NAN),
            Some(f64::INFINITY),
            Some(f64::NEG_INFINITY),
        ] {
            let data = numeric_fault_fixture(column, fault);
            let chart = configured_chart(&data, kind);
            let error = assert_rejected_before_begin(&chart, kind);
            assert!(error.contains(column), "{kind:?}: {error}");
            assert!(error.contains("source row 1"), "{kind:?}: {error}");
        }
    }
}

#[test]
fn aligned_drop_families_drop_null_or_nan_pairs_and_reject_infinity() {
    for kind in [PlotKind::Scatter, PlotKind::Regression, PlotKind::Marginal] {
        for column in ["x", "y"] {
            for fault in [None, Some(f64::NAN)] {
                let data = numeric_fault_fixture(column, fault);
                let chart = configured_chart(&data, kind);
                let mut backend = RecordingBackend::default();
                chart.render_kind(kind, &mut backend).unwrap();
                assert_eq!((backend.begins, backend.ends), (1, 1), "{kind:?}");
            }
            for fault in [Some(f64::INFINITY), Some(f64::NEG_INFINITY)] {
                let data = numeric_fault_fixture(column, fault);
                let chart = configured_chart(&data, kind);
                let error = assert_rejected_before_begin(&chart, kind);
                assert!(error.contains(column), "{kind:?}: {error}");
                assert!(error.contains("source row 1"), "{kind:?}: {error}");
            }
        }
    }
}

#[test]
fn two_point_regression_requires_hidden_confidence_band() {
    let data = DataFrame::new(
        2,
        vec![
            Series::new("x".into(), [1.0_f64, 2.0]).into(),
            Series::new("y".into(), [3.0_f64, 5.0]).into(),
        ],
    )
    .unwrap();
    let default = Plot::regression(
        crate::test_support::numeric(&data, "x"),
        crate::test_support::numeric(&data, "y"),
    );
    let mut backend = RecordingBackend::default();
    let error = default
        .render_kind(PlotKind::Regression, &mut backend)
        .unwrap_err();
    assert!(error.to_string().contains("at least 3 points"));
    assert_eq!(backend.begins, 0);

    let hidden = Plot::regression(
        crate::test_support::numeric(&data, "x"),
        crate::test_support::numeric(&data, "y"),
    )
    .regression_band(crate::RegressionBand::Hidden);
    let mut backend = RecordingBackend::default();
    hidden
        .render_kind(PlotKind::Regression, &mut backend)
        .unwrap();
    assert_eq!((backend.begins, backend.ends), (1, 1));
}

#[test]
fn required_categorical_channels_reject_null_with_source_row() {
    for (kind, column) in [
        (PlotKind::PointInterval, "interval_group"),
        (PlotKind::Interaction, "group"),
        (PlotKind::Heatmap, "interval_group"),
        (PlotKind::Bar, "group"),
        (PlotKind::StackedBar, "hue"),
        (PlotKind::Boxplot, "group"),
        (PlotKind::Beeswarm, "group"),
        (PlotKind::Waterfall, "group"),
    ] {
        let data = categorical_fault_fixture(column);
        let chart = configured_chart(&data, kind);
        let error = assert_rejected_before_begin(&chart, kind);
        assert!(error.contains(column), "{kind:?}: {error}");
        assert!(error.contains("null"), "{kind:?}: {error}");
        assert!(error.contains("source row 1"), "{kind:?}: {error}");
    }
}

#[test]
fn heatmap_preserves_null_cells_but_rejects_nonfinite_values() {
    let null_data = numeric_fault_fixture("color", None);
    configured_chart(&null_data, PlotKind::Heatmap)
        .render_kind(PlotKind::Heatmap, &mut RecordingBackend::default())
        .unwrap();

    for fault in [Some(f64::NAN), Some(f64::INFINITY), Some(f64::NEG_INFINITY)] {
        let data = numeric_fault_fixture("color", fault);
        let chart = configured_chart(&data, PlotKind::Heatmap);
        let error = assert_rejected_before_begin(&chart, PlotKind::Heatmap);
        assert!(error.contains("color") && error.contains("source row 1"));
    }
}

#[test]
fn structural_triangle_rejects_source_cells_outside_active_half() {
    let data = DataFrame::new(
        2,
        vec![
            Series::new("x".into(), ["b", "a"]).into(),
            Series::new("y".into(), ["a", "b"]).into(),
            Series::new("color".into(), [1.0_f64, 2.0]).into(),
        ],
    )
    .unwrap();
    let chart = Plot::heatmap(
        crate::test_support::category(&data, "x"),
        crate::test_support::category(&data, "y"),
        crate::test_support::numeric(&data, "color"),
    )
    .heatmap_layout(HeatmapLayout::LowerTriangle { diagonal: false });

    let error = assert_rejected_before_begin(&chart, PlotKind::Heatmap);
    assert!(error.contains("outside the active"));
}

#[test]
fn non_heatmap_kinds_reject_triangular_layout() {
    let data = fixture();
    for &kind in PlotKind::ALL {
        if kind == PlotKind::Heatmap {
            continue;
        }
        let chart = configured_chart(&data, kind)
            .heatmap_layout(HeatmapLayout::UpperTriangle { diagonal: true });
        let error = assert_rejected_before_begin(&chart, kind);
        assert!(error.contains("HeatmapLayout"), "{kind:?}: {error}");
    }
}

#[test]
fn unsupported_channels_fail_before_canvas_begin() {
    let data = fixture();
    let chart = configured_chart(&data, PlotKind::Regression)
        .hue(crate::test_support::category(&data, "hue"));
    let figure = crate::Canvas::grid(1, 1)
        .unwrap()
        .panel(0, 0, chart)
        .unwrap();
    let mut backend = RecordingBackend::default();

    let error = figure.render_with_backend(&mut backend).unwrap_err();
    assert!(error.to_string().contains("regression-plot: hue column"));
    assert_eq!((backend.begins, backend.ends), (0, 0));
}

#[test]
fn source_rows_drive_standalone_facet_and_canvas_diagnostics() {
    let data = DataFrame::new(
        1,
        vec![
            Series::new("x".into(), [1.0]).into(),
            Series::new("y".into(), [f64::INFINITY]).into(),
            Series::new("facet".into(), ["a"]).into(),
        ],
    )
    .unwrap();
    let source_rows = SourceRows::new([4]).unwrap();
    let chart = Plot::scatter(
        crate::test_support::numeric(&data, "x"),
        crate::test_support::numeric(&data, "y"),
    )
    .source_rows(source_rows.clone());
    assert!(assert_rejected_before_begin(&chart, PlotKind::Scatter).contains("source row 4"));

    let faceted = chart
        .clone()
        .facet_row(crate::test_support::category(&data, "facet"));
    assert!(assert_rejected_before_begin(&faceted, PlotKind::Scatter).contains("source row 4"));

    let figure = crate::Canvas::grid(1, 1)
        .unwrap()
        .panel(0, 0, chart)
        .unwrap();
    let error = figure
        .render_with_backend(&mut RecordingBackend::default())
        .unwrap_err();
    assert!(error.to_string().contains("source row 4"));
}

#[test]
fn raw_overlay_source_rows_precede_structural_rows() {
    let data = DataFrame::new(
        1,
        vec![
            Series::new("category".into(), ["a"]).into(),
            Series::new("value".into(), [f64::INFINITY]).into(),
            Series::new("overlay_row".into(), [9_u64]).into(),
        ],
    )
    .unwrap();
    let chart = Plot::boxplot(
        crate::test_support::category(&data, "category"),
        crate::test_support::numeric(&data, "value"),
    )
    .source_rows(SourceRows::new([4]).unwrap())
    .raw_overlay(RawOverlay::Points)
    .raw_overlay_source_rows(crate::test_support::source_rows(&data, "overlay_row"));

    let error = assert_rejected_before_begin(&chart, PlotKind::Boxplot);
    assert!(error.contains("source row 9"));
}
