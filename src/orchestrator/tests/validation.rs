use super::{
    assert_rejected_before_begin, categorical_fault_values, configured_chart, numeric_fault_values,
    CategoryField, HeatmapLayout, NumericField, Plot, PlotKind, RecordingBackend,
};
use crate::{CategoryValues, NumericValues, RawOverlay, SourceRows};

#[test]
fn strict_numeric_families_reject_null_nan_and_infinities_before_begin() {
    let cases = [
        (PlotKind::Histogram, NumericField::X),
        (PlotKind::Density, NumericField::X),
        (PlotKind::Qq, NumericField::X),
        (PlotKind::PointInterval, NumericField::Lower),
        (PlotKind::Line, NumericField::Y),
        (PlotKind::Interaction, NumericField::Upper),
        (PlotKind::Coefficient, NumericField::X),
        (PlotKind::Forest, NumericField::Lower),
        (PlotKind::Bar, NumericField::Y),
        (PlotKind::Boxplot, NumericField::Y),
        (PlotKind::Matrix, NumericField::X),
        (PlotKind::Beeswarm, NumericField::Color),
        (PlotKind::Waterfall, NumericField::Delta),
    ];
    for (kind, field) in cases {
        for fault in [
            None,
            Some(f64::NAN),
            Some(f64::INFINITY),
            Some(f64::NEG_INFINITY),
        ] {
            let data = numeric_fault_values(field, fault);
            let chart = configured_chart(&data, kind);
            let error = assert_rejected_before_begin(&chart, kind);
            assert!(error.contains(field.name()), "{kind:?}: {error}");
            assert!(error.contains("source row 1"), "{kind:?}: {error}");
        }
    }
}

#[test]
fn aligned_drop_families_drop_null_or_nan_pairs_and_reject_infinity() {
    for kind in [PlotKind::Scatter, PlotKind::Regression, PlotKind::Marginal] {
        for field in [NumericField::X, NumericField::Y] {
            for fault in [None, Some(f64::NAN)] {
                let data = numeric_fault_values(field, fault);
                let chart = configured_chart(&data, kind);
                let mut backend = RecordingBackend::default();
                chart.render_kind(kind, &mut backend).unwrap();
                assert_eq!((backend.begins, backend.ends), (1, 1), "{kind:?}");
            }
            for fault in [Some(f64::INFINITY), Some(f64::NEG_INFINITY)] {
                let data = numeric_fault_values(field, fault);
                let chart = configured_chart(&data, kind);
                let error = assert_rejected_before_begin(&chart, kind);
                assert!(error.contains(field.name()), "{kind:?}: {error}");
                assert!(error.contains("source row 1"), "{kind:?}: {error}");
            }
        }
    }
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
    for (kind, column) in [
        (PlotKind::PointInterval, CategoryField::IntervalGroup),
        (PlotKind::Interaction, CategoryField::Group),
        (PlotKind::Heatmap, CategoryField::IntervalGroup),
        (PlotKind::Bar, CategoryField::Group),
        (PlotKind::StackedBar, CategoryField::Hue),
        (PlotKind::Boxplot, CategoryField::Group),
        (PlotKind::Beeswarm, CategoryField::Group),
        (PlotKind::Waterfall, CategoryField::Group),
    ] {
        let data = categorical_fault_values(column);
        let chart = configured_chart(&data, kind);
        let error = assert_rejected_before_begin(&chart, kind);
        assert!(error.contains(column.name()), "{kind:?}: {error}");
        assert!(error.contains("null"), "{kind:?}: {error}");
        assert!(error.contains("source row 1"), "{kind:?}: {error}");
    }
}

#[test]
fn heatmap_preserves_null_cells_but_rejects_nonfinite_values() {
    let null_data = numeric_fault_values(NumericField::Color, None);
    configured_chart(&null_data, PlotKind::Heatmap)
        .render_kind(PlotKind::Heatmap, &mut RecordingBackend::default())
        .unwrap();

    for fault in [Some(f64::NAN), Some(f64::INFINITY), Some(f64::NEG_INFINITY)] {
        let data = numeric_fault_values(NumericField::Color, fault);
        let chart = configured_chart(&data, PlotKind::Heatmap);
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
