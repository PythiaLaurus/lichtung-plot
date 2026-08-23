use std::sync::atomic::{AtomicU64, Ordering};

use lichtung_plot::{
    Canvas, CategoryOrder, HeatmapAnnotations, HeatmapLabelFit, HeatmapLayout,
    HeatmapNullRowAnnotation, HeatmapScale, NumericLabelFormat, Plot, ValueSort,
};

#[path = "../support/data.rs"]
mod data;
use data::{df, DataFrame};

static TEST_DIRECTORY_COUNTER: AtomicU64 = AtomicU64::new(0);

fn fixture() -> DataFrame {
    df!(
        "column" => ["left", "right", "left", "right"],
        "row" => ["top", "top", "bottom", "bottom"],
        "value" => [0.0_f64, 1.0, 0.5, 0.75]
    )
    .unwrap()
}

fn chart(data_frame: &DataFrame) -> Plot<lichtung_plot::kind::Heatmap> {
    Plot::heatmap(
        data::category(data_frame, "column"),
        data::category(data_frame, "row"),
        data::numeric(data_frame, "value"),
    )
    .size(560, 360)
}

fn render(
    data: &DataFrame,
    configure: impl FnOnce(Plot<lichtung_plot::kind::Heatmap>) -> Plot<lichtung_plot::kind::Heatmap>,
) -> Result<String, String> {
    let directory = std::env::temp_dir().join(format!(
        "lichtung-heatmap-{}-{}",
        std::process::id(),
        TEST_DIRECTORY_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&directory).unwrap();
    let output = directory.join("plot.svg");
    let plot = configure(chart(data));
    let result = plot
        .svg(&output)
        .map(|_| std::fs::read_to_string(&output).unwrap())
        .map_err(|error| error.to_string());
    std::fs::remove_dir_all(directory).unwrap();
    result
}

#[test]
fn heatmap_constructor_accepts_typed_configuration() {
    let data = df!(
        "column" => ["a", "b", "b"],
        "row" => ["a", "a", "b"],
        "value" => [0.0_f64, 1.0, 0.5]
    )
    .unwrap();
    let svg = render(&data, |plot| {
        plot.color_label("Response")
            .heatmap_layout(HeatmapLayout::UpperTriangle { diagonal: true })
            .heatmap_scale(HeatmapScale::Diverging { center: 0.5 })
            .heatmap_color_limits(-1.0, 2.0)
            .heatmap_annotations(HeatmapAnnotations::Values(
                NumericLabelFormat::new(2).unwrap(),
            ))
    })
    .unwrap();
    assert!(svg.contains(">Response</text>"));
    assert!(svg.contains(">-1</text>") && svg.contains(">2</text>"));
}

#[test]
fn heatmap_cartesian_default_is_byte_identical_to_explicit_cartesian() {
    let data = fixture();

    let default = render(&data, |chart| chart).unwrap();
    let explicit = render(&data, |chart| {
        chart.heatmap_layout(HeatmapLayout::Cartesian)
    })
    .unwrap();

    assert_eq!(default, explicit);
}

#[test]
fn heatmap_hidden_annotations_and_all_false_highlights_are_byte_inert() {
    let data = df!(
        "column" => ["left", "right", "left", "right"],
        "row" => ["top", "top", "bottom", "bottom"],
        "value" => [0.0_f64, 1.0, 0.5, 0.75],
        "selected" => [false, false, false, false]
    )
    .unwrap();

    let default = render(&data, |chart| chart).unwrap();
    let explicit = render(&data, |chart| {
        chart
            .heatmap_annotations(HeatmapAnnotations::Hidden)
            .heatmap_highlight(data::boolean(&data, "selected"))
    })
    .unwrap();

    assert_eq!(default, explicit);
}

#[test]
fn heatmap_lower_triangle_suppresses_structural_cells_without_missing_marks() {
    let data = df!(
        "column" => ["a", "a", "b"],
        "row" => ["b", "c", "c"],
        "value" => [1.0_f64, 2.0, 3.0]
    )
    .unwrap();
    let svg = render(&data, |chart| {
        chart.heatmap_layout(HeatmapLayout::LowerTriangle { diagonal: false })
    })
    .unwrap();
    let marks = clipped_marks(&svg);

    assert_eq!(marks.matches("<polygon ").count(), 3);
    assert_eq!(marks.matches("<polyline ").count(), 0);
    assert!(!svg.contains(">Missing</text>"));
}

#[test]
fn heatmap_triangle_marks_only_missing_cells_inside_active_triangle() {
    let data = df!(
        "column" => ["a", "a"],
        "row" => ["b", "c"],
        "value" => [1.0_f64, 2.0]
    )
    .unwrap();
    let svg = render(&data, |chart| {
        chart.heatmap_layout(HeatmapLayout::LowerTriangle { diagonal: false })
    })
    .unwrap();
    let marks = clipped_marks(&svg);

    assert_eq!(marks.matches("<polygon ").count(), 3);
    assert_eq!(marks.matches("<polyline ").count(), 2);
    assert!(svg.contains(">Missing</text>"));
}

#[test]
fn heatmap_triangle_active_diagonal_uses_missing_semantics() {
    let data = df!(
        "column" => ["a", "a", "b"],
        "row" => ["b", "c", "c"],
        "value" => [1.0_f64, 2.0, 3.0]
    )
    .unwrap();
    let svg = render(&data, |chart| {
        chart.heatmap_layout(HeatmapLayout::LowerTriangle { diagonal: true })
    })
    .unwrap();
    let marks = clipped_marks(&svg);

    assert_eq!(marks.matches("<polygon ").count(), 6);
    assert_eq!(marks.matches("<polyline ").count(), 6);
    assert!(svg.contains(">Missing</text>"));
}

#[test]
fn heatmap_completes_grid_and_marks_absent_and_null_cells() {
    let data = df!(
        "column" => ["left", "right", "left"],
        "row" => ["top", "top", "bottom"],
        "value" => [Some(0.0_f64), None, Some(1.0)]
    )
    .unwrap();
    let svg = render(&data, |chart| chart).unwrap();
    let marks = clipped_marks(&svg);

    assert_eq!(marks.matches("<polygon ").count(), 4);
    assert_eq!(marks.matches("<polyline ").count(), 4);
    assert!(svg.contains(">Missing</text>"));
}

#[test]
fn heatmap_places_first_observed_x_left_and_first_observed_y_top() {
    let data = df!(
        "column" => ["second column", "first column"],
        "row" => ["second row", "first row"],
        "value" => [1.0_f64, 2.0]
    )
    .unwrap();
    let svg = render(&data, |chart| chart).unwrap();

    assert!(text_attr(&svg, "Second column", "x") < text_attr(&svg, "First column", "x"));
    assert!(text_attr(&svg, "Second row", "y") < text_attr(&svg, "First row", "y"));
}

#[test]
fn heatmap_applies_explicit_axis_orders_independently_of_cell_row_order() {
    let data = df!(
        "column" => ["right", "left", "right", "left"],
        "row" => ["bottom", "top", "top", "bottom"],
        "value" => [0.75_f64, 0.0, 1.0, 0.5]
    )
    .unwrap();
    let svg = render(&data, |chart| {
        chart
            .x_category_order(CategoryOrder::explicit(["left", "right"]))
            .y_category_order(CategoryOrder::explicit(["top", "bottom"]))
    })
    .unwrap();

    assert!(text_attr(&svg, "Left", "x") < text_attr(&svg, "Right", "x"));
    assert!(text_attr(&svg, "Top", "y") < text_attr(&svg, "Bottom", "y"));
}

#[test]
fn heatmap_reserves_long_y_labels_and_elides_crowded_x_labels() {
    let short = df!(
        "column" => ["c0", "c1", "c2", "c3", "c4", "c5"],
        "row" => ["short"; 6],
        "value" => [0.0_f64, 1.0, 2.0, 3.0, 4.0, 5.0]
    )
    .unwrap();
    let long = df!(
        "column" => [
            "alpha deliberately long label", "bravo deliberately long label",
            "charlie deliberately long label", "delta deliberately long label",
            "echo deliberately long label", "foxtrot deliberately long label"
        ],
        "row" => ["a deliberately long heatmap row label"; 6],
        "value" => [0.0_f64, 1.0, 2.0, 3.0, 4.0, 5.0]
    )
    .unwrap();
    let short_panel = clip_rect(&render(&short, |chart| chart).unwrap());
    let long_svg = render(&long, |chart| chart.size(1_000, 360)).unwrap();
    let long_panel = clip_rect(&long_svg);

    assert!(long_panel.0 > short_panel.0);
    assert!(long_svg.contains("...</text>") && !long_svg.contains("rotate(-45.0"));
}

#[test]
fn heatmap_wrap_policy_preserves_complete_crowded_axis_labels() {
    let data = df!(
        "column" => [
            "alpha deliberately long label", "bravo deliberately long label",
            "charlie deliberately long label", "delta deliberately long label",
            "echo deliberately long label", "foxtrot deliberately long label"
        ],
        "row" => ["a deliberately long heatmap row label"; 6],
        "value" => [0.0_f64, 1.0, 2.0, 3.0, 4.0, 5.0]
    )
    .unwrap();
    let svg = render(&data, |chart| {
        chart
            .size(1_000, 500)
            .heatmap_label_fit(HeatmapLabelFit::Wrap)
    })
    .unwrap();

    assert!(!svg.contains("...</text>"));
    for prefix in ["Alpha", "Bravo", "Charlie", "Delta", "Echo", "Foxtrot"] {
        assert!(svg.contains(prefix), "missing wrapped label {prefix}");
    }
}

#[test]
fn composed_heatmap_separates_x_tick_labels_from_axis_title() {
    let data = fixture();
    let left = chart(&data).x_label("Observed classification");
    let right = chart(&data).x_label("Observed classification");
    let canvas = Canvas::grid(1, 2)
        .unwrap()
        .panel(0, 0, left)
        .unwrap()
        .panel(0, 1, right)
        .unwrap()
        .size(1_400, 600);
    let directory = std::env::temp_dir().join(format!(
        "lichtung-heatmap-canvas-{}-{}",
        std::process::id(),
        TEST_DIRECTORY_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&directory).unwrap();
    let output = directory.join("canvas.svg");
    canvas.svg(&output).unwrap();
    let svg = std::fs::read_to_string(output).unwrap();
    std::fs::remove_dir_all(directory).unwrap();

    let tick_y = text_attr(&svg, "Left", "y");
    let title_y = text_attr(&svg, "Observed classification", "y");
    assert!(title_y - tick_y >= 18.0);
}

#[test]
fn heatmap_sequential_and_diverging_scales_use_pinned_endpoint_colors() {
    let sequential_data = df!(
        "column" => ["low", "high"],
        "row" => ["row", "row"],
        "value" => [0.0_f64, 10.0]
    )
    .unwrap();
    let sequential = render(&sequential_data, |chart| chart).unwrap();
    let diverging_data = df!(
        "column" => ["low", "mid", "high"],
        "row" => ["row", "row", "row"],
        "value" => [-1.0_f64, 0.0, 1.0]
    )
    .unwrap();
    let diverging = render(&diverging_data, |chart| {
        chart.heatmap_scale(HeatmapScale::Diverging { center: 0.0 })
    })
    .unwrap();

    assert!(clipped_marks(&sequential).contains("fill=\"#F7FBFF\""));
    assert!(clipped_marks(&sequential).contains("fill=\"#08306B\""));
    assert!(clipped_marks(&diverging).contains("fill=\"#3182BD\""));
    assert!(clipped_marks(&diverging).contains("fill=\"#F7F7F7\""));
    assert!(clipped_marks(&diverging).contains("fill=\"#DE2D26\""));
}

#[test]
fn heatmap_explicit_color_limits_are_exact_and_saturate_values() {
    let data = df!(
        "column" => ["below", "above"],
        "row" => ["row", "row"],
        "value" => [-10.0_f64, 10.0]
    )
    .unwrap();
    let svg = render(&data, |chart| chart.heatmap_color_limits(0.0, 1.0)).unwrap();
    let marks = clipped_marks(&svg);

    assert!(marks.contains("fill=\"#F7FBFF\""));
    assert!(marks.contains("fill=\"#08306B\""));
    assert!(svg.contains(">0</text>") && svg.contains(">1</text>"));
}

#[test]
fn heatmap_endpoint_diverging_center_is_neutral() {
    let data = df!(
        "column" => ["center", "high"],
        "row" => ["row", "row"],
        "value" => [0.0_f64, 1.0]
    )
    .unwrap();
    let svg = render(&data, |chart| {
        chart
            .heatmap_scale(HeatmapScale::Diverging { center: 0.0 })
            .heatmap_color_limits(0.0, 1.0)
    })
    .unwrap();

    assert!(clipped_marks(&svg).contains("fill=\"#F7F7F7\""));
}

#[test]
fn heatmap_all_missing_grid_renders_with_explicit_color_limits() {
    let data = df!(
        "column" => ["left", "right"],
        "row" => ["row", "row"],
        "value" => [None::<f64>, None]
    )
    .unwrap();
    let svg = render(&data, |chart| chart.heatmap_color_limits(0.0, 1.0)).unwrap();

    assert_eq!(clipped_marks(&svg).matches("<polygon ").count(), 2);
    assert!(svg.contains(">Missing</text>"));
}

#[test]
fn heatmap_rejects_canvas_too_short_for_missing_colorbar() {
    let data = df!(
        "column" => ["left", "right"],
        "row" => ["row", "row"],
        "value" => [Some(1.0_f64), None]
    )
    .unwrap();
    let error = render(&data, |chart| chart.size(320, 150)).unwrap_err();

    assert!(error.contains("minimum 80x60 plot panel"), "{error}");
}

#[test]
fn heatmap_constant_values_use_midpoint_without_invalid_svg() {
    let data = df!(
        "column" => ["one", "two"],
        "row" => ["row", "row"],
        "value" => [5.0_f64, 5.0]
    )
    .unwrap();
    let svg = render(&data, |chart| chart).unwrap();

    assert_eq!(clipped_marks(&svg).matches("fill=\"#8096B5\"").count(), 2);
    assert!(!svg.contains("NaN") && !svg.contains(">inf</text>"));
}

#[test]
fn heatmap_handles_signed_zero_subnormal_and_opposite_extremes() {
    let data = df!(
        "column" => ["negative", "zero", "tiny", "positive"],
        "row" => ["row", "row", "row", "row"],
        "value" => [-f64::MAX, -0.0, f64::from_bits(1), f64::MAX]
    )
    .unwrap();
    let svg = render(&data, |chart| {
        chart.heatmap_scale(HeatmapScale::Diverging { center: 0.0 })
    })
    .unwrap();

    assert!(clipped_marks(&svg).contains("fill=\"#3182BD\""));
    assert!(clipped_marks(&svg).contains("fill=\"#DE2D26\""));
    assert!(!svg.contains("NaN") && !svg.contains(">inf</text>"));
}

#[test]
fn heatmap_colorbar_is_mandatory_right_of_and_outside_marks_clip() {
    let data = fixture();
    let svg = render(&data, |chart| chart.color_label("Measured response")).unwrap();
    let clip_start = svg.find("<g clip-path=\"url(#clip-0)\">").unwrap();
    let clip_end = clip_start + svg[clip_start..].find("</g>").unwrap();
    let label = svg.find(">Measured response</text>").unwrap();
    let panel = clip_rect(&svg);

    assert!(label > clip_end);
    assert!(text_attr(&svg, "Measured response", "x") > panel.0 + panel.2);
    assert_eq!(svg.matches("<linearGradient ").count(), 1);
    assert_eq!(svg.matches("fill=\"url(#linear-gradient-0)\"").count(), 1);
    assert!(svg.contains("<stop offset=\"0.000000\" stop-color=\"#F7FBFF\"/>"));
    assert!(svg.contains("<stop offset=\"1.000000\" stop-color=\"#08306B\"/>"));
}

#[test]
fn heatmap_diverging_colorbar_places_neutral_stop_at_normalized_center() {
    let data = df!(
        "column" => ["low", "center", "high"],
        "row" => ["row", "row", "row"],
        "value" => [-1.0_f64, 0.0, 3.0]
    )
    .unwrap();
    let svg = render(&data, |chart| {
        chart
            .heatmap_scale(HeatmapScale::Diverging { center: 0.0 })
            .heatmap_color_limits(-1.0, 3.0)
    })
    .unwrap();

    assert!(svg.contains("<stop offset=\"0.250000\" stop-color=\"#F7F7F7\"/>"));
}

#[test]
fn heatmap_annotations_use_raw_precision_and_contrasting_text() {
    let data = df!(
        "column" => ["light", "dark"],
        "row" => ["row", "row"],
        "value" => [0.125_f64, 9.875]
    )
    .unwrap();
    let svg = render(&data, |chart| {
        chart.heatmap_annotations(HeatmapAnnotations::Values(
            NumericLabelFormat::new(2).unwrap(),
        ))
    })
    .unwrap();
    let marks = clipped_marks(&svg);

    assert!(marks.contains(">0.12</text>") && marks.contains(">9.88</text>"));
    assert!(marks.contains("fill=\"#000000\"") && marks.contains("fill=\"#FFFFFF\""));
}

#[test]
fn heatmap_merges_annotated_fully_null_row_and_preserves_finite_value_labels() {
    let data = df!(
        "column" => ["left", "middle", "right", "left", "middle", "right"],
        "row" => ["finite", "finite", "finite", "not estimated", "not estimated", "not estimated"],
        "value" => [Some(0.125_f64), Some(0.5), Some(0.875), None, None, None]
    )
    .unwrap();
    let annotation = HeatmapNullRowAnnotation::new(
        "not estimated",
        "No finite measurements were available for this modeled outcome.",
    )
    .unwrap();
    let svg = render(&data, |chart| {
        chart
            .size(480, 300)
            .heatmap_annotations(HeatmapAnnotations::Values(
                NumericLabelFormat::new(2).unwrap(),
            ))
            .heatmap_null_row_annotation(annotation)
    })
    .unwrap();
    let marks = clipped_marks(&svg);

    assert_eq!(marks.matches("<polygon ").count(), 4);
    assert_eq!(marks.matches("<polyline ").count(), 0);
    assert!(marks.contains(">0.12</text>") && marks.contains(">0.50</text>"));
    assert!(
        marks.matches("<text ").count() > 4,
        "annotation was not wrapped: {marks}"
    );
}

#[test]
fn heatmap_null_row_annotation_requires_nonempty_trimmed_labels() {
    let errors = [
        HeatmapNullRowAnnotation::new("", "explanation")
            .unwrap_err()
            .to_string(),
        HeatmapNullRowAnnotation::new(" row", "explanation")
            .unwrap_err()
            .to_string(),
        HeatmapNullRowAnnotation::new("row", " explanation ")
            .unwrap_err()
            .to_string(),
    ];

    assert!(errors
        .iter()
        .all(|error| error.contains("must be nonempty, trimmed")));
}

#[test]
fn heatmap_null_row_annotation_rejects_a_row_with_finite_cells() {
    let data = fixture();
    let annotation = HeatmapNullRowAnnotation::new("top", "Not available.").unwrap();
    let error = render(&data, |chart| chart.heatmap_null_row_annotation(annotation)).unwrap_err();

    assert!(error.contains("must be fully null"));
}

#[test]
fn heatmap_null_row_annotation_rejects_true_highlights_in_merged_row() {
    let data = df!(
        "column" => ["left", "right", "left", "right"],
        "row" => ["finite", "finite", "not estimated", "not estimated"],
        "value" => [Some(0.0_f64), Some(1.0), None, None],
        "selected" => [false, false, true, false]
    )
    .unwrap();
    let annotation = HeatmapNullRowAnnotation::new("not estimated", "Not available.").unwrap();
    let error = render(&data, |chart| {
        chart
            .heatmap_highlight(data::boolean(&data, "selected"))
            .heatmap_null_row_annotation(annotation)
    })
    .unwrap_err();

    assert!(error.contains("cannot combine a merged annotation with true cell highlights"));
}

#[test]
fn heatmap_null_row_annotation_errors_when_wrapped_text_cannot_fit() {
    let mut columns = Vec::new();
    let mut rows = Vec::new();
    let mut values = Vec::new();
    for row_index in 0..9 {
        for column in ["left", "right"] {
            columns.push(column);
            rows.push(format!("finite {row_index}"));
            values.push(Some(1.0_f64));
        }
    }
    for column in ["left", "right"] {
        columns.push(column);
        rows.push("not estimated".to_string());
        values.push(None);
    }
    let data = df!("column" => columns, "row" => rows, "value" => values).unwrap();
    let annotation = HeatmapNullRowAnnotation::new(
        "not estimated",
        "This deliberately long annotation must wrap onto too many lines for the available row height.",
    )
    .unwrap();
    let error = render(&data, |chart| {
        chart.size(480, 300).heatmap_null_row_annotation(annotation)
    })
    .unwrap_err();

    assert!(
        error.contains("does not fit its merged row"),
        "unexpected error: {error}"
    );
}

#[test]
fn heatmap_annotations_append_suffix_and_border_only_selected_cells() {
    let data = df!(
        "column" => ["left", "right"],
        "row" => ["row", "row"],
        "value" => [0.125_f64, 0.875],
        "selected" => [true, false]
    )
    .unwrap();
    let svg = render(&data, |chart| {
        chart
            .heatmap_highlight(data::boolean(&data, "selected"))
            .heatmap_annotations(HeatmapAnnotations::Values(
                NumericLabelFormat::with_suffix(1, "%").unwrap(),
            ))
    })
    .unwrap();
    let marks = clipped_marks(&svg);

    assert!(marks.contains(">0.1%</text>") && marks.contains(">0.9%</text>"));
    assert_eq!(marks.matches("stroke-width=\"2.5\"").count(), 1);
    assert_eq!(marks.matches("stroke-width=\"0.5\"").count(), 1);
}

#[test]
fn heatmap_highlight_requires_complete_boolean_values() {
    let null_value = df!(
        "column" => ["left", "right"],
        "row" => ["row", "row"],
        "value" => [0.125_f64, 0.875],
        "selected" => [Some(true), None]
    )
    .unwrap();

    let error = render(&null_value, |chart| {
        chart.heatmap_highlight(data::boolean(&null_value, "selected"))
    })
    .unwrap_err();
    assert!(error.contains("heatmap:") && error.contains("contains null"));
}

#[test]
fn numeric_label_format_rejects_invalid_precision_and_suffix() {
    let errors = [
        NumericLabelFormat::new(13).unwrap_err().to_string(),
        NumericLabelFormat::with_suffix(2, "\n")
            .unwrap_err()
            .to_string(),
        NumericLabelFormat::with_suffix(2, "")
            .unwrap_err()
            .to_string(),
    ];

    assert!(errors.iter().all(|error| error.contains("numeric label")));
}

#[test]
fn heatmap_annotations_error_when_cells_cannot_fit_values() {
    let columns = (0..30).map(|index| format!("c{index}")).collect::<Vec<_>>();
    let rows = vec!["row"; columns.len()];
    let values = vec![12345.6789_f64; columns.len()];
    let data = df!("column" => columns, "row" => rows, "value" => values).unwrap();
    let error = render(&data, |chart| {
        chart
            .size(360, 220)
            .heatmap_annotations(HeatmapAnnotations::Values(
                NumericLabelFormat::new(4).unwrap(),
            ))
    })
    .unwrap_err();

    assert!(error.contains("cells are too small"));
}

#[test]
fn heatmap_rejects_value_sorted_category_order() {
    let data = fixture();
    let error = render(&data, |chart| chart.x_category_order(ValueSort::Descending)).unwrap_err();

    assert!(error.contains("heatmap:"));
}

#[test]
fn heatmap_rejects_invalid_scale_options() {
    let data = fixture();
    let errors = [
        render(&data, |chart| chart.heatmap_color_limits(1.0, 1.0)).unwrap_err(),
        render(&data, |chart| {
            chart
                .heatmap_scale(HeatmapScale::Diverging { center: 0.0 })
                .heatmap_color_limits(0.5, 1.0)
        })
        .unwrap_err(),
        render(&data, |chart| {
            chart.heatmap_scale(HeatmapScale::Diverging {
                center: f64::INFINITY,
            })
        })
        .unwrap_err(),
    ];

    assert!(errors.iter().all(|error| error.contains("heatmap:")));
}

#[test]
fn heatmap_svg_is_byte_deterministic() {
    let data = fixture();
    let first = render(&data, |chart| chart).unwrap();
    let second = render(&data, |chart| chart).unwrap();

    assert_eq!(first, second);
}

#[cfg(feature = "png")]
#[test]
fn heatmap_png_is_rasterized_from_the_native_gradient_svg() {
    let data = fixture();
    let directory = std::env::temp_dir().join(format!(
        "lichtung-heatmap-png-{}-{}",
        std::process::id(),
        TEST_DIRECTORY_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&directory).unwrap();
    let svg_path = directory.join("plot.svg");
    let png_path = directory.join("plot.png");
    chart(&data).svg_and_png(&svg_path, &png_path).unwrap();

    let svg = std::fs::read_to_string(svg_path).unwrap();
    let png = std::fs::read(png_path).unwrap();
    assert!(svg.contains("<linearGradient id=\"linear-gradient-0\""));
    assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));
    std::fs::remove_dir_all(directory).unwrap();
}

fn clipped_marks(svg: &str) -> &str {
    svg.split_once("<g clip-path=\"url(#clip-0)\">")
        .unwrap()
        .1
        .split_once("</g>")
        .unwrap()
        .0
}

fn clip_rect(svg: &str) -> (f64, f64, f64, f64) {
    let clip = svg
        .split_once("<clipPath id=\"clip-0\">")
        .unwrap()
        .1
        .split_once("</clipPath>")
        .unwrap()
        .0;
    (
        attr(clip, "x"),
        attr(clip, "y"),
        attr(clip, "width"),
        attr(clip, "height"),
    )
}

fn text_tag<'a>(svg: &'a str, text: &str) -> &'a str {
    let needle = format!(">{text}</text>");
    let end = svg.find(&needle).unwrap();
    let start = svg[..end].rfind("<text ").unwrap();
    &svg[start..end]
}

fn text_attr(svg: &str, text: &str, name: &str) -> f64 {
    attr(text_tag(svg, text), name)
}

fn attr(tag: &str, name: &str) -> f64 {
    tag.split_once(&format!("{name}=\""))
        .unwrap()
        .1
        .split_once('"')
        .unwrap()
        .0
        .parse()
        .unwrap()
}
