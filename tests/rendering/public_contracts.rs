use super::*;

fn render<K: lichtung_plot::kind::Marker>(plot: Plot<K>, name: &str) -> String {
    let directory = unique_dir(name);
    let output = directory.join("plot.svg");
    let scene = plot.svg_preserve_text(&output).unwrap();
    let svg = std::fs::read_to_string(output).unwrap();
    assert!(svg.contains(&format!(
        "width=\"{}\" height=\"{}\"",
        scene.width(),
        scene.height()
    )));
    std::fs::remove_dir_all(directory).unwrap();
    svg
}

#[test]
fn erased_plot_preserves_selected_theme_text_dimensions_and_output() {
    use lichtung_plot::{PlotKind, Theme};

    let (x, y) = fixture_values();
    let alpha = lichtung_plot::AlphaValues::new(
        "confidence",
        (0..x.len()).map(|index| [0, 128, 230][index % 3]),
    )
    .unwrap();
    let exact = Plot::scatter(x, y)
        .point_alpha(lichtung_plot::AlphaEncoding::observations(alpha))
        .theme(Theme::nature())
        .size(321, 234)
        .title("Erasure parity");

    assert_eq!(exact.kind(), PlotKind::Scatter);
    let exact_svg = render(exact.clone(), "exact-before-erasure");
    assert!(exact_svg.contains(">Erasure parity</text>"));
    assert!(exact_svg.contains("width=\"321\" height=\"234\""));

    let erased: Plot = exact.erase();
    assert_eq!(erased.kind(), PlotKind::Scatter);
    erased.preflight().unwrap();

    assert_eq!(render(erased, "erased-terminal-output"), exact_svg);
}

#[test]
fn scatter_point_outline_can_be_disabled() {
    let (x, y) = fixture_values();
    let svg = render(
        Plot::scatter(x, y).hide_point_outline().size(320, 240),
        "scatter-point-outline-disabled",
    );

    assert!(
        !svg.contains("stroke=\"#FFFFFF\""),
        "hide_point_outline should render fill-only points"
    );
}

#[test]
fn preflight_rejects_invalid_explicit_axis_limits_centrally() {
    let x = NumericValues::new("x", [1.0, 2.0, 3.0]);
    let y = NumericValues::new("y", [2.0, 3.0, 4.0]);
    for (x_limits, y_limits) in [
        ((1.0, 1.0), (0.0, 1.0)),
        ((2.0, 1.0), (0.0, 1.0)),
        ((f64::NAN, 1.0), (0.0, 1.0)),
        ((0.0, 1.0), (0.0, f64::INFINITY)),
    ] {
        let plot = Plot::scatter(x.clone(), y.clone())
            .x_limits(x_limits.0, x_limits.1)
            .y_limits(y_limits.0, y_limits.1);

        let error = plot.preflight().unwrap_err().to_string();

        assert!(error.contains("limits must be finite and strictly ascending"));
    }
}

#[test]
fn waterfall_rejects_terminal_geometry_below_coordinate_precision() {
    let plot = Plot::waterfall(
        CategoryValues::new("label", ["a", "b"]).unwrap(),
        NumericValues::new("delta", [-10_000_000_000_000_000.0, 1.0]),
        10_000_000_000_000_000.0,
    );

    let error = plot.preflight().unwrap_err();

    assert!(matches!(error, lichtung_plot::Error::Render(_)), "{error}");
    assert!(error.to_string().contains("polygon"), "{error}");
    assert!(error.to_string().contains("collapse"), "{error}");
}

#[test]
fn waterfall_accepts_finite_ordered_geometry_when_detached_delta_sum_overflows() {
    // Two successive deltas overflow as a detached sum; ordered endpoints
    // remain finite. The broad exact window also gives the required
    // 300-digit total label a containing shelf without numeric-edge clipping.
    let span = f64::MAX * 0.6;
    let plot = Plot::waterfall(
        CategoryValues::new("label", ["a", "b", "c"]).unwrap(),
        NumericValues::new("delta", [span, span, -span]),
        -span,
    )
    .size(16_000, 300)
    .x_limits(-f64::MAX * 0.9, f64::MAX * 0.9);

    plot.preflight().unwrap();
}

#[test]
fn waterfall_rejects_nonfinite_intermediate_endpoint_with_source_row() {
    let plot = Plot::waterfall(
        CategoryValues::new("label", ["a", "b"]).unwrap(),
        NumericValues::new("delta", [f64::MAX, f64::MAX]),
        0.0,
    );

    assert_eq!(
        plot.preflight().unwrap_err().to_string(),
        "data error: waterfall: cumulative endpoint is non-finite at source row 1"
    );
}

#[test]
fn signed_horizontal_bar_supports_precomputed_sort_and_zero_line() {
    let plot = Plot::horizontal_bar(
        CategoryValues::new("label", ["neutral", "positive", "negative"]).unwrap(),
        BarInput::estimates(NumericValues::new("value", [0.1, 2.0, -1.0])),
    )
    .category_order(ValueSort::Descending)
    .color_by_sign()
    .zero_line();
    let svg = render(plot.clone(), "signed-horizontal-bar");
    assert!(svg.contains("Negative"));
    let directory = unique_dir("shared-signed-horizontal-bar");
    let path = directory.join("plot.svg");
    lichtung_plot::Canvas::grid(1, 2)
        .unwrap()
        .size(1000, 500)
        .panel(0, 0, plot.clone())
        .unwrap()
        .panel(0, 1, plot)
        .unwrap()
        .legend_policy(lichtung_plot::CanvasLegendPolicy::Shared)
        .svg_preserve_text(&path)
        .unwrap();
    let shared = std::fs::read_to_string(&path).unwrap();
    std::fs::remove_dir_all(directory).unwrap();
    for output in [&svg, &shared] {
        for source in ["Change", "Nondecreasing", "Decreasing"] {
            assert_eq!(output.matches(&format!(">{source}</text>")).count(), 1);
        }
    }
}
