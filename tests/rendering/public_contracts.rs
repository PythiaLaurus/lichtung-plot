use super::*;

fn render<K: lichtung_plot::kind::Marker>(plot: Plot<K>, name: &str) -> String {
    let directory = unique_dir(name);
    let output = directory.join("plot.svg");
    plot.svg(&output).unwrap();
    let svg = std::fs::read_to_string(output).unwrap();
    std::fs::remove_dir_all(directory).unwrap();
    svg
}

#[test]
fn categorical_plots_render_stacked_pairwise_brackets() {
    let df = df!(
        "group" => ["a", "a", "b", "b", "c", "c"],
        "value" => [1.0_f64, 1.2, 2.0, 2.2, 3.0, 3.2]
    )
    .unwrap();
    let comparisons = vec![
        Significance::between("a", "b", SignificanceMark::PValue(0.04)),
        Significance::between("b", "c", SignificanceMark::PValue(0.005)),
        Significance::between("a", "c", SignificanceMark::PValue(0.0005)),
    ];

    for kind in ["bar", "boxplot", "violin"] {
        let plot = match kind {
            "bar" => Plot::bar(
                data::category(&df, "group"),
                BarInput::samples(data::numeric(&df, "value")),
            )
            .significance(comparisons.clone())
            .size(360, 280)
            .erase(),
            "boxplot" => Plot::boxplot(data::category(&df, "group"), data::numeric(&df, "value"))
                .significance(comparisons.clone())
                .size(360, 280)
                .erase(),
            _ => Plot::violin(data::category(&df, "group"), data::numeric(&df, "value"))
                .significance(comparisons.clone())
                .size(360, 280)
                .erase(),
        };
        let svg = render(plot, &format!("significance-{kind}"));

        assert!(svg.contains(">*<"));
        assert!(svg.contains(">**<"));
        assert!(svg.contains(">***<"));
        assert_eq!(svg.matches("class=\"significance-bracket\"").count(), 3);
    }
}

#[test]
fn chinese_text_uses_system_font_fallback_stack_without_embedding() {
    let df = fixture_df();
    let svg = render(
        Plot::scatter(data::numeric(&df, "x"), data::numeric(&df, "y"))
            .title("中文分析")
            .x_label("时间")
            .y_label("测量值"),
        "chinese-text",
    );

    assert!(svg.contains(&format!(
        "font-family=\"{}\"",
        lichtung_plot::DEFAULT_FONT_FAMILY_STACK
    )));
    assert!(svg.contains("中文分析"));
    assert!(!svg.contains("data:font/woff2;base64,"));
}

#[test]
fn scatter_points_use_white_outline_by_default() {
    let df = fixture_df();
    let svg = render(
        Plot::scatter(data::numeric(&df, "x"), data::numeric(&df, "y")).size(320, 240),
        "scatter-point-outline-default",
    );

    assert!(
        svg.contains("stroke=\"#FFFFFF\" stroke-width=\"0.3\""),
        "scatter points should have a thin white outline by default"
    );
}

#[test]
fn scatter_point_outline_can_be_disabled() {
    let df = fixture_df();
    let svg = render(
        Plot::scatter(data::numeric(&df, "x"), data::numeric(&df, "y"))
            .hide_point_outline()
            .size(320, 240),
        "scatter-point-outline-disabled",
    );

    assert!(
        !svg.contains("stroke=\"#FFFFFF\""),
        "hide_point_outline should render fill-only points"
    );
}

#[test]
fn preflight_rejects_invalid_explicit_axis_limits_centrally() {
    let df = df!(
        "x" => [1.0_f64, 2.0, 3.0],
        "y" => [2.0_f64, 3.0, 4.0]
    )
    .unwrap();
    for (x_limits, y_limits) in [
        ((1.0, 1.0), (0.0, 1.0)),
        ((2.0, 1.0), (0.0, 1.0)),
        ((f64::NAN, 1.0), (0.0, 1.0)),
        ((0.0, 1.0), (0.0, f64::INFINITY)),
    ] {
        let plot = Plot::scatter(data::numeric(&df, "x"), data::numeric(&df, "y"))
            .x_limits(x_limits.0, x_limits.1)
            .y_limits(y_limits.0, y_limits.1);

        let error = plot.preflight().unwrap_err().to_string();

        assert!(error.contains("limits must be finite and strictly ascending"));
    }
}

#[test]
fn waterfall_terminal_follows_ordered_cancellation() {
    let df = df!(
        "label" => ["a", "b"],
        "delta" => [-10_000_000_000_000_000.0_f64, 1.0]
    )
    .unwrap();
    let plot = Plot::waterfall(
        data::category(&df, "label"),
        data::numeric(&df, "delta"),
        10_000_000_000_000_000.0,
    );

    plot.preflight().unwrap();
    assert!(render(plot, "waterfall-ordered-total").contains(">total 1.000</text>"));
}

#[test]
fn waterfall_accepts_finite_ordered_geometry_when_detached_delta_sum_overflows() {
    let df = df!(
        "label" => ["a", "b", "c"],
        "delta" => [f64::MAX, f64::MAX, -f64::MAX]
    )
    .unwrap();
    let plot = Plot::waterfall(
        data::category(&df, "label"),
        data::numeric(&df, "delta"),
        -f64::MAX,
    )
    .size(4_000, 240);

    plot.preflight().unwrap();
}

#[test]
fn waterfall_rejects_nonfinite_intermediate_endpoint_with_source_row() {
    let df = df!(
        "label" => ["a", "b"],
        "delta" => [f64::MAX, f64::MAX]
    )
    .unwrap();
    let plot = Plot::waterfall(
        data::category(&df, "label"),
        data::numeric(&df, "delta"),
        0.0,
    );

    assert_eq!(
        plot.preflight().unwrap_err().to_string(),
        "data error: waterfall: cumulative endpoint is non-finite at source row 1"
    );
}

#[test]
fn signed_horizontal_bar_supports_precomputed_sort_and_zero_line() {
    let df = df!(
        "label" => ["neutral", "positive", "negative"],
        "value" => [0.1_f64, 2.0, -1.0]
    )
    .unwrap();
    let svg = render(
        Plot::horizontal_bar(
            data::category(&df, "label"),
            BarInput::estimates(data::numeric(&df, "value")),
        )
        .category_order(ValueSort::Descending)
        .color_by_sign()
        .zero_line(),
        "signed-horizontal-bar",
    );

    assert!(svg.contains("Negative"));
}

#[test]
fn scatter_hue_jointly_drops_null_and_nan_pairs_without_panic() {
    let df = df!(
        "x" => [Some(1.0_f64), None, Some(f64::NAN), Some(4.0)],
        "y" => [Some(2.0_f64), Some(3.0), Some(4.0), Some(5.0)],
        "group" => ["a", "a", "b", "b"]
    )
    .unwrap();
    let svg = render(
        Plot::scatter(data::numeric(&df, "x"), data::numeric(&df, "y"))
            .hue(data::category(&df, "group"))
            .size(320, 240),
        "scatter-hue-invalid-pairs",
    );

    assert!(svg.contains("Group"), "legend title should be rendered");
}
