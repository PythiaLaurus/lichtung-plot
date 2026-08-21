use polars::prelude::NamedFrom;

use super::{
    assert_rejected_before_begin, categorical_fault_fixture, configured_chart, fixture,
    numeric_fault_fixture, Chart, DataFrame, HeatmapLayout, PlotKind, RecordingBackend, Series,
};
use crate::RawOverlay;

#[test]
fn every_plot_kind_rejects_missing_required_channels_before_begin() {
    let data = fixture();
    for &kind in PlotKind::ALL {
        let mut chart = Chart::new(&data);
        let _ = chart.select(kind);
        let error = assert_rejected_before_begin(&chart, kind);
        let family = match kind {
            PlotKind::Qq => "Q-Q plot",
            PlotKind::PointInterval => "point_interval",
            PlotKind::Bar
            | PlotKind::HorizontalBar
            | PlotKind::StackedBar
            | PlotKind::StackedHorizontalBar => "bar",
            PlotKind::Matrix => "matrix",
            PlotKind::MassSpectrum => "mass_spectrum",
            _ => kind.key_segment().split('-').next().unwrap(),
        };
        assert!(error.contains(family), "{kind:?}: {error}");
    }
}

#[test]
fn matrix_rejects_invalid_variable_lists_before_begin() {
    let data = fixture();
    for (variables, expected) in [
        (vec!["x"], "at least two"),
        (vec!["x", "  "], "nonempty"),
        (vec!["x", "x"], "duplicate"),
    ] {
        let mut chart = Chart::new(&data);
        let _ = chart.matrix_variables(variables).select(PlotKind::Matrix);
        let error = assert_rejected_before_begin(&chart, PlotKind::Matrix);
        assert!(error.contains(expected), "{error}");
    }
}

#[test]
fn matrix_rejects_parallel_xy_channels_before_begin() {
    let data = fixture();
    let mut chart = Chart::new(&data);
    let _ = chart
        .x("x")
        .y("y")
        .matrix_variables(vec!["x", "y"])
        .select(PlotKind::Matrix);

    let error = assert_rejected_before_begin(&chart, PlotKind::Matrix);

    assert!(error.contains("x and y channels are not supported"));
}

#[test]
fn every_strict_family_rejects_null_nan_and_both_infinities_before_begin() {
    let cases = [
        (PlotKind::Histogram, "x", "histogram"),
        (PlotKind::Density, "x", "density"),
        (PlotKind::Qq, "x", "Q-Q plot"),
        (PlotKind::PointInterval, "x", "point_interval"),
        (PlotKind::PointInterval, "lower", "point_interval"),
        (PlotKind::PointInterval, "upper", "point_interval"),
        (PlotKind::Line, "x", "line"),
        (PlotKind::Line, "y", "line"),
        (PlotKind::Line, "lower", "line"),
        (PlotKind::Line, "upper", "line"),
        (PlotKind::Interaction, "y", "interaction"),
        (PlotKind::Interaction, "lower", "interaction"),
        (PlotKind::Interaction, "upper", "interaction"),
        (PlotKind::Coefficient, "x", "coefficient"),
        (PlotKind::Coefficient, "lower", "coefficient"),
        (PlotKind::Coefficient, "upper", "coefficient"),
        (PlotKind::Forest, "x", "forest"),
        (PlotKind::Forest, "lower", "forest"),
        (PlotKind::Forest, "upper", "forest"),
        (PlotKind::Bar, "y", "bar"),
        (PlotKind::HorizontalBar, "y", "bar"),
        (PlotKind::StackedBar, "y", "bar"),
        (PlotKind::StackedHorizontalBar, "y", "bar"),
        (PlotKind::Boxplot, "y", "boxplot"),
        (PlotKind::Violin, "y", "violin"),
        (PlotKind::Matrix, "x", "matrix"),
        (PlotKind::Matrix, "y", "matrix"),
        (PlotKind::MassSpectrum, "x", "mass_spectrum"),
        (PlotKind::MassSpectrum, "y", "mass_spectrum"),
        (PlotKind::Beeswarm, "x", "beeswarm"),
        (PlotKind::Beeswarm, "color", "beeswarm"),
        (PlotKind::Waterfall, "delta", "waterfall"),
        (PlotKind::Waterfall, "base", "waterfall"),
        (PlotKind::Waterfall, "terminal", "waterfall"),
    ];
    for (kind, column, family) in cases {
        for fault in [
            None,
            Some(f64::NAN),
            Some(f64::INFINITY),
            Some(f64::NEG_INFINITY),
        ] {
            let data = numeric_fault_fixture(column, fault);
            let chart = configured_chart(&data, kind);
            let chart = if kind == PlotKind::Line && matches!(column, "lower" | "upper") {
                chart.interval_bounds("lower", "upper")
            } else {
                chart
            };
            let error = assert_rejected_before_begin(&chart, kind);
            assert!(error.contains(family), "{kind:?}: {error}");
            assert!(error.contains(column), "{kind:?}: {error}");
            assert!(error.contains("source row 1"), "{kind:?}: {error}");
        }
    }
}

#[test]
fn aligned_drop_families_drop_only_null_or_nan_pairs_and_reject_infinity() {
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
fn aligned_drop_families_reject_empty_data_after_joint_drop() {
    let x = Series::new("x".into(), [None::<f64>; 4]);
    let y = Series::new("y".into(), [Some(f64::NAN); 4]);
    let data = DataFrame::new(4, vec![x.into(), y.into()]).unwrap();
    for kind in [PlotKind::Scatter, PlotKind::Regression, PlotKind::Marginal] {
        let mut chart = Chart::new(&data);
        let _ = chart.x("x").y("y");
        let _ = chart.select(kind);
        let error = assert_rejected_before_begin(&chart, kind);
        assert!(
            error.contains("no rows") || error.contains("got 0"),
            "{kind:?}: {error}"
        );
    }
}

#[test]
fn two_point_regression_rejects_default_band_and_accepts_hidden_descriptive_fit() {
    let data = DataFrame::new(
        2,
        vec![
            Series::new("x".into(), [1.0_f64, 2.0]).into(),
            Series::new("y".into(), [3.0_f64, 5.0]).into(),
        ],
    )
    .unwrap();
    let mut default = Chart::new(&data);
    let _ = default.x("x").y("y").select(PlotKind::Regression);
    let mut backend = RecordingBackend::default();
    let error = default
        .render_kind(PlotKind::Regression, &mut backend)
        .unwrap_err();
    assert!(error.to_string().contains("at least 3 points"));
    assert_eq!(backend.begins, 0);

    let mut hidden = Chart::new(&data).regression_band(crate::RegressionBand::Hidden);
    let _ = hidden.x("x").y("y").select(PlotKind::Regression);
    let mut backend = RecordingBackend::default();
    hidden
        .render_kind(PlotKind::Regression, &mut backend)
        .unwrap();
    assert_eq!((backend.begins, backend.ends), (1, 1));
    assert!(backend
        .texts
        .iter()
        .all(|(_, _, text)| !text.contains(crate::RegressionBand::Hidden.provenance())));
}

#[test]
fn every_required_categorical_channel_rejects_null_with_source_row_before_begin() {
    let cases = [
        (PlotKind::Scatter, "hue", true),
        (PlotKind::Density, "hue", true),
        (PlotKind::PointInterval, "interval_group", false),
        (PlotKind::Line, "hue", true),
        (PlotKind::Interaction, "group", false),
        (PlotKind::Interaction, "hue", false),
        (PlotKind::Coefficient, "interval_group", false),
        (PlotKind::Forest, "interval_group", false),
        (PlotKind::Heatmap, "interval_group", false),
        (PlotKind::Heatmap, "group", false),
        (PlotKind::Bar, "group", false),
        (PlotKind::Bar, "hue", true),
        (PlotKind::HorizontalBar, "group", false),
        (PlotKind::HorizontalBar, "hue", true),
        (PlotKind::StackedBar, "group", false),
        (PlotKind::StackedBar, "hue", false),
        (PlotKind::StackedHorizontalBar, "group", false),
        (PlotKind::StackedHorizontalBar, "hue", false),
        (PlotKind::Boxplot, "group", false),
        (PlotKind::Boxplot, "hue", true),
        (PlotKind::Violin, "group", false),
        (PlotKind::Violin, "hue", true),
        (PlotKind::Marginal, "hue", true),
        (PlotKind::Matrix, "hue", true),
        (PlotKind::Beeswarm, "group", false),
        (PlotKind::Waterfall, "group", false),
    ];
    for (kind, column, add_hue) in cases {
        let data = categorical_fault_fixture(column);
        let chart = configured_chart(&data, kind);
        let chart = if add_hue { chart.hue("hue") } else { chart };
        let error = assert_rejected_before_begin(&chart, kind);
        assert!(error.contains(column), "{kind:?}: {error}");
        assert!(error.contains("null"), "{kind:?}: {error}");
        assert!(error.contains("source row 1"), "{kind:?}: {error}");
    }
}

#[test]
fn marginal_rejects_null_hue_on_a_droppable_numeric_row_before_begin() {
    let x = Series::new("x".into(), [Some(1.0_f64), None, Some(3.0)]);
    let y = Series::new("y".into(), [Some(2.0_f64), Some(f64::NAN), Some(4.0)]);
    let hue = Series::new("hue".into(), [Some("a"), None, Some("a")]);
    let data = DataFrame::new(3, vec![x.into(), y.into(), hue.into()]).unwrap();
    let mut chart = Chart::new(&data).hue("hue");
    let _ = chart.x("x").y("y");
    let _ = chart.select(PlotKind::Marginal);

    let error = assert_rejected_before_begin(&chart, PlotKind::Marginal);

    assert!(error.contains("hue") && error.contains("source row 1"));
}

#[test]
fn heatmap_preserves_null_cells_but_rejects_present_nonfinite_values_before_begin() {
    let null_data = numeric_fault_fixture("color", None);
    let null_chart = configured_chart(&null_data, PlotKind::Heatmap);
    let mut backend = RecordingBackend::default();
    null_chart
        .render_kind(PlotKind::Heatmap, &mut backend)
        .unwrap();
    assert_eq!((backend.begins, backend.ends), (1, 1));

    for fault in [Some(f64::NAN), Some(f64::INFINITY), Some(f64::NEG_INFINITY)] {
        let data = numeric_fault_fixture("color", fault);
        let chart = configured_chart(&data, PlotKind::Heatmap);
        let error = assert_rejected_before_begin(&chart, PlotKind::Heatmap);
        assert!(error.contains("color") && error.contains("source row 1"));
    }
}

#[test]
fn heatmap_rejects_structural_triangle_source_cells_before_begin() {
    let x = Series::new("x".into(), ["b", "a"]);
    let y = Series::new("y".into(), ["a", "b"]);
    let color = Series::new("color".into(), [1.0_f64, 2.0]);
    let data = DataFrame::new(2, vec![x.into(), y.into(), color.into()]).unwrap();
    let mut chart = Chart::new(&data)
        .color("color")
        .heatmap_layout(HeatmapLayout::LowerTriangle { diagonal: false });
    let _ = chart.x("x").y("y");

    let error = assert_rejected_before_begin(&chart, PlotKind::Heatmap);

    assert!(error.contains("outside the active"));
}

#[test]
fn every_non_heatmap_kind_rejects_triangular_heatmap_layout_before_begin() {
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
fn explicit_cartesian_heatmap_layout_is_inert_for_non_heatmap_kind() {
    let data = fixture();
    let chart = configured_chart(&data, PlotKind::Scatter).heatmap_layout(HeatmapLayout::Cartesian);
    let mut backend = RecordingBackend::default();

    chart.render_kind(PlotKind::Scatter, &mut backend).unwrap();

    assert_eq!((backend.begins, backend.ends), (1, 1));
}

#[test]
fn every_unsupported_hue_and_color_combination_rejects_before_begin() {
    let data = fixture();
    for &kind in PlotKind::ALL {
        if !matches!(
            kind,
            PlotKind::Scatter
                | PlotKind::Density
                | PlotKind::Line
                | PlotKind::Interaction
                | PlotKind::Bar
                | PlotKind::HorizontalBar
                | PlotKind::StackedBar
                | PlotKind::StackedHorizontalBar
                | PlotKind::Boxplot
                | PlotKind::Violin
                | PlotKind::Marginal
                | PlotKind::Matrix
        ) {
            let chart = configured_chart(&data, kind).hue("hue");
            let error = assert_rejected_before_begin(&chart, kind);
            assert!(error.contains("hue column"), "{kind:?}: {error}");
        }
        if !matches!(kind, PlotKind::Heatmap | PlotKind::Beeswarm) {
            let chart = configured_chart(&data, kind).color("color");
            let error = assert_rejected_before_begin(&chart, kind);
            assert!(
                error.contains("continuous color column"),
                "{kind:?}: {error}"
            );
            let chart = configured_chart(&data, kind).color_label("measurement");
            let error = assert_rejected_before_begin(&chart, kind);
            assert!(error.contains("color label"), "{kind:?}: {error}");
        }
    }
    for kind in [PlotKind::Boxplot, PlotKind::Violin] {
        let chart = configured_chart(&data, kind).hue("group");
        let error = assert_rejected_before_begin(&chart, kind);
        assert!(error.contains("redundant hue channel"), "{kind:?}: {error}");
    }
}

#[test]
fn injected_marginal_coordinates_reject_hue_before_begin() {
    let data = fixture();
    let mut chart = configured_chart(&data, PlotKind::Marginal).hue("hue");
    let _ = chart.add_marginal_coords(vec![(1.0, 2.0), (2.0, 3.0)]);

    let error = assert_rejected_before_begin(&chart, PlotKind::Marginal);

    assert!(error.contains("injected marginal coordinates"));
}

#[test]
fn composed_figure_rejects_unsupported_channels_before_begin() {
    let data = fixture();
    let chart = configured_chart(&data, PlotKind::Regression).hue("hue");
    let figure = crate::Figure::grid(1, 1)
        .unwrap()
        .panel(0, 0, chart)
        .unwrap();
    let mut backend = RecordingBackend::default();

    let error = figure.render_with_backend(&mut backend).unwrap_err();

    assert!(error.to_string().contains("regression-plot: hue column"));
    assert_eq!((backend.begins, backend.ends), (0, 0));
}

#[test]
fn structural_source_rows_drive_validation_diagnostics() {
    let data = DataFrame::new(
        1,
        vec![
            Series::new("x".into(), [1.0]).into(),
            Series::new("y".into(), [f64::INFINITY]).into(),
        ],
    )
    .unwrap();
    let mut chart = Chart::new(&data).source_rows(&[4]);
    let _ = chart.x("x").y("y");

    let error = assert_rejected_before_begin(&chart, PlotKind::Scatter);

    assert!(error.contains("source row 4"));
}

#[test]
fn composed_figure_preserves_structural_source_rows_before_begin() {
    let data = DataFrame::new(
        1,
        vec![
            Series::new("x".into(), [1.0]).into(),
            Series::new("y".into(), [f64::INFINITY]).into(),
        ],
    )
    .unwrap();
    let mut chart = Chart::new(&data).source_rows(&[4]);
    chart.x("x").y("y").scatter();
    let figure = crate::Figure::grid(1, 1)
        .unwrap()
        .panel(0, 0, chart)
        .unwrap();
    let mut backend = RecordingBackend::default();

    let error = figure.render_with_backend(&mut backend).unwrap_err();

    assert!(error.to_string().contains("source row 4"));
    assert_eq!(backend.begins, 0);
}

#[test]
fn faceting_preserves_structural_source_rows_before_begin() {
    let data = DataFrame::new(
        1,
        vec![
            Series::new("x".into(), [1.0]).into(),
            Series::new("y".into(), [f64::INFINITY]).into(),
            Series::new("facet".into(), ["a"]).into(),
        ],
    )
    .unwrap();
    let mut chart = Chart::new(&data).source_rows(&[4]).facet_row("facet");
    let _ = chart.x("x").y("y");
    let mut backend = RecordingBackend::default();

    let error = chart
        .render_kind(PlotKind::Scatter, &mut backend)
        .unwrap_err();

    assert!(error.to_string().contains("source row 4"));
    assert_eq!(backend.begins, 0);
}

#[test]
fn source_row_map_length_is_rejected_before_begin() {
    let data = fixture();
    let chart = configured_chart(&data, PlotKind::Scatter).source_rows(&[4]);
    let mut backend = RecordingBackend::default();

    let error = chart
        .render_kind(PlotKind::Scatter, &mut backend)
        .unwrap_err();

    assert!(error.to_string().contains("source-row map has length 1"));
    assert_eq!(backend.begins, 0);
}

#[test]
fn raw_overlay_source_row_column_precedes_structural_map() {
    let data = DataFrame::new(
        1,
        vec![
            Series::new("category".into(), ["a"]).into(),
            Series::new("value".into(), [f64::INFINITY]).into(),
            Series::new("overlay_row".into(), [9_u64]).into(),
        ],
    )
    .unwrap();
    let mut chart = Chart::new(&data)
        .source_rows(&[4])
        .raw_overlay(RawOverlay::Points)
        .raw_overlay_source_row("overlay_row");
    let _ = chart.x("category").y("value");

    let error = assert_rejected_before_begin(&chart, PlotKind::Boxplot);

    assert!(error.contains("source row 9"));
}

#[test]
fn every_unsupported_auxiliary_channel_rejects_before_begin() {
    let data = fixture();
    for &kind in PlotKind::ALL {
        if !matches!(
            kind,
            PlotKind::PointInterval
                | PlotKind::Line
                | PlotKind::Interaction
                | PlotKind::Coefficient
                | PlotKind::Forest
        ) {
            let chart = configured_chart(&data, kind).interval_bounds("lower", "upper");
            assert_rejected_before_begin(&chart, kind);
        }
        if kind != PlotKind::Waterfall {
            let chart = configured_chart(&data, kind).waterfall_totals("base", "terminal");
            assert_rejected_before_begin(&chart, kind);
        }
        if kind != PlotKind::MassSpectrum {
            let mut chart = configured_chart(&data, kind);
            let _ = chart.add_peaks(vec![(1.0, 2.0)]);
            assert_rejected_before_begin(&chart, kind);
        }
        if kind != PlotKind::Marginal {
            let mut chart = configured_chart(&data, kind);
            let _ = chart.add_marginal_coords(vec![(1.0, 2.0), (2.0, 3.0)]);
            assert_rejected_before_begin(&chart, kind);
        }
        if kind != PlotKind::Matrix {
            let mut chart = configured_chart(&data, kind);
            let _ = chart.matrix_variables(vec!["x", "y"]);
            assert_rejected_before_begin(&chart, kind);
        }
    }
}
