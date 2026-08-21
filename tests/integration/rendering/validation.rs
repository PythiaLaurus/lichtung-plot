use super::*;

#[test]
fn categorical_plots_render_stacked_pairwise_brackets() {
    let df = df!(
        "group" => ["a", "a", "b", "b", "c", "c"],
        "value" => [1.0_f64, 1.2, 2.0, 2.2, 3.0, 3.2]
    )
    .unwrap();
    let comparisons = vec![
        ComparisonAnnotation::new("a", "b", 0.04),
        ComparisonAnnotation::new("b", "c", 0.005),
        ComparisonAnnotation::new("a", "c", 0.0005),
    ];

    for kind in ["bar", "boxplot", "violin"] {
        let dir = unique_dir(&format!("significance-{kind}"));
        let out = dir.join(format!("{kind}.svg"));
        let mut chart = Chart::new(&df);
        let _ = chart.x("group").y("value").size(360, 280);
        let _ = chart.add_comparisons(comparisons.clone());
        match kind {
            "bar" => {
                let _ = chart.bar();
            }
            "boxplot" => {
                let _ = chart.boxplot();
            }
            _ => {
                let _ = chart.violin();
            }
        }
        chart.render_to_svg(&out).expect("categorical render");
        let svg = std::fs::read_to_string(out).expect("read significance SVG");

        assert!(svg.contains(">*<"));
        assert!(svg.contains(">**<"));
        assert!(svg.contains(">***<"));
        assert_eq!(svg.matches("class=\"significance-bracket\"").count(), 3);
    }
}

#[test]
fn chinese_text_uses_system_font_fallback_stack_without_embedding() {
    let df = fixture_df();
    let dir = tempdir();
    let out = dir.join("chinese.svg");
    let mut chart = Chart::new(&df)
        .title("中文分析")
        .x_label("时间")
        .y_label("测量值");
    chart.x("x").y("y").scatter();

    chart.render_to_svg(&out).expect("render Chinese SVG");

    let svg = std::fs::read_to_string(out).expect("read Chinese SVG");
    assert!(svg.contains(&format!(
        "font-family=\"{}\"",
        lichtung_plot::DEFAULT_FONT_FAMILY_STACK
    )));
    assert!(svg.contains("中文分析"));
    assert!(!svg.contains("data:font/woff2;base64,"));
}

#[test]
fn point_style_constructs() {
    // Sanity: the types round-trip.
    let s = PointStyle::new((0, 114, 178), 4);
    let p = Point2D::new(1.0, 2.0);
    assert_eq!(s.radius, 4);
    assert!(s.outline.is_some());
    assert!((s.outline.unwrap().width_for_radius(s.radius) - 0.4).abs() < 1e-12);
    assert!((p.x - 1.0).abs() < 1e-12);
}

#[test]
fn scatter_points_use_white_outline_by_default() {
    let df = fixture_df();
    let dir = unique_dir("scatter-point-outline-default");
    let out = dir.join("scatter.svg");
    let mut chart = Chart::new(&df);
    let _ = chart.x("x").y("y").size(320, 240).scatter();
    chart
        .render_to_svg(&out)
        .expect("scatter render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");

    assert!(
        svg.contains("stroke=\"#FFFFFF\" stroke-width=\"0.3\""),
        "scatter points should have a thin white outline by default"
    );
}

#[test]
fn scatter_point_outline_can_be_disabled() {
    let df = fixture_df();
    let dir = unique_dir("scatter-point-outline-disabled");
    let out = dir.join("scatter.svg");
    let mut chart = Chart::new(&df).hide_point_outline();
    let _ = chart.x("x").y("y").size(320, 240).scatter();
    chart
        .render_to_svg(&out)
        .expect("scatter render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");

    assert!(
        !svg.contains("stroke=\"#FFFFFF\""),
        "hide_point_outline should render fill-only points"
    );
}

#[test]
fn render_rejects_invalid_explicit_axis_limits_centrally() {
    let df = df!(
        "x" => [1.0_f64, 2.0, 3.0],
        "y" => [2.0_f64, 3.0, 4.0]
    )
    .unwrap();
    for (index, (x_limits, y_limits)) in [
        ((1.0, 1.0), (0.0, 1.0)),
        ((2.0, 1.0), (0.0, 1.0)),
        ((f64::NAN, 1.0), (0.0, 1.0)),
        ((0.0, 1.0), (0.0, f64::INFINITY)),
    ]
    .into_iter()
    .enumerate()
    {
        let dir = unique_dir(&format!("invalid-axis-limits-{index}"));
        let output = dir.join("scatter.svg");
        let mut chart = Chart::new(&df)
            .x_limits(x_limits.0, x_limits.1)
            .y_limits(y_limits.0, y_limits.1);
        chart.x("x").y("y").scatter();

        let error = chart.render_to_svg(&output).unwrap_err().to_string();

        assert!(error.contains("limits must be finite and strictly ascending"));
        std::fs::remove_dir_all(dir).unwrap();
    }
}

#[test]
fn waterfall_validates_terminal_total() {
    let df = df!(
        "label" => ["a", "b"],
        "delta" => [2.0_f64, -1.0],
        "base" => [10.0_f64, 10.0],
        "terminal" => [12.0_f64, 12.0]
    )
    .unwrap();
    let dir = unique_dir("waterfall-total-validation");
    let out = dir.join("waterfall.svg");
    let mut chart = Chart::new(&df).waterfall_totals("base", "terminal");
    chart.x("label").y("delta").waterfall();
    let error = chart.render_to_svg(&out).unwrap_err();
    assert!(error.to_string().contains("base plus deltas"));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn signed_horizontal_bar_supports_precomputed_sort_and_zero_line() {
    let df = df!(
        "label" => ["neutral", "positive", "negative"],
        "value" => [0.1_f64, 2.0, -1.0]
    )
    .unwrap();
    let dir = unique_dir("signed-horizontal-bar");
    let out = dir.join("bar.svg");
    let mut chart = Chart::new(&df)
        .bar_aggregation(BarAggregation::Identity)
        .value_sort(ValueSort::Descending)
        .color_by_sign()
        .zero_line();
    chart.x("label").y("value").horizontal_bar();
    chart.render_to_svg(&out).unwrap();
    assert!(std::fs::read_to_string(&out).unwrap().contains("Negative"));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn scatter_hue_jointly_drops_null_and_nan_pairs_without_panic() {
    let df = df!(
        "x" => [Some(1.0_f64), None, Some(f64::NAN), Some(4.0)],
        "y" => [Some(2.0_f64), Some(3.0), Some(4.0), Some(5.0)],
        "group" => ["a", "a", "b", "b"]
    )
    .unwrap();

    let dir = unique_dir("scatter-hue-invalid-pairs");
    let out = dir.join("scatter.svg");
    let mut chart = Chart::new(&df).hue("group");
    let _ = chart.x("x").y("y").size(320, 240).scatter();
    chart
        .render_to_svg(&out)
        .expect("scatter render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");
    assert!(svg.contains("Group"), "legend title should be rendered");
}
