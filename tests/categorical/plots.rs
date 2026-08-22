use super::*;

#[test]
fn violin_auto_range_includes_kde_support_beyond_raw_counts() {
    let df = df!(
        "genotype" => ["aa", "aa", "aa", "Aa", "Aa", "Aa", "AA", "AA", "AA"],
        "count" => [25.0_f64, 30.0, 80.0, 50.0, 50.0, 5.0, 25.0, 20.0, 15.0]
    )
    .unwrap();

    let dir = unique_dir("violin-kde-support-range");
    let out = dir.join("violin.svg");
    let chart = Plot::violin(data::category(&df, "genotype"), data::numeric(&df, "count"))
        .labels(Labels::raw())
        .size(420, 300);
    chart.svg(&out).expect("violin render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");
    let numeric_ticks: Vec<f64> = parse_text_elements(&svg)
        .into_iter()
        .filter_map(|text| text.content.parse::<f64>().ok())
        .collect();
    let min_tick = numeric_ticks.iter().copied().fold(f64::INFINITY, f64::min);
    let max_tick = numeric_ticks
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);

    assert!(
        min_tick < 0.0 && max_tick > 105.0,
        "violin y-axis should include KDE support beyond raw 5..80 counts: ticks={numeric_ticks:?}"
    );
}

#[test]
fn bar_with_hue_renders_grouped_bars_and_hue_legend() {
    let df = df!(
        "group" => ["A", "A", "B", "B"],
        "series" => ["control", "treatment", "control", "treatment"],
        "value" => [1.0_f64, 3.0, 2.0, 5.0]
    )
    .unwrap();

    let dir = unique_dir("bar-hue-grouped");
    let out = dir.join("bar.svg");
    let chart = Plot::bar(data::category(&df, "group"), data::numeric(&df, "value"))
        .group(data::category(&df, "series"))
        .hue(data::category(&df, "series"))
        .bar_aggregation(BarAggregation::Identity);
    chart
        .size(420, 300)
        .svg(&out)
        .expect("bar render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");
    let bar_count = parse_polygon_bounds(&svg).len();

    assert!(has_legend(&svg), "hue bar should draw a legend");
    assert!(
        svg.contains(">Control</text>") && svg.contains(">Treatment</text>"),
        "hue labels should be rendered in the legend"
    );
    assert_eq!(
        bar_count, 4,
        "2 categories × 2 hue levels should draw 4 bars"
    );
}

#[test]
fn grouped_bar_comfort_spacing_keeps_internal_gap() {
    let gap = grouped_bar_internal_gap(BarSpacing::Comfort);
    assert!(
        gap > 2.0,
        "comfort grouped bars should keep a visible internal gap, got {gap}"
    );
}

#[test]
fn grouped_bar_compact_spacing_places_internal_items_side_by_side() {
    let gap = grouped_bar_internal_gap(BarSpacing::Compact);
    assert!(
        gap <= 1.0,
        "compact grouped bars should touch or nearly touch, got {gap}"
    );
}

#[test]
fn horizontal_bar_with_hue_renders_grouped_bars_and_hue_legend() {
    let df = df!(
        "group" => ["A", "A", "B", "B"],
        "series" => ["control", "treatment", "control", "treatment"],
        "value" => [1.0_f64, 3.0, 2.0, 5.0]
    )
    .unwrap();

    let dir = unique_dir("horizontal-bar-hue-grouped");
    let out = dir.join("horizontal_bar.svg");
    let chart = Plot::horizontal_bar(data::category(&df, "group"), data::numeric(&df, "value"))
        .group(data::category(&df, "series"))
        .hue(data::category(&df, "series"))
        .bar_aggregation(BarAggregation::Identity);
    chart
        .size(420, 300)
        .svg(&out)
        .expect("horizontal bar render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");
    let bar_count = parse_polygon_bounds(&svg).len();

    assert!(has_legend(&svg), "hue horizontal bar should draw a legend");
    assert_eq!(
        bar_count, 4,
        "2 categories × 2 hue levels should draw 4 bars"
    );
}

#[test]
fn stacked_bar_sums_duplicate_category_hue_rows() {
    let df = df!(
        "group" => ["A", "A", "A"],
        "series" => ["first", "first", "second"],
        "value" => [2.0_f64, 3.0, 4.0]
    )
    .unwrap();

    let dir = unique_dir("stacked-bar-duplicate-sum");
    let out = dir.join("stacked_bar.svg");
    let chart = Plot::stacked_bar(
        data::category(&df, "group"),
        data::numeric(&df, "value"),
        data::category(&df, "series"),
    )
    .hue(data::category(&df, "series"))
    .size(360, 260);
    chart.svg(&out).expect("stacked bar render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");
    let bars = parse_polygon_bounds(&svg);

    assert_eq!(
        bars.len(),
        2,
        "duplicates should aggregate to one segment per hue"
    );
    assert!(
        bars.iter().any(|a| bars
            .iter()
            .any(|b| { !std::ptr::eq(a, b) && (a.y - (b.y + b.height)).abs() < 3.0 })),
        "stacked segments should share the summed boundary: {bars:?}"
    );
}

#[test]
fn categorical_plots_preserve_observed_order() {
    let df = df!(
        "species" => ["b", "a", "c"],
        "value" => [2.0_f64, 1.0, 4.0]
    )
    .unwrap();

    let dir = unique_dir("bar-observed-order");
    let out = dir.join("bar.svg");
    Plot::bar(data::category(&df, "species"), data::numeric(&df, "value"))
        .bar_aggregation(BarAggregation::Identity)
        .size(320, 240)
        .svg(&out)
        .expect("bar render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");

    let b_pos = svg.find(">B</text>").expect("b tick label");
    let a_pos = svg.find(">A</text>").expect("a tick label");
    let c_pos = svg.find(">C</text>").expect("c tick label");
    assert!(b_pos < a_pos && a_pos < c_pos);
}

#[test]
fn bar_order_is_plot_owned_and_value_sort_runs_after_mean_aggregation() {
    let df = df!(
        "species" => ["b", "b", "a", "a", "c", "c"],
        "value" => [10.0_f64, 0.0, 1.0, 1.0, 8.0, 8.0]
    )
    .unwrap();
    let categories = data::category(&df, "species");
    let values = data::numeric(&df, "value");
    let dir = unique_dir("bar-plot-owned-order");
    let explicit_path = dir.join("explicit.svg");
    let sorted_path = dir.join("sorted.svg");

    Plot::bar(categories.clone(), values.clone())
        .category_order(CategoryOrder::explicit(["c", "a", "b"]))
        .size(320, 240)
        .svg(&explicit_path)
        .unwrap();
    Plot::bar(categories, values)
        .category_order(ValueSort::Descending)
        .size(320, 240)
        .svg(&sorted_path)
        .unwrap();

    let explicit = std::fs::read_to_string(explicit_path).unwrap();
    let sorted = std::fs::read_to_string(sorted_path).unwrap();
    assert!(explicit.find(">C</text>") < explicit.find(">A</text>"));
    assert!(explicit.find(">A</text>") < explicit.find(">B</text>"));
    assert!(sorted.find(">C</text>") < sorted.find(">B</text>"));
    assert!(sorted.find(">B</text>") < sorted.find(">A</text>"));
}

#[test]
fn grouped_bar_rejects_ambiguous_value_order() {
    let df = df!(
        "category" => ["a", "a", "b", "b"],
        "group" => ["one", "two", "one", "two"],
        "value" => [1.0_f64, 2.0, 3.0, 4.0]
    )
    .unwrap();

    let error = Plot::bar(data::category(&df, "category"), data::numeric(&df, "value"))
        .group(data::category(&df, "group"))
        .bar_aggregation(BarAggregation::Identity)
        .category_order(ValueSort::Descending)
        .preflight()
        .unwrap_err()
        .to_string();

    assert!(error.contains("requires one ungrouped bar per category"));
}

#[test]
fn positive_bars_preserve_zero_baseline() {
    let df = df!(
        "group" => ["a", "b", "c"],
        "value" => [1.0_f64, 2.0, 3.0]
    )
    .unwrap();

    let dir = unique_dir("bar-zero-baseline");
    let out = dir.join("bar.svg");
    let chart = Plot::bar(data::category(&df, "group"), data::numeric(&df, "value"))
        .bar_aggregation(BarAggregation::Identity)
        .size(320, 240);
    let panel = chart.figure_layout().panel_box;
    chart.svg(&out).expect("bar render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");
    let bar_bounds = parse_polygon_bounds(&svg);
    let panel_bottom = 240.0 - panel.y0;

    assert!(
        !bar_bounds.is_empty()
            && bar_bounds
                .iter()
                .all(|bounds| ((bounds.y + bounds.height) - panel_bottom).abs() < 1.0),
        "positive bars should start exactly from the zero baseline: panel={panel:?}, bounds={bar_bounds:?}"
    );
}

#[test]
fn boxplot_renders_fliers_by_default() {
    let df = df!(
        "group" => ["a", "a", "a", "a", "a", "b", "b", "b", "b", "b"],
        "value" => [1.0_f64, 1.0, 1.0, 1.0, 10.0, 2.0, 2.1, 2.2, 2.3, 2.4]
    )
    .unwrap();

    let dir = unique_dir("boxplot-fliers");
    let out = dir.join("boxplot.svg");
    Plot::boxplot(data::category(&df, "group"), data::numeric(&df, "value"))
        .size(360, 260)
        .svg(&out)
        .expect("boxplot render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");
    let circles = parse_circles(&svg);

    assert!(
        !circles.is_empty(),
        "boxplot should render outlier/fliers as point markers"
    );
}

#[test]
fn publication_frame_uses_axis_color_on_all_sides() {
    let df = fixture_df();
    let dir = unique_dir("frame-axis-color");
    let out = dir.join("scatter.svg");
    Plot::scatter(data::numeric(&df, "x"), data::numeric(&df, "y"))
        .size(320, 240)
        .svg(&out)
        .expect("scatter render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");

    assert!(svg.contains("stroke=\"#000000\" stroke-width=\"0.8\""));
    assert!(
        !svg.contains("stroke=\"#DCDCDC\" stroke-width=\"1\""),
        "top/right frame should not use the grid color"
    );
}

#[test]
fn violin_draws_inner_whisker_mark() {
    let df = df!(
        "group" => ["a", "a", "a", "a", "a", "b", "b", "b", "b", "b"],
        "value" => [1.0_f64, 1.2, 1.5, 1.8, 2.0, 3.0, 3.2, 3.5, 3.8, 4.0]
    )
    .unwrap();

    let dir = unique_dir("violin-inner-whisker");
    let out = dir.join("violin.svg");
    Plot::violin(data::category(&df, "group"), data::numeric(&df, "value"))
        .size(360, 260)
        .svg(&out)
        .expect("violin render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");

    let black_lines = svg
        .matches("stroke=\"#000000\" stroke-width=\"0.8\"")
        .count();
    assert!(
        black_lines >= 6,
        "violin should render whisker stems and caps, got {black_lines} black axis-width lines"
    );
}

#[test]
fn violin_body_does_not_touch_frame_edges() {
    let df = df!(
        "group" => ["a", "a", "a", "a", "a", "b", "b", "b", "b", "b"],
        "value" => [1.0_f64, 1.2, 1.5, 1.8, 2.0, 3.0, 3.2, 3.5, 3.8, 4.0]
    )
    .unwrap();

    let dir = unique_dir("violin-body-frame-padding");
    let out = dir.join("violin.svg");
    Plot::violin(data::category(&df, "group"), data::numeric(&df, "value"))
        .size(360, 260)
        .svg(&out)
        .expect("violin render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");

    let first_polygon = svg
        .split("<polygon points=\"")
        .nth(1)
        .and_then(|s| s.split_once('"'))
        .map(|(points, _)| points)
        .expect("violin body polygon");

    assert!(
        !first_polygon.contains(",30.000") && !first_polygon.contains(",200.000"),
        "violin body should end inside the frame, not on the top/bottom axes"
    );
}
