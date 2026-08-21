use super::*;

#[test]
fn faceting_uses_one_lifecycle_unique_clips_and_panel_owned_annotations() {
    let data = estimate_data();
    let comparison = ComparisonAnnotation::new("before", "after", 0.01).in_facet("facet", "one");
    let mut chart = Chart::new(&data)
        .facet_col("facet")
        .interval_bounds("lower", "upper")
        .interval_semantics(confidence(InferentialScale::Response))
        .interaction_source(EstimateSource::ObservedCellSummary)
        .group("trace")
        .hue("trace");
    chart
        .x("factor")
        .y("estimate")
        .add_comparisons([comparison])
        .interaction();
    // Interaction is not a bracket family, which protects the explicit-pairwise rule.
    assert!(chart
        .render_with_backend(&mut LifecycleBackend::default())
        .unwrap_err()
        .to_string()
        .contains("not supported"));

    let mut line = Chart::new(&data).facet_col("facet");
    line.x("x").y("estimate").line();
    let mut backend = LifecycleBackend::default();
    line.render_with_backend(&mut backend).unwrap();
    assert_eq!((backend.begins, backend.ends), (1, 1));
    assert_eq!((backend.clips, backend.clip_ends), (2, 2));

    let dir = tempdir("faceted-annotation");
    let output = dir.join("bar.svg");
    let mut bar = Chart::new(&data)
        .facet_col("facet")
        .facet_scales(FacetScales::Free)
        .bar_aggregation(BarAggregation::Identity);
    bar.x("factor")
        .y("estimate")
        .add_comparisons([ComparisonAnnotation::new("before", "after", 0.01)
            .in_facet("facet", "one")
            .in_facet("facet", "one")])
        .bar();
    bar.render_to_svg(&output).unwrap();
    assert_eq!(
        std::fs::read_to_string(output)
            .unwrap()
            .matches("class=\"significance-bracket\"")
            .count(),
        1
    );
}

#[test]
fn contradictory_duplicate_facet_ownership_fails_before_backend_begin() {
    let data = estimate_data();
    let comparison = ComparisonAnnotation::new("before", "after", 0.01)
        .in_facet("facet", "one")
        .in_facet("facet", "two");
    let mut chart = Chart::new(&data)
        .facet_col("facet")
        .bar_aggregation(BarAggregation::Identity);
    chart
        .x("factor")
        .y("estimate")
        .add_comparisons([comparison])
        .bar();
    let mut backend = LifecycleBackend::default();

    let error = chart.render_with_backend(&mut backend).unwrap_err();

    assert!(error
        .to_string()
        .contains("contradictory duplicate ownership"));
    assert_eq!(backend.begins, 0);
}

#[test]
fn facet_hue_preflight_keeps_repeated_endpoint_pairs_panel_local() {
    let data = df!(
        "facet" => ["one", "one", "two", "two"],
        "category" => ["before", "before", "before", "before"],
        "hue" => ["control", "active", "control", "active"],
        "value" => [2.0_f64, 3.0, 4.0, 5.0]
    )
    .unwrap();
    let comparisons = ["one", "two"].map(|facet| {
        ComparisonAnnotation {
            endpoint_a: ComparisonEndpoint::new("before")
                .with_group("control")
                .with_hue("control"),
            endpoint_b: ComparisonEndpoint::new("before")
                .with_group("active")
                .with_hue("active"),
            ..ComparisonAnnotation::new("before", "before", 0.01)
        }
        .in_facet("facet", facet)
    });
    let mut chart = Chart::new(&data)
        .group("hue")
        .hue("hue")
        .facet_col("facet")
        .bar_aggregation(BarAggregation::Identity);
    chart
        .x("category")
        .y("value")
        .add_comparisons(comparisons)
        .bar();
    let directory = tempdir("facet-hue-panel-comparisons");
    let output = directory.join("facets.svg");

    chart.render_to_svg(&output).unwrap();
    let svg = std::fs::read_to_string(output).unwrap();

    assert_eq!(svg.matches("class=\"significance-bracket\"").count(), 2);
    assert_eq!(svg.matches(">Control</text>").count(), 1);
    assert_eq!(svg.matches(">Active</text>").count(), 1);
}

#[test]
fn facet_owned_comparison_on_unfaceted_chart_fails_before_backend_begin() {
    let data = estimate_data();
    let mut chart = Chart::new(&data);
    chart
        .x("factor")
        .y("estimate")
        .add_comparisons([
            ComparisonAnnotation::new("before", "after", 0.01).in_facet("facet", "one")
        ])
        .bar();
    let mut backend = LifecycleBackend::default();

    let error = chart.render_with_backend(&mut backend).unwrap_err();

    assert!(error
        .to_string()
        .contains("facet-owned significance comparisons require"));
    assert_eq!(backend.begins, 0);
}

#[test]
fn fixed_facets_use_renderer_stacks_and_reject_incompatible_categories() {
    let stacked_data = df!(
        "facet" => ["one", "one", "two", "two"],
        "category" => ["a", "a", "a", "a"],
        "hue" => ["left", "right", "left", "right"],
        "value" => [5.0_f64, 5.0, 1.0, 1.0]
    )
    .unwrap();
    let dir = tempdir("facet-stacked-domain");
    let output = dir.join("stacked.svg");
    let mut stacked = Chart::new(&stacked_data)
        .facet_col("facet")
        .group("hue")
        .hue("hue");
    stacked.x("category").y("value").stacked_bar();
    stacked.render_to_svg(&output).unwrap();
    let svg = std::fs::read_to_string(output).unwrap();
    assert_eq!(svg.matches(">10</text>").count(), 2);
    assert_grid_layers_before_marks(&svg, &["facet-0-grid", "facet-1-grid"]);

    let incompatible_data = df!(
        "facet" => ["one", "one", "two", "two"],
        "category" => ["a", "b", "a", "c"],
        "value" => [1.0_f64, 2.0, 3.0, 4.0]
    )
    .unwrap();
    let mut incompatible = Chart::new(&incompatible_data)
        .facet_col("facet")
        .bar_aggregation(BarAggregation::Identity);
    incompatible.x("category").y("value").bar();
    let error = incompatible
        .render_with_backend(&mut LifecycleBackend::default())
        .unwrap_err()
        .to_string();
    assert!(error.contains("identical categorical levels and ordering"));
}

#[test]
fn sparse_cartesian_facets_draw_global_legend_on_first_rendered_panel() {
    let data = df!(
        "row" => ["r1", "r1", "r2", "r2"],
        "column" => ["c2", "c2", "c1", "c1"],
        "x" => [1.0_f64, 2.0, 1.0, 2.0],
        "y" => [1.0_f64, 2.0, 2.0, 3.0],
        "trace" => ["control", "control", "active", "active"]
    )
    .unwrap();
    let mut chart = Chart::new(&data)
        .hue("trace")
        .facet_row("row")
        .facet_col("column");
    chart.x("x").y("y").size(760, 520).line();
    let dir = tempdir("sparse-facet-legend");
    let output = dir.join("facets.svg");

    chart.render_to_svg(&output).unwrap();
    let svg = std::fs::read_to_string(output).unwrap();

    assert_eq!(svg.matches("-legend\"").count(), 1);
    assert_eq!(svg.matches(">Control</text>").count(), 1);
    assert_eq!(svg.matches(">Active</text>").count(), 1);
    assert_grid_layers_before_marks(&svg, &["facet-0-grid", "facet-3-grid"]);
}

#[test]
fn long_label_facets_measure_gutters_and_wrap_strips_inside_owned_cells() {
    let category = "categorical level requiring measured gutter";
    let other_category = "another deliberately wide category label";
    let left_facet = "first panel with a deliberately long descriptive facet value";
    let right_facet = "second panel with another deliberately long descriptive facet value";
    let data = df!(
        "facet_dimension_with_long_name" => [left_facet, left_facet, right_facet, right_facet],
        "category" => [category, other_category, category, other_category],
        "value" => [1.0_f64, 2.0, 1.5, 2.5]
    )
    .unwrap();
    let mut chart = Chart::new(&data)
        .facet_col("facet_dimension_with_long_name")
        .bar_aggregation(BarAggregation::Identity);
    chart
        .x("category")
        .y("value")
        .size(1_100, 420)
        .horizontal_bar();
    let mut backend = LifecycleBackend::default();

    chart.render_with_backend(&mut backend).unwrap();

    assert_eq!(backend.clip_regions.len(), 2);
    let mut category_labels = backend
        .texts
        .iter()
        .filter(|(_, _, text)| text.starts_with("Categorical"))
        .map(|(x, _, text)| (*x, text.as_str()))
        .collect::<Vec<_>>();
    category_labels.sort_unstable_by_key(|(x, _)| *x);
    assert_eq!(category_labels.len(), 2, "{:?}", backend.texts);
    let label_left = f64::from(category_labels[1].0)
        - lichtung_plot::layout::deterministic_text_width(category_labels[1].1, 12.0).ceil();
    assert!(label_left > backend.clip_regions[0].x1.ceil());

    let full_right_strip =
        "Facet dimension with long name = Second panel with another deliberately long descriptive facet value";
    assert!(!backend
        .texts
        .iter()
        .any(|(_, _, text)| text == full_right_strip));
    assert!(backend
        .texts
        .iter()
        .filter(|(_, _, text)| text.starts_with("Facet dimension"))
        .all(|(_, y, _)| f64::from(*y) > backend.clip_regions[0].y1));
}

#[test]
fn faceted_shared_axis_titles_preserve_explicit_multiline_text_outside_panels() {
    let data = df!(
        "facet" => ["left", "left", "right", "right"],
        "x" => [1.0_f64, 2.0, 1.0, 2.0],
        "y" => [1.0_f64, 2.0, 2.0, 3.0]
    )
    .unwrap();
    let mut chart = Chart::new(&data)
        .facet_col("facet")
        .x_label("Shared horizontal axis\ncontinued horizontally")
        .y_label("Shared vertical axis\ncontinued vertically");
    chart.x("x").y("y").size(760, 420).line();
    let mut backend = LifecycleBackend::default();

    chart.render_with_backend(&mut backend).unwrap();

    let horizontal = backend
        .texts
        .iter()
        .filter(|(_, _, text)| {
            matches!(
                text.as_str(),
                "Shared horizontal axis" | "continued horizontally"
            )
        })
        .collect::<Vec<_>>();
    let vertical = backend
        .texts
        .iter()
        .filter(|(_, _, text)| {
            matches!(
                text.as_str(),
                "Shared vertical axis" | "continued vertically"
            )
        })
        .collect::<Vec<_>>();
    let panel_left = backend
        .clip_regions
        .iter()
        .map(|panel| panel.x0)
        .reduce(f64::min)
        .unwrap();
    let panel_bottom = backend
        .clip_regions
        .iter()
        .map(|panel| panel.y0)
        .reduce(f64::min)
        .unwrap();

    assert_eq!(horizontal.len(), 2, "{:?}", backend.texts);
    assert_eq!(vertical.len(), 2, "{:?}", backend.texts);
    assert!(horizontal
        .iter()
        .all(|(_, y, _)| f64::from(*y) < panel_bottom));
    assert!(vertical.iter().all(|(x, _, _)| f64::from(*x) < panel_left));
}

#[test]
fn shared_facet_axis_titles_clear_outside_left_and_bottom_legends() {
    let data = df!(
        "facet" => ["left", "left", "right", "right"],
        "x" => [1.0_f64, 2.0, 1.0, 2.0],
        "y" => [1.0_f64, 2.0, 2.0, 3.0],
        "trace" => ["control", "active", "control", "active"]
    )
    .unwrap();
    let render = |position| {
        let mut chart = Chart::new(&data)
            .facet_col("facet")
            .group("trace")
            .hue("trace")
            .x_label("Shared horizontal")
            .y_label("Shared vertical");
        let _ = chart.legend_position(position);
        chart.x("x").y("y").size(820, 520).line();
        let mut backend = LifecycleBackend::default();
        chart.render_with_backend(&mut backend).unwrap();
        backend
    };

    let left = render(LegendPosition::OutsideLeft);
    let vertical_x = left
        .texts
        .iter()
        .find(|(_, _, text)| text == "Shared vertical")
        .unwrap()
        .0;
    let left_legend_x = left
        .texts
        .iter()
        .find(|(_, _, text)| text == "Trace")
        .unwrap()
        .0;
    assert!(vertical_x < left_legend_x);

    let bottom = render(LegendPosition::OutsideBottom);
    let horizontal_y = bottom
        .texts
        .iter()
        .find(|(_, _, text)| text == "Shared horizontal")
        .unwrap()
        .1;
    let bottom_legend_y = bottom
        .texts
        .iter()
        .find(|(_, _, text)| text == "Trace")
        .unwrap()
        .1;
    assert!(horizontal_y < bottom_legend_y);
}

#[test]
fn crossed_facets_order_valid_hues_by_original_source_rows() {
    let data = df!(
        "row" => ["r1", "r2", "r2", "r1"],
        "column" => ["c1", "c2", "c2", "c1"],
        "x" => [1.0_f64, f64::NAN, 3.0, 2.0],
        "y" => [1.0_f64, 2.0, 3.0, 2.0],
        "trace" => ["control", "dropped", "active", "treated"]
    )
    .unwrap();
    let mut chart = Chart::new(&data)
        .hue("trace")
        .facet_row("row")
        .facet_col("column");
    chart.x("x").y("y").size(760, 520).scatter();
    let directory = tempdir("crossed-facet-source-hue-order");
    let output = directory.join("facets.svg");

    chart.render_to_svg(&output).unwrap();
    let svg = std::fs::read_to_string(output).unwrap();
    let first_panel_marks = svg
        .split_once("<g clip-path=")
        .unwrap()
        .1
        .split_once("</g>")
        .unwrap()
        .0;
    let legend = svg
        .split_once("<g id=\"facet-0-legend\">")
        .unwrap()
        .1
        .split_once("</g>")
        .unwrap()
        .0;

    let blue_mark = first_panel_marks.find("fill=\"#0072B2\"").unwrap();
    let green_mark = first_panel_marks.find("fill=\"#009E73\"").unwrap();
    assert!(blue_mark < green_mark);
    assert!(!first_panel_marks.contains("fill=\"#D55E00\""));

    let blue_legend = legend.find("fill=\"#0072B2\"").unwrap();
    let control = legend.find(">Control</text>").unwrap();
    let vermilion_legend = legend.find("fill=\"#D55E00\"").unwrap();
    let active = legend.find(">Active</text>").unwrap();
    let green_legend = legend.find("fill=\"#009E73\"").unwrap();
    let treated = legend.find(">Treated</text>").unwrap();
    assert!(
        blue_legend < control
            && control < vermilion_legend
            && vermilion_legend < active
            && active < green_legend
            && green_legend < treated
    );
    assert!(!svg.contains(">Dropped</text>"));
}

#[test]
fn faceted_line_allows_repeated_hue_x_keys_and_keeps_one_global_legend() {
    let data = df!(
        "facet" => ["left", "left", "left", "left", "right", "right", "right", "right"],
        "x" => [1.0_f64, 2.0, 1.0, 2.0, 1.0, 2.0, 1.0, 2.0],
        "y" => [1.0_f64, 2.0, 2.0, 3.0, 3.0, 4.0, 4.0, 5.0],
        "trace" => ["active", "active", "control", "control", "active", "active", "control", "control"]
    )
    .unwrap();
    let mut chart = Chart::new(&data)
        .group("trace")
        .hue("trace")
        .facet_col("facet");
    chart.x("x").y("y").size(760, 360).line();
    let dir = tempdir("line-repeated-facet-keys");
    let output = dir.join("facets.svg");

    chart.render_to_svg(&output).unwrap();
    let svg = std::fs::read_to_string(output).unwrap();

    assert_eq!(svg.matches(">Active</text>").count(), 1);
    assert_eq!(svg.matches(">Control</text>").count(), 1);
    assert!(svg.find(">Active</text>").unwrap() < svg.find(">Control</text>").unwrap());
}

#[test]
fn facet_source_rows_do_not_collide_with_old_reserved_numeric_channel() {
    let data = df!(
        "facet" => ["left", "right", "left", "right"],
        "x" => [1.0_f64, 1.0, 2.0, 2.0],
        "__lichtung_internal_source_row_7a" => [10.0_f64, 20.0, 11.0, 21.0],
        "lower" => [9.0_f64, 19.0, 10.0, 20.0],
        "upper" => [11.0_f64, 21.0, 12.0, 22.0]
    )
    .unwrap();
    let mut chart = Chart::new(&data)
        .facet_col("facet")
        .interval_bounds("lower", "upper");
    chart.x("x").y("__lichtung_internal_source_row_7a").line();
    let mut backend = LifecycleBackend::default();

    chart.render_with_backend(&mut backend).unwrap();

    assert_eq!((backend.begins, backend.ends), (1, 1));
    assert_eq!(
        data.column("__lichtung_internal_source_row_7a")
            .unwrap()
            .f64()
            .unwrap()
            .into_no_null_iter()
            .collect::<Vec<_>>(),
        [10.0, 20.0, 11.0, 21.0]
    );
}

#[test]
fn facet_source_rows_do_not_collide_with_old_reserved_categorical_channel() {
    let data = df!(
        "facet" => ["left", "right", "left", "right"],
        "__lichtung_internal_source_row_7a" => ["alpha", "beta", "alpha", "beta"],
        "value" => [1.0_f64, 2.0, 3.0, 4.0]
    )
    .unwrap();
    let mut chart = Chart::new(&data)
        .facet_col("facet")
        .facet_scales(FacetScales::FreeX);
    chart
        .x("__lichtung_internal_source_row_7a")
        .y("value")
        .size(760, 360)
        .boxplot();
    let directory = tempdir("facet-reserved-categorical-channel");
    let output = directory.join("facets.svg");

    chart.render_to_svg(&output).unwrap();
    let svg = std::fs::read_to_string(output).unwrap();

    assert!(svg.contains(">Alpha</text>"));
    assert!(svg.contains(">Beta</text>"));
    assert_eq!(
        data.column("__lichtung_internal_source_row_7a")
            .unwrap()
            .str()
            .unwrap()
            .iter()
            .collect::<Vec<_>>(),
        [Some("alpha"), Some("beta"), Some("alpha"), Some("beta")]
    );
}

#[test]
fn faceted_stacked_bar_preflights_extreme_finite_panels_without_pooling() {
    let value = f64::MAX * 0.26;
    let data = df!(
        "facet" => ["left", "left", "right", "right"],
        "category" => ["total", "total", "total", "total"],
        "series" => ["active", "control", "active", "control"],
        "value" => [value, value, value, value]
    )
    .unwrap();
    let mut pooled = Chart::new(&data).group("series").hue("series");
    pooled
        .x("category")
        .y("value")
        .legend_position(LegendPosition::OutsideRight)
        .stacked_bar();
    let pooled_error = pooled
        .render_with_backend(&mut LifecycleBackend::default())
        .unwrap_err();
    assert!(pooled_error.to_string().contains("positive stack geometry"));

    let mut chart = Chart::new(&data)
        .group("series")
        .hue("series")
        .facet_col("facet");
    chart
        .x("category")
        .y("value")
        .legend_position(LegendPosition::OutsideRight)
        .size(760, 360)
        .stacked_bar();
    let directory = tempdir("facet-extreme-stacked-bar");
    let output = directory.join("facets.svg");

    chart.render_to_svg(&output).unwrap();
    let svg = std::fs::read_to_string(output).unwrap();

    assert_eq!(svg.matches(">Active</text>").count(), 1);
    assert_eq!(svg.matches(">Control</text>").count(), 1);
    assert!(svg.find(">Active</text>").unwrap() < svg.find(">Control</text>").unwrap());
}

#[test]
fn faceted_interaction_allows_repeated_trace_x_keys() {
    let data = df!(
        "facet" => ["left", "left", "left", "left", "right", "right", "right", "right"],
        "factor" => ["before", "after", "before", "after", "before", "after", "before", "after"],
        "trace" => ["control", "control", "active", "active", "control", "control", "active", "active"],
        "estimate" => [1.0_f64, 2.0, 1.5, 2.5, 3.0, 4.0, 3.5, 4.5],
        "lower" => [0.5_f64, 1.5, 1.0, 2.0, 2.5, 3.5, 3.0, 4.0],
        "upper" => [1.5_f64, 2.5, 2.0, 3.0, 3.5, 4.5, 4.0, 5.0]
    )
    .unwrap();
    let mut chart = Chart::new(&data)
        .group("trace")
        .hue("trace")
        .facet_col("facet")
        .interval_bounds("lower", "upper")
        .interval_semantics(confidence(InferentialScale::Response))
        .interaction_source(EstimateSource::EstimatedMarginalMean);
    chart.x("factor").y("estimate").size(760, 360).interaction();
    let mut backend = LifecycleBackend::default();

    chart.render_with_backend(&mut backend).unwrap();

    assert_eq!((backend.begins, backend.ends), (1, 1));
}

#[test]
fn faceted_numeric_errors_report_original_source_rows_before_begin() {
    let data = df!(
        "facet" => ["left", "right", "left", "right"],
        "x" => [1.0_f64, 1.0, 2.0, 2.0],
        "y" => [1.0_f64, 2.0, 3.0, f64::INFINITY]
    )
    .unwrap();
    let mut chart = Chart::new(&data).facet_col("facet");
    chart.x("x").y("y").line();
    let mut backend = LifecycleBackend::default();

    let error = chart.render_with_backend(&mut backend).unwrap_err();

    assert!(error.to_string().contains("column \"y\""));
    assert!(error.to_string().contains("source row 3"));
    assert_eq!(backend.begins, 0);
}

#[test]
fn faceted_categorical_errors_report_original_source_rows_before_begin() {
    let data = df!(
        "facet" => ["left", "right", "left", "right"],
        "category" => [Some("a"), Some("b"), Some("a"), None],
        "value" => [1.0_f64, 2.0, 3.0, 4.0]
    )
    .unwrap();
    let mut chart = Chart::new(&data).facet_col("facet");
    chart.x("category").y("value").boxplot();
    let mut backend = LifecycleBackend::default();

    let error = chart.render_with_backend(&mut backend).unwrap_err();

    assert!(error.to_string().contains("column \"category\""));
    assert!(error.to_string().contains("source row 3"));
    assert_eq!(backend.begins, 0);
}

#[test]
fn scatter_facets_reject_null_hue_rows_before_begin() {
    let data = df!(
        "facet" => ["one", "one", "two", "two"],
        "x" => [1.0_f64, 2.0, 1.0, 2.0],
        "y" => [1.0_f64, 2.0, 2.0, 3.0],
        "trace" => [Some("control"), None, Some("active"), Some("active")]
    )
    .unwrap();
    let mut chart = Chart::new(&data).hue("trace").facet_col("facet");
    chart.x("x").y("y").size(700, 320).scatter();
    let mut backend = LifecycleBackend::default();

    let error = chart.render_with_backend(&mut backend).unwrap_err();

    assert!(error.to_string().contains("required categorical column"));
    assert_eq!(backend.begins, 0);
}

#[test]
fn strict_line_facets_still_reject_null_hue_rows() {
    let data = df!(
        "facet" => ["one", "one"],
        "x" => [1.0_f64, 2.0],
        "y" => [1.0_f64, 2.0],
        "trace" => [Some("control"), None]
    )
    .unwrap();
    let mut chart = Chart::new(&data).hue("trace").facet_col("facet");
    chart.x("x").y("y").line();

    let error = chart
        .render_with_backend(&mut LifecycleBackend::default())
        .unwrap_err();

    assert!(error.to_string().contains("contains null"));
}
