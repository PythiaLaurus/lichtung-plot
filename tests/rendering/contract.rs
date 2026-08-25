use lichtung_plot::{
    BarInput, BarPointLayer, Bounds, Canvas, CategoryValues, FacetSpec, NumericValues, Plot,
    TextPanel,
};

use super::{unique_dir, Significance, SignificanceMark};

struct IntervalData {
    category: CategoryValues,
    value: NumericValues,
    lower: NumericValues,
    upper: NumericValues,
}

fn identity_interval_data() -> IntervalData {
    IntervalData {
        category: CategoryValues::new("category", ["a", "b"]).unwrap(),
        value: NumericValues::new("value", [2.0, 5.0]),
        lower: NumericValues::new("lower", [1.0, 3.5]),
        upper: NumericValues::new("upper", [4.0, 8.0]),
    }
}

fn identity_interval_plot(data: &IntervalData, horizontal: bool) -> Plot {
    if horizontal {
        Plot::horizontal_bar(
            data.category.clone(),
            BarInput::estimates_with_error_bars(
                data.value.clone(),
                Bounds::new(data.lower.clone(), data.upper.clone()),
            ),
        )
        .erase()
    } else {
        Plot::bar(
            data.category.clone(),
            BarInput::estimates_with_error_bars(
                data.value.clone(),
                Bounds::new(data.lower.clone(), data.upper.clone()),
            ),
        )
        .erase()
    }
}

fn svg<K: lichtung_plot::kind::Marker>(plot: Plot<K>, name: &str) -> String {
    let directory = unique_dir(name);
    let output = directory.join("plot.svg");
    plot.svg(&output).unwrap();
    let svg = std::fs::read_to_string(output).unwrap();
    std::fs::remove_dir_all(directory).unwrap();
    svg
}

#[test]
fn precomputed_bar_bounds_render_asymmetric_endpoints() {
    let data = identity_interval_data();
    let vertical = svg(
        identity_interval_plot(&data, false),
        "vertical-identity-interval",
    );
    let horizontal = svg(
        identity_interval_plot(&data, true),
        "horizontal-identity-interval",
    );

    let plain = svg(
        Plot::bar(
            data.category.clone(),
            BarInput::estimates(data.value.clone()),
        ),
        "plain-identity-bar",
    );
    assert!(vertical.matches("<polyline ").count() > plain.matches("<polyline ").count());
    assert!(horizontal.matches("<polyline ").count() > plain.matches("<polyline ").count());
}

#[test]
fn identity_bar_rejects_invalid_endpoint_order_during_preflight() {
    let data = IntervalData {
        category: CategoryValues::new("category", ["a"]).unwrap(),
        value: NumericValues::new("value", [2.0]),
        lower: NumericValues::new("lower", [3.0]),
        upper: NumericValues::new("upper", [4.0]),
    };
    let error = identity_interval_plot(&data, false)
        .preflight()
        .unwrap_err();

    assert!(error.to_string().contains("lower <= value <= upper"));
}

#[test]
fn bar_point_layers_retain_rows_at_exact_grouped_centers_in_both_orientations() {
    let category = CategoryValues::new("category", ["a", "a", "b", "b"]).unwrap();
    let group = CategoryValues::new("group", ["left", "right", "left", "right"]).unwrap();
    let value = NumericValues::new("value", [2.0, 3.0, 4.0, 5.0]);
    let point_category = CategoryValues::new("point_category", ["a", "a", "b"]).unwrap();
    let point_group = CategoryValues::new("point_group", ["left", "right", "left"]).unwrap();
    let point_value = NumericValues::new("point_value", [1.0, 7.0, 6.0]);
    let layer = || {
        BarPointLayer::new(
            point_category.clone(),
            Some(point_group.clone()),
            point_value.clone(),
            (123, 45, 200),
            5,
            0.4,
        )
        .unwrap()
    };
    let render = |horizontal| {
        if horizontal {
            svg(
                Plot::horizontal_bar(category.clone(), BarInput::estimates(value.clone()))
                    .series(group.clone())
                    .point_layer(layer()),
                "horizontal-bar-point-layer",
            )
        } else {
            svg(
                Plot::bar(category.clone(), BarInput::estimates(value.clone()))
                    .series(group.clone())
                    .point_layer(layer()),
                "vertical-bar-point-layer",
            )
        }
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
    let layer = BarPointLayer::new(
        CategoryValues::new("category", ["missing"]).unwrap(),
        None,
        NumericValues::new("value", [3.0]),
        (1, 2, 3),
        3,
        0.5,
    )
    .unwrap();
    let plot = Plot::bar(
        CategoryValues::new("category", ["a"]).unwrap(),
        BarInput::estimates(NumericValues::new("value", [2.0])),
    )
    .point_layer(layer);

    let error = plot.preflight().unwrap_err();

    assert!(error.to_string().contains("unknown category"));
}

#[test]
fn bar_point_layers_reject_nonfinite_mismatch_and_facets() {
    let category = CategoryValues::new("category", ["a", "b"]).unwrap();
    let group = CategoryValues::new("group", ["left", "left"]).unwrap();
    let value = NumericValues::new("value", [2.0, 3.0]);
    let facet = CategoryValues::new("facet", ["one", "two"]).unwrap();
    let point_category = CategoryValues::new("category", ["a"]).unwrap();
    let point_group = CategoryValues::new("group", ["left"]).unwrap();
    let point_value = NumericValues::new("value", [2.5]);
    let nonfinite = BarPointLayer::new(
        point_category.clone(),
        None,
        NumericValues::new("value", [f64::NAN]),
        (1, 2, 3),
        3,
        0.5,
    )
    .unwrap();
    let grouped = BarPointLayer::new(
        point_category.clone(),
        Some(point_group),
        point_value.clone(),
        (1, 2, 3),
        3,
        0.5,
    )
    .unwrap();
    let nonfinite_plot =
        Plot::bar(category.clone(), BarInput::estimates(value.clone())).point_layer(nonfinite);
    let mismatch_plot = Plot::bar(category.clone(), BarInput::estimates(value.clone()))
        .series(group.clone())
        .point_layer(
            BarPointLayer::new(point_category, None, point_value, (1, 2, 3), 3, 0.5).unwrap(),
        );
    let faceted_plot = Plot::bar(category, BarInput::estimates(value))
        .series(group)
        .point_layer(grouped)
        .facet(FacetSpec::wrap(facet, 2).unwrap());

    let cases = [
        (nonfinite_plot, "finite"),
        (mismatch_plot, "must match"),
        (faceted_plot, "faceting"),
    ];
    for (plot, expected) in cases {
        let error = plot.preflight().unwrap_err();
        assert!(error.to_string().contains(expected), "{error}");
    }
}

#[test]
fn explicit_and_p_value_marks_render_the_same_four_stars() {
    let plot = Plot::bar(
        CategoryValues::new("category", ["a", "b"]).unwrap(),
        BarInput::estimates(NumericValues::new("value", [2.0, 4.0])),
    );
    let render = |mark, name| {
        svg(
            plot.clone()
                .significance([Significance::between("a", "b", mark)]),
            name,
        )
    };

    let explicit = render(SignificanceMark::FourStars, "explicit-four-stars");
    let derived = render(SignificanceMark::PValue(0.00001), "p-value-four-stars");

    assert_eq!(explicit, derived);
    assert!(explicit.contains(">****</text>"));
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
    let plot = Plot::bar(
        CategoryValues::new("category", ["a"]).unwrap(),
        BarInput::estimates(NumericValues::new("value", [2.0])),
    );
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
