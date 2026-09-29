use std::sync::atomic::{AtomicU64, Ordering};

use lichtung_plot::{
    BooleanValues, Canvas, CategoryOrder, CategoryValues, HeatmapAnnotations, HeatmapLabelFit,
    HeatmapLayout, HeatmapNullRowAnnotation, HeatmapScale, NumericLabelFormat, NumericValues, Plot,
    ValueSort,
};

static TEST_DIRECTORY_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Clone)]
struct HeatmapData {
    column: CategoryValues,
    row: CategoryValues,
    value: NumericValues,
    highlight: Option<BooleanValues>,
}

impl HeatmapData {
    fn new(column: CategoryValues, row: CategoryValues, value: NumericValues) -> Self {
        Self {
            column,
            row,
            value,
            highlight: None,
        }
    }

    fn with_highlight(mut self, highlight: BooleanValues) -> Self {
        self.highlight = Some(highlight);
        self
    }
}

fn standard_data() -> HeatmapData {
    HeatmapData::new(
        CategoryValues::new("column", ["left", "right", "left", "right"]).unwrap(),
        CategoryValues::new("row", ["top", "top", "bottom", "bottom"]).unwrap(),
        NumericValues::new("value", [0.0, 1.0, 0.5, 0.75]),
    )
}

fn chart(data: &HeatmapData) -> Plot<lichtung_plot::kind::Heatmap> {
    Plot::heatmap(data.column.clone(), data.row.clone(), data.value.clone()).size(560, 360)
}

fn render(
    data: &HeatmapData,
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
        .svg_preserve_text(&output)
        .map(|_| std::fs::read_to_string(&output).unwrap())
        .map_err(|error| error.to_string());
    std::fs::remove_dir_all(directory).unwrap();
    result
}

#[test]
fn heatmap_constructor_accepts_typed_configuration() {
    let data = HeatmapData::new(
        CategoryValues::new("column", ["a", "b", "b"]).unwrap(),
        CategoryValues::new("row", ["a", "a", "b"]).unwrap(),
        NumericValues::new("value", [0.0, 1.0, 0.5]),
    );
    let svg = render(&data, |plot| {
        plot.color_label("Response")
            .layout(HeatmapLayout::UpperTriangle { diagonal: true })
            .scale(HeatmapScale::Diverging { center: 0.5 })
            .color_limits(-1.0, 2.0)
            .annotations(HeatmapAnnotations::Values(NumericLabelFormat::default()))
    })
    .unwrap();
    assert!(svg.contains(">Response</text>"));
    assert!(svg.contains(">-1</text>") && svg.contains(">2</text>"));
}

#[test]
fn heatmap_cartesian_default_is_byte_identical_to_explicit_cartesian() {
    let data = standard_data();

    let default = render(&data, |chart| chart).unwrap();
    let explicit = render(&data, |chart| chart.layout(HeatmapLayout::Cartesian)).unwrap();

    assert_eq!(default, explicit);
}

#[test]
fn heatmap_hidden_annotations_and_all_false_highlights_are_byte_inert() {
    let data = standard_data()
        .with_highlight(BooleanValues::new("selected", [false, false, false, false]));

    let default = render(&data, |chart| chart).unwrap();
    let explicit = render(&data, |chart| {
        chart
            .annotations(HeatmapAnnotations::Hidden)
            .heatmap_highlight(data.highlight.clone().unwrap())
    })
    .unwrap();

    assert_eq!(default, explicit);
}

#[test]
fn heatmap_lower_triangle_suppresses_structural_cells_without_missing_marks() {
    let data = HeatmapData::new(
        CategoryValues::new("column", ["a", "a", "b"]).unwrap(),
        CategoryValues::new("row", ["b", "c", "c"]).unwrap(),
        NumericValues::new("value", [1.0, 2.0, 3.0]),
    );
    let svg = render(&data, |chart| {
        chart.layout(HeatmapLayout::LowerTriangle { diagonal: false })
    })
    .unwrap();
    let marks = clipped_marks(&svg);

    assert_eq!(marks.matches("<polygon ").count(), 3);
    assert_eq!(marks.matches("<polyline ").count(), 0);
    assert!(!svg.contains(">Missing</text>"));
}

#[test]
fn heatmap_triangle_marks_only_missing_cells_inside_active_triangle() {
    let data = HeatmapData::new(
        CategoryValues::new("column", ["a", "a"]).unwrap(),
        CategoryValues::new("row", ["b", "c"]).unwrap(),
        NumericValues::new("value", [1.0, 2.0]),
    );
    let svg = render(&data, |chart| {
        chart.layout(HeatmapLayout::LowerTriangle { diagonal: false })
    })
    .unwrap();
    let marks = clipped_marks(&svg);

    assert_eq!(marks.matches("<polygon ").count(), 3);
    assert_eq!(marks.matches("<polyline ").count(), 2);
    assert!(svg.contains(">Missing</text>"));
}

#[test]
fn heatmap_triangle_active_diagonal_uses_missing_semantics() {
    let data = HeatmapData::new(
        CategoryValues::new("column", ["a", "a", "b"]).unwrap(),
        CategoryValues::new("row", ["b", "c", "c"]).unwrap(),
        NumericValues::new("value", [1.0, 2.0, 3.0]),
    );
    let svg = render(&data, |chart| {
        chart.layout(HeatmapLayout::LowerTriangle { diagonal: true })
    })
    .unwrap();
    let marks = clipped_marks(&svg);

    assert_eq!(marks.matches("<polygon ").count(), 6);
    assert_eq!(marks.matches("<polyline ").count(), 6);
    assert!(svg.contains(">Missing</text>"));
}

#[test]
fn heatmap_completes_grid_and_marks_absent_and_null_cells() {
    let data = HeatmapData::new(
        CategoryValues::new("column", ["left", "right", "left"]).unwrap(),
        CategoryValues::new("row", ["top", "top", "bottom"]).unwrap(),
        NumericValues::nullable("value", [Some(0.0), None, Some(1.0)]),
    );
    let svg = render(&data, |chart| chart).unwrap();
    let marks = clipped_marks(&svg);

    assert_eq!(marks.matches("<polygon ").count(), 4);
    assert_eq!(marks.matches("<polyline ").count(), 4);
    assert!(svg.contains(">Missing</text>"));
}

#[test]
fn heatmap_places_first_observed_x_left_and_first_observed_y_top() {
    let data = HeatmapData::new(
        CategoryValues::new("column", ["second column", "first column"]).unwrap(),
        CategoryValues::new("row", ["second row", "first row"]).unwrap(),
        NumericValues::new("value", [1.0, 2.0]),
    );
    let svg = render(&data, |chart| chart).unwrap();

    assert!(text_attr(&svg, "Second column", "x") < text_attr(&svg, "First column", "x"));
    assert!(text_attr(&svg, "Second row", "y") < text_attr(&svg, "First row", "y"));
}

#[test]
fn heatmap_applies_explicit_axis_orders_independently_of_cell_row_order() {
    let data = HeatmapData::new(
        CategoryValues::new("column", ["right", "left", "right", "left"]).unwrap(),
        CategoryValues::new("row", ["bottom", "top", "top", "bottom"]).unwrap(),
        NumericValues::new("value", [0.75, 0.0, 1.0, 0.5]),
    );
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
    let short = HeatmapData::new(
        CategoryValues::new("column", ["c0", "c1", "c2", "c3", "c4", "c5"]).unwrap(),
        CategoryValues::new("row", ["short"; 6]).unwrap(),
        NumericValues::new("value", [0.0, 1.0, 2.0, 3.0, 4.0, 5.0]),
    );
    let long = HeatmapData::new(
        CategoryValues::new(
            "column",
            [
                "alpha very long label",
                "bravo very long label",
                "charlie very long label",
                "delta very long label",
                "echo very long label",
                "foxtrot very long label",
            ],
        )
        .unwrap(),
        CategoryValues::new("row", ["a very long heatmap row label"; 6]).unwrap(),
        NumericValues::new("value", [0.0, 1.0, 2.0, 3.0, 4.0, 5.0]),
    );
    let short_panel = clip_rect(&render(&short, |chart| chart).unwrap());
    let long_svg = render(&long, |chart| chart.size(1_000, 360)).unwrap();
    let long_panel = clip_rect(&long_svg);

    assert!(long_panel.0 > short_panel.0);
    assert!(long_svg.contains("...</text>") && !long_svg.contains("rotate(-45.0"));
}

#[test]
fn heatmap_wrap_policy_preserves_complete_crowded_axis_labels() {
    let data = HeatmapData::new(
        CategoryValues::new(
            "column",
            [
                "alpha deliberately long label",
                "bravo deliberately long label",
                "charlie deliberately long label",
                "delta deliberately long label",
                "echo deliberately long label",
                "foxtrot deliberately long label",
            ],
        )
        .unwrap(),
        CategoryValues::new("row", ["a deliberately long heatmap row label"; 6]).unwrap(),
        NumericValues::new("value", [0.0, 1.0, 2.0, 3.0, 4.0, 5.0]),
    );
    let svg = render(&data, |chart| {
        chart
            .size(1_200, 500)
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
    let data = standard_data();
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
    canvas.svg_preserve_text(&output).unwrap();
    let svg = std::fs::read_to_string(output).unwrap();
    std::fs::remove_dir_all(directory).unwrap();

    let tick_y = text_attr(&svg, "Left", "y");
    let title_y = text_attr(&svg, "Observed classification", "y");
    assert!(title_y - tick_y >= 18.0);
}

fn canvas_heatmap_label_policy(policy: &lichtung_plot::Labels, expected: [&str; 7]) {
    for explicit in [false, true] {
        let plot = Plot::heatmap(
            CategoryValues::new("x_axis", ["a_b", "a b", "a_b", "a b"]).unwrap(),
            CategoryValues::new("y_axis", ["c_d", "c_d", "c d", "c d"]).unwrap(),
            NumericValues::new("color_value", [1.0, 2.0, 3.0, 4.0]),
        );
        let plot = if explicit {
            plot.x_label("literal_x")
                .y_label("literal_y")
                .color_label("literal_color")
        } else {
            plot
        };
        let canvas = Canvas::grid(1, 1)
            .unwrap()
            .panel(0, 0, plot)
            .unwrap()
            .labels(
                policy
                    .clone()
                    .alias("literal_x", "Must not replace x")
                    .alias("literal_y", "Must not replace y")
                    .alias("literal_color", "Must not replace color"),
            )
            .size(800, 600);
        let directory = tempfile::tempdir().unwrap();
        let output = directory.path().join("canvas.svg");
        canvas.svg_preserve_text(&output).unwrap();
        let svg = std::fs::read_to_string(output).unwrap();
        for label in &expected[..4] {
            assert!(
                svg.contains(&format!(">{label}</text>")),
                "missing tick {label}"
            );
        }
        let titles = if explicit {
            &["literal_x", "literal_y", "literal_color"]
        } else {
            &expected[4..]
        };
        for label in titles {
            assert!(
                svg.contains(&format!(">{label}</text>")),
                "missing title {label}"
            );
        }
        assert!(!svg.contains("Must not replace"));
    }
}

#[test]
fn canvas_heatmap_raw_labels_preserve_categories_that_collide_when_humanized() {
    canvas_heatmap_label_policy(
        &lichtung_plot::Labels::raw(),
        [
            "a_b",
            "a b",
            "c_d",
            "c d",
            "x_axis",
            "y_axis",
            "color_value",
        ],
    );
}

#[test]
fn canvas_heatmap_aliases_disambiguate_both_category_axes() {
    canvas_heatmap_label_policy(
        &lichtung_plot::Labels::humanized()
            .alias("a_b", "First X")
            .alias("a b", "Second X")
            .alias("c_d", "First Y")
            .alias("c d", "Second Y")
            .alias("x_axis", "Horizontal")
            .alias("y_axis", "Vertical")
            .alias("color_value", "Intensity"),
        [
            "First X",
            "Second X",
            "First Y",
            "Second Y",
            "Horizontal",
            "Vertical",
            "Intensity",
        ],
    );
}

#[test]
fn heatmap_sequential_and_diverging_scales_use_pinned_endpoint_colors() {
    let sequential_data = HeatmapData::new(
        CategoryValues::new("column", ["low", "high"]).unwrap(),
        CategoryValues::new("row", ["row", "row"]).unwrap(),
        NumericValues::new("value", [0.0, 10.0]),
    );
    let sequential = render(&sequential_data, |chart| chart).unwrap();
    let diverging_data = HeatmapData::new(
        CategoryValues::new("column", ["low", "mid", "high"]).unwrap(),
        CategoryValues::new("row", ["row", "row", "row"]).unwrap(),
        NumericValues::new("value", [-1.0, 0.0, 1.0]),
    );
    let diverging = render(&diverging_data, |chart| {
        chart.scale(HeatmapScale::Diverging { center: 0.0 })
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
    let data = HeatmapData::new(
        CategoryValues::new("column", ["below", "above"]).unwrap(),
        CategoryValues::new("row", ["row", "row"]).unwrap(),
        NumericValues::new("value", [-10.0, 10.0]),
    );
    let svg = render(&data, |chart| chart.color_limits(0.0, 1.0)).unwrap();
    let marks = clipped_marks(&svg);

    assert!(marks.contains("fill=\"#F7FBFF\""));
    assert!(marks.contains("fill=\"#08306B\""));
    assert!(svg.contains(">0</text>") && svg.contains(">1</text>"));
}

#[test]
fn heatmap_endpoint_diverging_center_is_neutral() {
    let data = HeatmapData::new(
        CategoryValues::new("column", ["center", "high"]).unwrap(),
        CategoryValues::new("row", ["row", "row"]).unwrap(),
        NumericValues::new("value", [0.0, 1.0]),
    );
    let svg = render(&data, |chart| {
        chart
            .scale(HeatmapScale::Diverging { center: 0.0 })
            .color_limits(0.0, 1.0)
    })
    .unwrap();

    assert!(clipped_marks(&svg).contains("fill=\"#F7F7F7\""));
}

#[test]
fn heatmap_all_missing_grid_renders_with_explicit_color_limits() {
    let data = HeatmapData::new(
        CategoryValues::new("column", ["left", "right"]).unwrap(),
        CategoryValues::new("row", ["row", "row"]).unwrap(),
        NumericValues::nullable("value", [None, None]),
    );
    let svg = render(&data, |chart| chart.color_limits(0.0, 1.0)).unwrap();

    assert_eq!(clipped_marks(&svg).matches("<polygon ").count(), 2);
    assert!(svg.contains(">Missing</text>"));
}

#[test]
fn heatmap_rejects_canvas_too_short_for_missing_colorbar() {
    let data = HeatmapData::new(
        CategoryValues::new("column", ["left", "right"]).unwrap(),
        CategoryValues::new("row", ["row", "row"]).unwrap(),
        NumericValues::nullable("value", [Some(1.0), None]),
    );
    let error = render(&data, |chart| chart.size(320, 150)).unwrap_err();

    assert!(
        error.contains("root title block exceeds its exact allocated height"),
        "{error}"
    );
}

#[test]
fn heatmap_constant_values_use_midpoint_without_invalid_svg() {
    let data = HeatmapData::new(
        CategoryValues::new("column", ["one", "two"]).unwrap(),
        CategoryValues::new("row", ["row", "row"]).unwrap(),
        NumericValues::new("value", [5.0, 5.0]),
    );
    let svg = render(&data, |chart| chart).unwrap();

    assert_eq!(clipped_marks(&svg).matches("fill=\"#8096B5\"").count(), 2);
    assert!(!svg.contains("NaN") && !svg.contains(">inf</text>"));
}

#[test]
fn heatmap_handles_signed_zero_subnormal_and_opposite_extremes() {
    let data = HeatmapData::new(
        CategoryValues::new("column", ["negative", "zero", "tiny", "positive"]).unwrap(),
        CategoryValues::new("row", ["row", "row", "row", "row"]).unwrap(),
        NumericValues::new("value", [-f64::MAX, -0.0, f64::from_bits(1), f64::MAX]),
    );
    let svg = render(&data, |chart| {
        chart.scale(HeatmapScale::Diverging { center: 0.0 })
    })
    .unwrap();

    assert!(clipped_marks(&svg).contains("fill=\"#3182BD\""));
    assert!(clipped_marks(&svg).contains("fill=\"#DE2D26\""));
    assert!(!svg.contains("NaN") && !svg.contains(">inf</text>"));
}

#[test]
fn heatmap_colorbar_is_mandatory_right_of_and_outside_marks_clip() {
    let data = standard_data();
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
    let data = HeatmapData::new(
        CategoryValues::new("column", ["low", "center", "high"]).unwrap(),
        CategoryValues::new("row", ["row", "row", "row"]).unwrap(),
        NumericValues::new("value", [-1.0, 0.0, 3.0]),
    );
    let svg = render(&data, |chart| {
        chart
            .scale(HeatmapScale::Diverging { center: 0.0 })
            .color_limits(-1.0, 3.0)
    })
    .unwrap();

    assert!(svg.contains("<stop offset=\"0.250000\" stop-color=\"#F7F7F7\"/>"));
}

#[test]
fn heatmap_annotations_use_default_and_chained_formats_with_contrasting_text() {
    let data = HeatmapData::new(
        CategoryValues::new("column", ["light", "dark"]).unwrap(),
        CategoryValues::new("row", ["row", "row"]).unwrap(),
        NumericValues::new("value", [0.125, 9.875]),
    );
    let default_svg = render(&data, |chart| {
        chart.annotations(HeatmapAnnotations::Values(NumericLabelFormat::default()))
    })
    .unwrap();
    let configured_svg = render(&data, |chart| {
        chart.annotations(HeatmapAnnotations::Values(
            NumericLabelFormat::default()
                .precision(1)
                .unwrap()
                .suffix("%")
                .unwrap(),
        ))
    })
    .unwrap();
    let default_marks = clipped_marks(&default_svg);
    let configured_marks = clipped_marks(&configured_svg);

    assert!(default_marks.contains(">0.12</text>") && default_marks.contains(">9.88</text>"));
    assert!(configured_marks.contains(">0.1%</text>") && configured_marks.contains(">9.9%</text>"));
    assert!(
        default_marks.contains("fill=\"#000000\"") && default_marks.contains("fill=\"#FFFFFF\"")
    );
}

#[test]
fn diverging_cells_choose_complete_light_or_dark_text_before_paint() {
    let data = HeatmapData::new(
        CategoryValues::new("column", ["low", "center", "high"]).unwrap(),
        CategoryValues::new("row", ["one", "one", "one"]).unwrap(),
        NumericValues::new("value", [-1.0, 0.0, 1.0]),
    );
    let svg = render(&data, |chart| {
        chart
            .scale(HeatmapScale::Diverging { center: 0.0 })
            .color_limits(-1.0, 1.0)
            .annotations(HeatmapAnnotations::Values(NumericLabelFormat::default()))
    })
    .unwrap();
    let marks = clipped_marks(&svg);
    for (text, paint) in [
        ("-1.00", "#000000"),
        ("0.00", "#000000"),
        ("1.00", "#FFFFFF"),
    ] {
        let line = marks
            .lines()
            .find(|line| line.contains(&format!(">{text}</text>")))
            .unwrap();
        assert!(line.contains(&format!("fill=\"{paint}\"")), "{line}");
    }
}

#[test]
fn heatmap_merges_annotated_fully_null_row_and_preserves_finite_value_labels() {
    let data = HeatmapData::new(
        CategoryValues::new(
            "column",
            ["left", "middle", "right", "left", "middle", "right"],
        )
        .unwrap(),
        CategoryValues::new(
            "row",
            [
                "finite", "finite", "finite", "not fit", "not fit", "not fit",
            ],
        )
        .unwrap(),
        NumericValues::nullable(
            "value",
            [Some(0.125), Some(0.5), Some(0.875), None, None, None],
        ),
    );
    let annotation = HeatmapNullRowAnnotation::new(
        "not fit",
        "No finite measurements were available for this modeled outcome.",
    )
    .unwrap();
    let svg = render(&data, |chart| {
        chart
            .size(480, 300)
            .annotations(HeatmapAnnotations::Values(NumericLabelFormat::default()))
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
    let data = standard_data();
    let annotation = HeatmapNullRowAnnotation::new("top", "Not available.").unwrap();
    let error = render(&data, |chart| chart.heatmap_null_row_annotation(annotation)).unwrap_err();

    assert!(error.contains("must be fully null"));
}

#[test]
fn heatmap_null_row_annotation_rejects_true_highlights_in_merged_row() {
    let data = HeatmapData::new(
        CategoryValues::new("column", ["left", "right", "left", "right"]).unwrap(),
        CategoryValues::new(
            "row",
            ["finite", "finite", "not estimated", "not estimated"],
        )
        .unwrap(),
        NumericValues::nullable("value", [Some(0.0), Some(1.0), None, None]),
    )
    .with_highlight(BooleanValues::new("selected", [false, false, true, false]));
    let annotation = HeatmapNullRowAnnotation::new("not estimated", "Not available.").unwrap();
    let error = render(&data, |chart| {
        chart
            .heatmap_highlight(data.highlight.clone().unwrap())
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
    let data = HeatmapData::new(
        CategoryValues::new("column", columns).unwrap(),
        CategoryValues::new("row", rows).unwrap(),
        NumericValues::nullable("value", values),
    );
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
    let data = HeatmapData::new(
        CategoryValues::new("column", ["left", "right"]).unwrap(),
        CategoryValues::new("row", ["row", "row"]).unwrap(),
        NumericValues::new("value", [0.125, 0.875]),
    )
    .with_highlight(BooleanValues::new("selected", [true, false]));
    let svg = render(&data, |chart| {
        chart
            .heatmap_highlight(data.highlight.clone().unwrap())
            .annotations(HeatmapAnnotations::Values(
                NumericLabelFormat::default()
                    .precision(1)
                    .unwrap()
                    .suffix("%")
                    .unwrap(),
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
    let null_value = HeatmapData::new(
        CategoryValues::new("column", ["left", "right"]).unwrap(),
        CategoryValues::new("row", ["row", "row"]).unwrap(),
        NumericValues::new("value", [0.125, 0.875]),
    )
    .with_highlight(BooleanValues::nullable("selected", [Some(true), None]));

    let error = render(&null_value, |chart| {
        chart.heatmap_highlight(null_value.highlight.clone().unwrap())
    })
    .unwrap_err();
    assert!(error.contains("heatmap:") && error.contains("contains null"));
}

#[test]
fn numeric_label_format_rejects_invalid_precision_and_suffix() {
    let errors = [
        NumericLabelFormat::default()
            .precision(13)
            .unwrap_err()
            .to_string(),
        NumericLabelFormat::default()
            .suffix("")
            .unwrap_err()
            .to_string(),
        NumericLabelFormat::default()
            .suffix("\n")
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
    let data = HeatmapData::new(
        CategoryValues::new("column", columns).unwrap(),
        CategoryValues::new("row", rows).unwrap(),
        NumericValues::new("value", values),
    );
    let plot = chart(&data)
        .size(360, 220)
        .annotations(HeatmapAnnotations::Values(
            NumericLabelFormat::default().precision(4).unwrap(),
        ));
    let directory = tempfile::tempdir().unwrap();
    let output = directory.path().join("plot.svg");
    let preflight_error = plot.preflight().unwrap_err();
    let error = plot.svg_preserve_text(&output).map(|_| ()).unwrap_err();

    assert!(!output.exists());
    assert_eq!(preflight_error.to_string(), error.to_string());
    let lichtung_plot::Error::Render(message) = error else {
        panic!("expected a layout fit error, got {error:?}");
    };
    // Complete formatted labels now reject at the root child-hard boundary,
    // before the former generic panel reservation or any cell allocation.
    assert_eq!(
        message,
        concat!(
            "layout fit FitReason { kind: TrackOrResidual, ",
            "address: ComponentAddress { root: AddressSegment { role: Root, slot_index: 0 }, ",
            "descendants: [] }, ordinal: 0 }: ",
            "root title block leaves less than the measured child body minimum"
        )
    );

    let fitting = render(&data, |chart| {
        chart
            .size(3000, 220)
            .annotations(HeatmapAnnotations::Values(
                NumericLabelFormat::default().precision(4).unwrap(),
            ))
    })
    .unwrap();
    let marks = clipped_marks(&fitting);
    assert_eq!(marks.matches("<polygon ").count(), 30);
    assert_eq!(marks.matches(">12345.6789</text>").count(), 30);
}

#[test]
fn heatmap_rejects_value_sorted_category_order() {
    let data = standard_data();
    let error = render(&data, |chart| chart.x_category_order(ValueSort::Descending)).unwrap_err();

    assert!(error.contains("heatmap:"));
}

#[test]
fn heatmap_rejects_invalid_scale_options() {
    let data = standard_data();
    let errors = [
        render(&data, |chart| chart.color_limits(1.0, 1.0)).unwrap_err(),
        render(&data, |chart| {
            chart
                .scale(HeatmapScale::Diverging { center: 0.0 })
                .color_limits(0.5, 1.0)
        })
        .unwrap_err(),
        render(&data, |chart| {
            chart.scale(HeatmapScale::Diverging {
                center: f64::INFINITY,
            })
        })
        .unwrap_err(),
    ];

    assert!(errors.iter().all(|error| error.contains("heatmap:")));
}

#[test]
fn heatmap_svg_is_byte_deterministic() {
    let data = standard_data();
    let first = render(&data, |chart| chart).unwrap();
    let second = render(&data, |chart| chart).unwrap();

    assert_eq!(first, second);
}

#[test]
fn heatmap_png_is_rasterized_from_the_native_gradient_svg() {
    let data = standard_data();
    let directory = std::env::temp_dir().join(format!(
        "lichtung-heatmap-png-{}-{}",
        std::process::id(),
        TEST_DIRECTORY_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&directory).unwrap();
    let svg_path = directory.join("plot.svg");
    let png_path = directory.join("plot.png");
    chart(&data)
        .svg_preserve_text(&svg_path)
        .unwrap()
        .png_preview(&png_path)
        .unwrap();

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
