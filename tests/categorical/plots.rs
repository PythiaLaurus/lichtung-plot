use super::*;

#[test]
fn violin_auto_range_includes_kde_support_beyond_raw_counts() {
    let genotype = CategoryValues::new(
        "genotype",
        ["aa", "aa", "aa", "Aa", "Aa", "Aa", "AA", "AA", "AA"],
    )
    .unwrap();
    let count = NumericValues::new(
        "count",
        [25.0, 30.0, 80.0, 50.0, 50.0, 5.0, 25.0, 20.0, 15.0],
    );

    let dir = unique_dir("violin-kde-support-range");
    let out = dir.join("violin.svg");
    let chart = Plot::violin(genotype, count)
        .labels(Labels::raw())
        .size(420, 300);
    chart
        .svg_preserve_text(&out)
        .expect("violin render must succeed");
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
fn bar_with_series_renders_grouped_bars_and_series_legend() {
    let group = CategoryValues::new("group", ["A", "A", "B", "B"]).unwrap();
    let series =
        CategoryValues::new("series", ["control", "treatment", "control", "treatment"]).unwrap();
    let value = NumericValues::new("value", [1.0, 3.0, 2.0, 5.0]);

    let dir = unique_dir("bar-series-grouped");
    let out = dir.join("bar.svg");
    let chart = Plot::bar(group, BarInput::estimates(value)).series(series);
    chart
        .size(420, 300)
        .svg_preserve_text(&out)
        .expect("bar render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");
    let bar_count = parse_polygon_bounds(&svg).len();

    assert!(has_legend(&svg), "series bar should draw a legend");
    assert!(
        svg.contains(">Control</text>") && svg.contains(">Treatment</text>"),
        "series labels should be rendered in the legend"
    );
    assert_eq!(
        bar_count, 4,
        "2 categories × 2 series levels should draw 4 bars"
    );
}

#[test]
fn stacked_bar_sums_duplicate_category_series_rows() {
    let group = CategoryValues::new("group", ["A", "A", "A"]).unwrap();
    let series = CategoryValues::new("series", ["first", "first", "second"]).unwrap();
    let value = NumericValues::new("value", [2.0, 3.0, 4.0]);

    let dir = unique_dir("stacked-bar-duplicate-sum");
    let out = dir.join("stacked_bar.svg");
    let chart = Plot::stacked_bar(group, value, series).size(360, 260);
    chart
        .svg_preserve_text(&out)
        .expect("stacked bar render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");
    let bars = parse_polygon_bounds(&svg);

    assert_eq!(
        bars.len(),
        2,
        "duplicates should aggregate to one segment per series"
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
    let species = CategoryValues::new("species", ["b", "a", "c"]).unwrap();
    let value = NumericValues::new("value", [2.0, 1.0, 4.0]);

    let dir = unique_dir("bar-observed-order");
    let out = dir.join("bar.svg");
    Plot::bar(species, BarInput::estimates(value))
        .size(320, 240)
        .svg_preserve_text(&out)
        .expect("bar render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");

    let b_pos = svg.find(">B</text>").expect("b tick label");
    let a_pos = svg.find(">A</text>").expect("a tick label");
    let c_pos = svg.find(">C</text>").expect("c tick label");
    assert!(b_pos < a_pos && a_pos < c_pos);
}

#[test]
fn bar_order_is_plot_owned_and_value_sort_runs_after_mean_aggregation() {
    let categories = CategoryValues::new("species", ["b", "b", "a", "a", "c", "c"]).unwrap();
    let values = NumericValues::new("value", [10.0, 0.0, 1.0, 1.0, 8.0, 8.0]);
    let dir = unique_dir("bar-plot-owned-order");
    let explicit_path = dir.join("explicit.svg");
    let sorted_path = dir.join("sorted.svg");

    Plot::bar(categories.clone(), BarInput::samples(values.clone()))
        .category_order(CategoryOrder::explicit(["c", "a", "b"]))
        .size(320, 240)
        .svg_preserve_text(&explicit_path)
        .unwrap();
    Plot::bar(categories, BarInput::samples(values))
        .category_order(ValueSort::Descending)
        .size(320, 240)
        .svg_preserve_text(&sorted_path)
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
    let category = CategoryValues::new("category", ["a", "a", "b", "b"]).unwrap();
    let group = CategoryValues::new("group", ["one", "two", "one", "two"]).unwrap();
    let value = NumericValues::new("value", [1.0, 2.0, 3.0, 4.0]);

    let error = Plot::bar(category, BarInput::estimates(value))
        .series(group)
        .category_order(ValueSort::Descending)
        .preflight()
        .unwrap_err()
        .to_string();

    assert!(error.contains("requires one ungrouped bar per category"));
}

#[test]
fn positive_bars_preserve_zero_baseline() {
    let group = CategoryValues::new("group", ["a", "b", "c"]).unwrap();
    let value = NumericValues::new("value", [1.0, 2.0, 3.0]);

    let dir = unique_dir("bar-zero-baseline");
    let out = dir.join("bar.svg");
    let chart = Plot::bar(group, BarInput::estimates(value)).size(320, 240);
    chart
        .svg_preserve_text(&out)
        .expect("bar render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");
    let panel = parse_first_clip_rect(&svg);
    let bar_bounds = parse_polygon_bounds(&svg);
    let panel_bottom = panel.y + panel.height;

    assert!(
        !bar_bounds.is_empty()
            && bar_bounds
                .iter()
                .all(|bounds| ((bounds.y + bounds.height) - panel_bottom).abs() < 1.0),
        "positive bars should start exactly from the zero baseline: panel={panel:?}, bounds={bar_bounds:?}"
    );
}
