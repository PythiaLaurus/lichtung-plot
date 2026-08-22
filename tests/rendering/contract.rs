use lichtung_plot::{
    BarAggregation, BarPointLayer, Canvas, InferentialScale, IntervalKind, IntervalSemantics, Plot,
    SignificanceStarLimit, TextPanel,
};

use super::{data, df, unique_dir, ComparisonAnnotation, DataFrame};

fn interval_semantics() -> IntervalSemantics {
    IntervalSemantics::new(
        IntervalKind::Confidence,
        0.9,
        InferentialScale::Response,
        "caller bootstrap",
    )
    .unwrap()
}

fn identity_interval_data() -> DataFrame {
    df!(
        "category" => ["a", "b"],
        "value" => [2.0_f64, 5.0],
        "lower" => [1.0_f64, 3.5],
        "upper" => [4.0_f64, 8.0],
    )
    .unwrap()
}

fn identity_interval_plot(data: &DataFrame, horizontal: bool) -> Plot {
    let plot = if horizontal {
        Plot::horizontal_bar(
            data::category(data, "category"),
            data::numeric(data, "value"),
        )
    } else {
        Plot::bar(
            data::category(data, "category"),
            data::numeric(data, "value"),
        )
    };
    plot.bar_aggregation(BarAggregation::Identity)
        .interval_bounds(data::numeric(data, "lower"), data::numeric(data, "upper"))
        .interval_semantics(interval_semantics())
}

fn svg(plot: Plot, name: &str) -> String {
    let directory = unique_dir(name);
    let output = directory.join("plot.svg");
    plot.svg(&output).unwrap();
    let svg = std::fs::read_to_string(output).unwrap();
    std::fs::remove_dir_all(directory).unwrap();
    svg
}

#[test]
fn identity_bar_intervals_render_asymmetric_endpoints_without_canvas_provenance() {
    let data = identity_interval_data();
    let vertical = svg(
        identity_interval_plot(&data, false),
        "vertical-identity-interval",
    );
    let horizontal = svg(
        identity_interval_plot(&data, true),
        "horizontal-identity-interval",
    );

    assert!(!vertical.contains("90% confidence interval · response scale · caller bootstrap"));
    assert!(!horizontal.contains("90% confidence interval · response scale · caller bootstrap"));
    let plain = svg(
        Plot::bar(
            data::category(&data, "category"),
            data::numeric(&data, "value"),
        )
        .bar_aggregation(BarAggregation::Identity),
        "plain-identity-bar",
    );
    assert!(vertical.matches("<polyline ").count() > plain.matches("<polyline ").count());
    assert!(horizontal.matches("<polyline ").count() > plain.matches("<polyline ").count());
}

#[test]
fn generated_mean_bars_reject_explicit_identity_interval_contracts() {
    let data = identity_interval_data();
    let mean = Plot::bar(
        data::category(&data, "category"),
        data::numeric(&data, "value"),
    )
    .interval_bounds(data::numeric(&data, "lower"), data::numeric(&data, "upper"))
    .interval_semantics(interval_semantics());

    let error = mean.preflight().unwrap_err().to_string();

    assert!(error.contains("generated mean intervals are owned by BarAggregation"));
}

#[test]
fn identity_bar_rejects_invalid_endpoint_order_during_preflight() {
    let data = df!(
        "category" => ["a"],
        "value" => [2.0_f64],
        "lower" => [3.0_f64],
        "upper" => [4.0_f64],
    )
    .unwrap();
    let error = identity_interval_plot(&data, false)
        .preflight()
        .unwrap_err();

    assert!(error.to_string().contains("lower <= value <= upper"));
}

#[test]
fn bar_point_layers_retain_rows_at_exact_grouped_centers_in_both_orientations() {
    let bars = df!(
        "category" => ["a", "a", "b", "b"],
        "group" => ["left", "right", "left", "right"],
        "value" => [2.0_f64, 3.0, 4.0, 5.0],
    )
    .unwrap();
    let points = df!(
        "point_category" => ["a", "a", "b"],
        "point_group" => ["left", "right", "left"],
        "point_value" => [1.0_f64, 7.0, 6.0],
    )
    .unwrap();
    let layer = || {
        BarPointLayer::new(
            data::category(&points, "point_category"),
            Some(data::category(&points, "point_group")),
            data::numeric(&points, "point_value"),
            (123, 45, 200),
            5,
            0.4,
        )
        .unwrap()
    };
    let render = |horizontal| {
        let plot = if horizontal {
            Plot::horizontal_bar(
                data::category(&bars, "category"),
                data::numeric(&bars, "value"),
            )
        } else {
            Plot::bar(
                data::category(&bars, "category"),
                data::numeric(&bars, "value"),
            )
        };
        svg(
            plot.group(data::category(&bars, "group"))
                .bar_aggregation(BarAggregation::Identity)
                .bar_point_layer(layer()),
            if horizontal {
                "horizontal-bar-point-layer"
            } else {
                "vertical-bar-point-layer"
            },
        )
    };

    for output in [render(false), render(true)] {
        assert_eq!(
            output
                .lines()
                .filter(|line| line.starts_with("<circle ")
                    && line.contains("fill=\"#7B2DC8\"")
                    && line.contains("r=\"5\"")
                    && line.contains("fill-opacity=\"0.4\""))
                .count(),
            3
        );
    }
}

#[test]
fn unknown_bar_point_category_rejects_during_preflight() {
    let bars = df!("category" => ["a"], "value" => [2.0_f64]).unwrap();
    let points = df!("category" => ["missing"], "value" => [3.0_f64]).unwrap();
    let layer = BarPointLayer::new(
        data::category(&points, "category"),
        None,
        data::numeric(&points, "value"),
        (1, 2, 3),
        3,
        0.5,
    )
    .unwrap();
    let plot = Plot::bar(
        data::category(&bars, "category"),
        data::numeric(&bars, "value"),
    )
    .bar_aggregation(BarAggregation::Identity)
    .bar_point_layer(layer);

    let error = plot.preflight().unwrap_err();

    assert!(error.to_string().contains("unknown category"));
}

#[test]
fn bar_point_layers_reject_nonfinite_mismatch_stacking_other_kinds_and_facets() {
    let bars = df!(
        "category" => ["a", "b"],
        "group" => ["left", "left"],
        "value" => [2.0_f64, 3.0],
        "facet" => ["one", "two"],
    )
    .unwrap();
    let nonfinite_points = df!("category" => ["a"], "value" => [f64::NAN]).unwrap();
    let grouped_points = df!(
        "category" => ["a"],
        "group" => ["left"],
        "value" => [2.5_f64],
    )
    .unwrap();
    let nonfinite = BarPointLayer::new(
        data::category(&nonfinite_points, "category"),
        None,
        data::numeric(&nonfinite_points, "value"),
        (1, 2, 3),
        3,
        0.5,
    )
    .unwrap();
    let grouped = BarPointLayer::new(
        data::category(&grouped_points, "category"),
        Some(data::category(&grouped_points, "group")),
        data::numeric(&grouped_points, "value"),
        (1, 2, 3),
        3,
        0.5,
    )
    .unwrap();
    let nonfinite_plot = Plot::bar(
        data::category(&bars, "category"),
        data::numeric(&bars, "value"),
    )
    .bar_aggregation(BarAggregation::Identity)
    .bar_point_layer(nonfinite);
    let mismatch_plot = Plot::bar(
        data::category(&bars, "category"),
        data::numeric(&bars, "value"),
    )
    .group(data::category(&bars, "group"))
    .bar_aggregation(BarAggregation::Identity)
    .bar_point_layer(
        BarPointLayer::new(
            data::category(&grouped_points, "category"),
            None,
            data::numeric(&grouped_points, "value"),
            (1, 2, 3),
            3,
            0.5,
        )
        .unwrap(),
    );
    let stacked_plot = Plot::stacked_bar(
        data::category(&bars, "category"),
        data::numeric(&bars, "value"),
        data::category(&bars, "group"),
    )
    .bar_point_layer(grouped.clone());
    let scatter_plot = Plot::scatter(data::numeric(&bars, "value"), data::numeric(&bars, "value"))
        .bar_point_layer(grouped.clone());
    let faceted_plot = Plot::bar(
        data::category(&bars, "category"),
        data::numeric(&bars, "value"),
    )
    .group(data::category(&bars, "group"))
    .bar_aggregation(BarAggregation::Identity)
    .bar_point_layer(grouped)
    .facet_wrap(data::category(&bars, "facet"), 2);

    let cases = [
        (nonfinite_plot, "finite"),
        (mismatch_plot, "must match"),
        (stacked_plot, "ordinary Bar"),
        (scatter_plot, "ordinary Bar"),
        (faceted_plot, "faceting"),
    ];
    for (plot, expected) in cases {
        let error = plot.preflight().unwrap_err();
        assert!(error.to_string().contains(expected), "{error}");
    }
}

#[test]
fn three_star_limit_caps_planning_and_rendering_while_four_is_byte_inert() {
    let data = df!("category" => ["a", "b"], "value" => [2.0_f64, 4.0]).unwrap();
    let make = |limit: Option<SignificanceStarLimit>, name: &str| {
        let plot = Plot::bar(
            data::category(&data, "category"),
            data::numeric(&data, "value"),
        )
        .bar_aggregation(BarAggregation::Identity);
        let plot = limit.map_or(plot.clone(), |limit| plot.significance_star_limit(limit));
        svg(
            plot.add_comparisons([ComparisonAnnotation::new("a", "b", 0.00001)]),
            name,
        )
    };

    let default = make(None, "default-four-star-limit");
    let explicit_four = make(
        Some(SignificanceStarLimit::Four),
        "explicit-four-star-limit",
    );
    let capped = make(Some(SignificanceStarLimit::Three), "three-star-limit");

    assert_eq!(default, explicit_four);
    assert!(capped.contains(">***</text>") && !capped.contains(">****</text>"));
}

#[test]
fn canvas_supports_all_text_and_mixed_cells_without_text_legends() {
    let all_text = svg_canvas(
        Canvas::grid(1, 2)
            .unwrap()
            .text_panel(
                0,
                0,
                TextPanel::new("Context", "Deterministically wrapped prose.").unwrap(),
            )
            .unwrap()
            .text_panel(
                0,
                1,
                TextPanel::new("Result", "Centered explanatory text.").unwrap(),
            )
            .unwrap(),
        "all-text-canvas",
    );
    let data = df!("category" => ["a"], "value" => [2.0_f64]).unwrap();
    let plot = Plot::bar(
        data::category(&data, "category"),
        data::numeric(&data, "value"),
    )
    .bar_aggregation(BarAggregation::Identity);
    let mixed = svg_canvas(
        Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, plot)
            .unwrap()
            .text_panel(
                0,
                1,
                TextPanel::new("Interpretation", "One chart and one prose cell.").unwrap(),
            )
            .unwrap(),
        "mixed-content-canvas",
    );

    assert_eq!(all_text.matches("<svg ").count(), 1);
    assert!(all_text.contains("Context") && !all_text.contains("legend"));
    assert!(mixed.contains("Interpretation") && mixed.contains("<polygon "));
}

#[test]
fn canvas_text_cells_enforce_occupancy_and_fitting() {
    let panel = TextPanel::new("Title", "Body").unwrap();
    let occupied = Canvas::grid(1, 1)
        .unwrap()
        .text_panel(0, 0, panel.clone())
        .unwrap()
        .text_panel(0, 0, panel)
        .err()
        .unwrap();
    let too_long = TextPanel::new("Title", "word ".repeat(500).trim().to_owned()).unwrap();
    let fitting = Canvas::grid(1, 1)
        .unwrap()
        .text_panel(0, 0, too_long)
        .unwrap()
        .size(160, 120)
        .preflight()
        .unwrap_err();

    assert!(occupied.to_string().contains("already occupied"));
    assert!(fitting.to_string().contains("text panel"));
}

#[test]
fn all_text_canvas_renders_one_svg_document() {
    let canvas = Canvas::grid(1, 1)
        .unwrap()
        .text_panel(
            0,
            0,
            TextPanel::new("Summary", "One deterministic text cell.").unwrap(),
        )
        .unwrap();

    let output = svg_canvas(canvas, "single-text-canvas");

    assert_eq!(output.matches("<svg ").count(), 1);
}

fn svg_canvas(canvas: Canvas, name: &str) -> String {
    let directory = unique_dir(name);
    let output = directory.join("canvas.svg");
    canvas.svg(&output).unwrap();
    let svg = std::fs::read_to_string(output).unwrap();
    std::fs::remove_dir_all(directory).unwrap();
    svg
}
