use std::sync::atomic::Ordering;

use polars::prelude::NamedFrom;

use super::{
    configured_chart, fixture, test_directory, BackendFailure, ComparisonAnnotation, DataFrame,
    Plot, PlotKind, RecordingBackend, Series, ValueSort, RENDER_PANEL_CALLS, TEMP_FILE_COUNTER,
};

#[test]
fn impossible_wrapped_caption_fails_before_backend_begin() {
    let data = fixture();
    let chart = configured_chart(&data, PlotKind::Scatter)
        .caption("diagnostic provenance and model adjustment details ".repeat(20))
        .size(160, 120);
    let mut backend = RecordingBackend::default();

    let error = chart.render_with_backend(&mut backend).unwrap_err();

    assert!(error.to_string().contains("measured figure text"));
    assert_eq!(backend.begins, 0);
}

#[test]
fn invalid_chart_preflights_before_output_path_access() {
    let chart = Plot::scatter(
        crate::NumericValues::new("x", [1.0, 2.0]),
        crate::NumericValues::new("y", [1.0]),
    );
    let parent = std::env::temp_dir().join(format!(
        "lichtung-invalid-preflight-{}-{}",
        std::process::id(),
        TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let output = parent.join("missing").join("chart.svg");

    let error = chart.svg(&output).err().unwrap();

    assert!(error
        .to_string()
        .contains("plot value \"y\" has length 1, expected 2"));
    assert!(!parent.exists());
}

#[test]
fn orchestration_begins_and_ends_once_and_projects_into_offset_panel() {
    let data = fixture();
    let chart = configured_chart(&data, PlotKind::Scatter);
    let expected_panel = chart.figure_layout().panel_box;
    let mut backend = RecordingBackend::default();

    chart.render_kind(PlotKind::Scatter, &mut backend).unwrap();

    assert_eq!((backend.begins, backend.ends), (1, 1));
    assert_eq!(
        (backend.clips.as_slice(), backend.clip_ends),
        (&[expected_panel][..], 1)
    );
    assert!(backend.scatter_points.iter().all(|point| {
        point.x >= expected_panel.x0
            && point.x <= expected_panel.x1
            && point.y >= expected_panel.y0
            && point.y <= expected_panel.y1
    }));
}

#[test]
fn orchestration_runs_measurement_planning_and_render_passes() {
    let data = fixture();
    let chart = configured_chart(&data, PlotKind::Scatter);
    RENDER_PANEL_CALLS.with(|calls| calls.set(0));
    chart
        .render_kind(PlotKind::Scatter, &mut RecordingBackend::default())
        .unwrap();
    let chart_calls = RENDER_PANEL_CALLS.with(std::cell::Cell::get);

    let facet = Series::new("facet".into(), ["left", "left", "right", "right"]);
    let mut faceted_data = data.clone();
    faceted_data.with_column(facet.into()).unwrap();
    let faceted = configured_chart(&faceted_data, PlotKind::Scatter)
        .facet_col(crate::test_support::category(&faceted_data, "facet"));
    RENDER_PANEL_CALLS.with(|calls| calls.set(0));
    faceted
        .render_kind(PlotKind::Scatter, &mut RecordingBackend::default())
        .unwrap();
    let facet_calls = RENDER_PANEL_CALLS.with(std::cell::Cell::get);

    let panel = configured_chart(&data, PlotKind::Scatter);
    let figure = crate::Canvas::grid(1, 1)
        .unwrap()
        .panel(0, 0, panel)
        .unwrap();
    RENDER_PANEL_CALLS.with(|calls| calls.set(0));
    figure
        .render_with_backend(&mut RecordingBackend::default())
        .unwrap();
    let figure_calls = RENDER_PANEL_CALLS.with(std::cell::Cell::get);

    let preflight_panel = configured_chart(&data, PlotKind::Scatter);
    let preflight_figure = crate::Canvas::grid(1, 1)
        .unwrap()
        .panel(0, 0, preflight_panel)
        .unwrap();
    RENDER_PANEL_CALLS.with(|calls| calls.set(0));
    preflight_figure.preflight().unwrap();
    let preflight_calls = RENDER_PANEL_CALLS.with(std::cell::Cell::get);

    let directory = test_directory("figure-single-preflight");
    let output = directory.join("figure.svg");
    let rendered_panel = configured_chart(&data, PlotKind::Scatter);
    let rendered_figure = crate::Canvas::grid(1, 1)
        .unwrap()
        .panel(0, 0, rendered_panel)
        .unwrap();
    RENDER_PANEL_CALLS.with(|calls| calls.set(0));
    rendered_figure.svg(&output).unwrap();
    let rendered_figure_calls = RENDER_PANEL_CALLS.with(std::cell::Cell::get);
    std::fs::remove_dir_all(directory).unwrap();

    assert_eq!(
        (
            chart_calls,
            facet_calls,
            figure_calls,
            preflight_calls,
            rendered_figure_calls
        ),
        (2, 8, 3, 2, 3)
    );
}

#[test]
fn every_canonical_plot_kind_renders_through_one_lifecycle() {
    let data = fixture();
    for &kind in PlotKind::ALL {
        let chart = configured_chart(&data, kind);
        let mut backend = RecordingBackend::default();

        chart
            .render_kind(kind, &mut backend)
            .unwrap_or_else(|error| panic!("{kind:?} failed: {error}"));
        assert_eq!((backend.begins, backend.ends), (1, 1), "{kind:?}");
    }
}

#[test]
fn heatmap_validation_failure_happens_before_canvas_begin() {
    let data = fixture();
    let chart = configured_chart(&data, PlotKind::Heatmap).value_sort(ValueSort::Ascending);
    let mut backend = RecordingBackend::default();

    let error = chart
        .render_kind(PlotKind::Heatmap, &mut backend)
        .unwrap_err()
        .to_string();

    assert!(error.contains("ValueSort is not supported"));
    assert_eq!((backend.begins, backend.ends), (0, 0));
}

#[test]
fn unsupported_significance_fails_before_canvas_begin() {
    let data = fixture();
    let chart = configured_chart(&data, PlotKind::Scatter)
        .add_comparisons([ComparisonAnnotation::new("a", "b", 0.01)]);
    let mut backend = RecordingBackend::default();

    assert!(chart.render_kind(PlotKind::Scatter, &mut backend).is_err());
    assert_eq!((backend.begins, backend.ends), (0, 0));
}

#[test]
fn orchestration_does_not_end_a_canvas_that_failed_to_begin() {
    let data = fixture();
    let chart = configured_chart(&data, PlotKind::Scatter);
    let mut backend = RecordingBackend {
        failures: vec![BackendFailure::Begin],
        ..RecordingBackend::default()
    };

    assert!(chart.render_kind(PlotKind::Scatter, &mut backend).is_err());
    assert_eq!((backend.begins, backend.ends), (1, 0));
}

#[test]
fn orchestration_rejects_every_invalid_axis_endpoint_before_begin() {
    let data = fixture();
    let invalid_ranges = [
        (f64::NAN, 1.0),
        (0.0, f64::NAN),
        (f64::NEG_INFINITY, 1.0),
        (0.0, f64::INFINITY),
        (f64::INFINITY, 1.0),
        (0.0, f64::NEG_INFINITY),
        (1.0, 1.0),
        (2.0, 1.0),
    ];
    for limits in invalid_ranges {
        for x_axis in [true, false] {
            let mut chart = configured_chart(&data, PlotKind::Scatter);
            chart = if x_axis {
                chart.x_limits(limits.0, limits.1)
            } else {
                chart.y_limits(limits.0, limits.1)
            };
            let mut backend = RecordingBackend::default();

            assert!(chart.render_with_backend(&mut backend).is_err());
            assert_eq!(backend.begins, 0, "limits={limits:?}, x_axis={x_axis}");
        }
    }
}

#[test]
fn orchestration_renders_valid_adjacent_subnormal_limits() {
    let upper = f64::from_bits(1);
    let x = Series::new("x".into(), [0.0, upper]);
    let y = Series::new("y".into(), [0.0, upper]);
    let data = DataFrame::new(2, vec![x.into(), y.into()]).unwrap();
    let chart = Plot::scatter(
        crate::test_support::numeric(&data, "x"),
        crate::test_support::numeric(&data, "y"),
    )
    .x_limits(0.0, upper)
    .y_limits(0.0, upper);
    let mut backend = RecordingBackend::default();

    chart.render_with_backend(&mut backend).unwrap();

    assert_eq!((backend.begins, backend.ends), (1, 1));
    assert!(backend
        .scatter_points
        .iter()
        .all(|point| point.x.is_finite() && point.y.is_finite()));
}

#[test]
fn significance_annotations_are_drawn_after_marks_clip_closes() {
    let data = fixture();
    let chart = configured_chart(&data, PlotKind::Boxplot)
        .add_comparisons([ComparisonAnnotation::new("a", "b", 0.01)]);
    let mut backend = RecordingBackend::default();

    chart.render_kind(PlotKind::Boxplot, &mut backend).unwrap();

    assert_eq!(backend.bracket_clip_depths, [0]);
}

#[test]
fn unclamped_projection_leaves_out_of_range_marks_for_backend_clipping() {
    let x = Series::new("x".into(), [-1.0_f64, 0.5, 2.0]);
    let y = Series::new("y".into(), [0.5_f64, 0.5, 0.5]);
    let data = DataFrame::new(x.len(), vec![x.into(), y.into()]).unwrap();
    let chart = Plot::scatter(
        crate::test_support::numeric(&data, "x"),
        crate::test_support::numeric(&data, "y"),
    )
    .x_limits(0.0, 1.0)
    .y_limits(0.0, 1.0);
    let mut backend = RecordingBackend::default();

    chart.render_kind(PlotKind::Scatter, &mut backend).unwrap();

    let panel = backend.clips[0];
    assert!(backend.scatter_points[0].x < panel.x0);
    assert!(backend.scatter_points[1].x > panel.x0 && backend.scatter_points[1].x < panel.x1);
    assert!(backend.scatter_points[2].x > panel.x1);

    let dir = test_directory("out-of-range-svg");
    let output = dir.join("figure.svg");
    chart.clone().svg(&output).unwrap();
    let svg = std::fs::read_to_string(&output).unwrap();
    assert!(svg.contains(&format!("cx=\"{:.3}\"", backend.scatter_points[0].x)));
    assert!(svg.contains(&format!("cx=\"{:.3}\"", backend.scatter_points[2].x)));
    assert!(svg.contains("<g clip-path=\"url(#clip-0)\">"));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn scope_and_canvas_cleanup_errors_preserve_primary_context() {
    let data = fixture();
    let chart = configured_chart(&data, PlotKind::Scatter);
    let mut backend = RecordingBackend {
        failures: vec![
            BackendFailure::Scatter,
            BackendFailure::ClipEnd,
            BackendFailure::End,
        ],
        ..RecordingBackend::default()
    };

    let error = chart
        .render_kind(PlotKind::Scatter, &mut backend)
        .unwrap_err()
        .to_string();

    assert!(error.contains("scatter draw failed"));
    assert!(error.contains("clip cleanup failed"));
    assert!(error.contains("canvas cleanup failed"));
    assert_eq!((backend.clip_ends, backend.ends), (1, 1));
}

#[test]
fn decoration_layer_closes_after_inner_draw_error() {
    let data = fixture();
    let chart = configured_chart(&data, PlotKind::Scatter);
    let mut backend = RecordingBackend {
        failures: vec![BackendFailure::Line, BackendFailure::LayerEnd],
        ..RecordingBackend::default()
    };

    let error = chart
        .render_kind(PlotKind::Scatter, &mut backend)
        .unwrap_err()
        .to_string();

    assert!(error.contains("line draw failed"));
    assert!(error.contains("layer cleanup failed"));
    assert_eq!(backend.layer_ends, 1);
    assert_eq!(backend.ends, 1);
}

#[test]
fn matrix_legend_layer_closes_after_inner_draw_error() {
    let data = fixture();
    let chart = configured_chart(&data, PlotKind::Matrix)
        .group(crate::test_support::category(&data, "hue"))
        .hue(crate::test_support::category(&data, "hue"));
    let mut backend = RecordingBackend {
        failures: vec![BackendFailure::PolygonInLayer, BackendFailure::LayerEnd],
        ..RecordingBackend::default()
    };

    let error = chart
        .render_kind(PlotKind::Matrix, &mut backend)
        .unwrap_err()
        .to_string();

    assert!(error.contains("layer polygon draw failed"));
    assert!(error.contains("layer cleanup failed"));
    assert_eq!(backend.layer_ends, 1);
    assert_eq!(backend.ends, 1);
}
