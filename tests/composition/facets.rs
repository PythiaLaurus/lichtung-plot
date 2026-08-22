use super::*;

fn line(data_frame: &DataFrame) -> Plot {
    Plot::line(
        data::numeric(data_frame, "x"),
        data::numeric(data_frame, "estimate"),
    )
}

fn identity_bar(data_frame: &DataFrame) -> Plot {
    Plot::bar(
        data::category(data_frame, "factor"),
        data::numeric(data_frame, "estimate"),
    )
    .bar_aggregation(BarAggregation::Identity)
}

#[test]
fn faceting_uses_unique_clips_and_panel_owned_annotations() {
    let frame = estimate_data();
    let comparison = ComparisonAnnotation::new("before", "after", 0.01).in_facet("facet", "one");
    let interaction = Plot::interaction(
        data::category(&frame, "factor"),
        data::numeric(&frame, "estimate"),
        data::category(&frame, "trace"),
        data::numeric(&frame, "lower"),
        data::numeric(&frame, "upper"),
    )
    .interval_semantics(confidence(InferentialScale::Response))
    .interaction_source(EstimateSource::ObservedCellSummary)
    .hue(data::category(&frame, "trace"))
    .facet_col(data::category(&frame, "facet"))
    .add_comparisons([comparison]);
    assert!(interaction
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("not supported"));

    let svg = render_svg(
        line(&frame).facet_col(data::category(&frame, "facet")),
        "faceted-line",
    )
    .unwrap();
    assert_eq!(svg.matches("<clipPath id=").count(), 2);
    assert_unique_svg_ids(&svg);

    let bar = identity_bar(&frame)
        .facet_col(data::category(&frame, "facet"))
        .facet_scales(FacetScales::Free)
        .add_comparisons([ComparisonAnnotation::new("before", "after", 0.01)
            .in_facet("facet", "one")
            .in_facet("facet", "one")]);
    let svg = render_svg(bar, "faceted-annotation").unwrap();
    assert_eq!(svg.matches("class=\"significance-bracket\"").count(), 1);
}

#[test]
fn contradictory_duplicate_facet_ownership_fails_preflight() {
    let frame = estimate_data();
    let comparison = ComparisonAnnotation::new("before", "after", 0.01)
        .in_facet("facet", "one")
        .in_facet("facet", "two");
    let error = identity_bar(&frame)
        .facet_col(data::category(&frame, "facet"))
        .add_comparisons([comparison])
        .preflight()
        .unwrap_err();
    assert!(error
        .to_string()
        .contains("contradictory duplicate ownership"));
}

#[test]
fn facet_hue_preflight_keeps_repeated_endpoint_pairs_panel_local() {
    let frame = df!(
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
    let svg = render_svg(
        Plot::bar(
            data::category(&frame, "category"),
            data::numeric(&frame, "value"),
        )
        .bar_aggregation(BarAggregation::Identity)
        .group(data::category(&frame, "hue"))
        .hue(data::category(&frame, "hue"))
        .facet_col(data::category(&frame, "facet"))
        .add_comparisons(comparisons),
        "facet-hue-panel-comparisons",
    )
    .unwrap();
    assert_eq!(svg.matches("class=\"significance-bracket\"").count(), 2);
    assert_eq!(svg.matches(">Control</text>").count(), 1);
    assert_eq!(svg.matches(">Active</text>").count(), 1);
}

#[test]
fn facet_owned_comparison_on_unfaceted_plot_fails_preflight() {
    let error = Plot::bar(
        lichtung_plot::CategoryValues::new("factor", ["before", "after"]).unwrap(),
        lichtung_plot::NumericValues::new("estimate", [1.0, 2.0]),
    )
    .bar_aggregation(BarAggregation::Identity)
    .add_comparisons([ComparisonAnnotation::new("before", "after", 0.01).in_facet("facet", "one")])
    .preflight()
    .unwrap_err();
    assert!(error
        .to_string()
        .contains("facet-owned significance comparisons require"));
}

#[test]
fn fixed_facets_use_renderer_stacks_and_reject_incompatible_categories() {
    let stacked = df!(
        "facet" => ["one", "one", "two", "two"],
        "category" => ["a", "a", "a", "a"],
        "hue" => ["left", "right", "left", "right"],
        "value" => [5.0_f64, 5.0, 1.0, 1.0]
    )
    .unwrap();
    let svg = render_svg(
        Plot::stacked_bar(
            data::category(&stacked, "category"),
            data::numeric(&stacked, "value"),
            data::category(&stacked, "hue"),
        )
        .hue(data::category(&stacked, "hue"))
        .facet_col(data::category(&stacked, "facet")),
        "facet-stacked-domain",
    )
    .unwrap();
    assert_eq!(svg.matches(">10</text>").count(), 2);
    let first_marks = svg.find("<g clip-path=").unwrap();
    assert_eq!(svg[..first_marks].matches("-grid\"").count(), 2);

    let incompatible = df!(
        "facet" => ["one", "one", "two", "two"],
        "category" => ["a", "b", "a", "c"],
        "value" => [1.0_f64, 2.0, 3.0, 4.0]
    )
    .unwrap();
    let error = Plot::bar(
        data::category(&incompatible, "category"),
        data::numeric(&incompatible, "value"),
    )
    .bar_aggregation(BarAggregation::Identity)
    .facet_col(data::category(&incompatible, "facet"))
    .preflight()
    .unwrap_err();
    assert!(error
        .to_string()
        .contains("identical categorical levels and ordering"));
}

#[test]
fn sparse_cartesian_facets_draw_global_legend_on_first_rendered_panel() {
    let frame = df!(
        "row" => ["r1", "r1", "r2", "r2"],
        "column" => ["c2", "c2", "c1", "c1"],
        "x" => [1.0_f64, 2.0, 1.0, 2.0],
        "y" => [1.0_f64, 2.0, 2.0, 3.0],
        "trace" => ["control", "control", "active", "active"]
    )
    .unwrap();
    let svg = render_svg(
        Plot::line(data::numeric(&frame, "x"), data::numeric(&frame, "y"))
            .hue(data::category(&frame, "trace"))
            .facet_row(data::category(&frame, "row"))
            .facet_col(data::category(&frame, "column"))
            .size(760, 520),
        "sparse-facet-legend",
    )
    .unwrap();
    assert_eq!(svg.matches("-legend\"").count(), 1);
    assert_eq!(svg.matches(">Control</text>").count(), 1);
    assert_eq!(svg.matches(">Active</text>").count(), 1);
    let first_marks = svg.find("<g clip-path=").unwrap();
    assert_eq!(svg[..first_marks].matches("-grid\"").count(), 2);
}

#[test]
fn long_label_facets_measure_gutters_and_wrap_strips_inside_owned_cells() {
    let category = "categorical level requiring measured gutter";
    let other = "another deliberately wide category label";
    let left = "first panel with a deliberately long descriptive facet value";
    let right = "second panel with another deliberately long descriptive facet value";
    let frame = df!(
        "facet_dimension_with_long_name" => [left, left, right, right],
        "category" => [category, other, category, other],
        "value" => [1.0_f64, 2.0, 1.5, 2.5]
    )
    .unwrap();
    let svg = render_svg(
        Plot::horizontal_bar(
            data::category(&frame, "category"),
            data::numeric(&frame, "value"),
        )
        .bar_aggregation(BarAggregation::Identity)
        .facet_col(data::category(&frame, "facet_dimension_with_long_name"))
        .size(1_100, 420),
        "long-label-facets",
    )
    .unwrap();
    assert!(svg.contains(">Categorical"));
    assert!(!svg.contains(
        ">Facet dimension with long name = Second panel with another deliberately long descriptive facet value</text>"
    ));
    assert_eq!(svg.matches("<clipPath id=").count(), 2);
}

#[test]
fn faceted_shared_axis_titles_preserve_explicit_multiline_text() {
    let frame = df!(
        "facet" => ["left", "left", "right", "right"],
        "x" => [1.0_f64, 2.0, 1.0, 2.0],
        "y" => [1.0_f64, 2.0, 2.0, 3.0]
    )
    .unwrap();
    let svg = render_svg(
        Plot::line(data::numeric(&frame, "x"), data::numeric(&frame, "y"))
            .facet_col(data::category(&frame, "facet"))
            .x_label("Shared horizontal axis\ncontinued horizontally")
            .y_label("Shared vertical axis\ncontinued vertically")
            .size(760, 420),
        "multiline-facet-axes",
    )
    .unwrap();
    for text in [
        "Shared horizontal axis",
        "continued horizontally",
        "Shared vertical axis",
        "continued vertically",
    ] {
        assert_eq!(svg.matches(&format!(">{text}</text>")).count(), 1);
    }
}

#[test]
fn shared_facet_axis_titles_clear_outside_legends() {
    let frame = df!(
        "facet" => ["left", "left", "right", "right"],
        "x" => [1.0_f64, 2.0, 1.0, 2.0],
        "y" => [1.0_f64, 2.0, 2.0, 3.0],
        "trace" => ["control", "active", "control", "active"]
    )
    .unwrap();
    for position in [LegendPosition::OutsideLeft, LegendPosition::OutsideBottom] {
        let svg = render_svg(
            Plot::line(data::numeric(&frame, "x"), data::numeric(&frame, "y"))
                .group(data::category(&frame, "trace"))
                .hue(data::category(&frame, "trace"))
                .facet_col(data::category(&frame, "facet"))
                .x_label("Shared horizontal")
                .y_label("Shared vertical")
                .legend_position(position)
                .size(820, 520),
            "facet-outside-legend",
        )
        .unwrap();
        assert!(svg.contains(">Shared horizontal</text>"));
        assert!(svg.contains(">Shared vertical</text>"));
        assert!(svg.contains(">Trace</text>"));
    }
}

#[test]
fn crossed_facets_order_valid_hues_by_original_source_rows() {
    let frame = df!(
        "row" => ["r1", "r2", "r2", "r1"],
        "column" => ["c1", "c2", "c2", "c1"],
        "x" => [1.0_f64, f64::NAN, 3.0, 2.0],
        "y" => [1.0_f64, 2.0, 3.0, 2.0],
        "trace" => ["control", "dropped", "active", "treated"]
    )
    .unwrap();
    let svg = render_svg(
        Plot::scatter(data::numeric(&frame, "x"), data::numeric(&frame, "y"))
            .hue(data::category(&frame, "trace"))
            .facet_row(data::category(&frame, "row"))
            .facet_col(data::category(&frame, "column"))
            .size(760, 520),
        "crossed-facet-source-hue-order",
    )
    .unwrap();
    assert!(svg.find(">Control</text>").unwrap() < svg.find(">Active</text>").unwrap());
    assert!(svg.find(">Active</text>").unwrap() < svg.find(">Treated</text>").unwrap());
    assert!(!svg.contains(">Dropped</text>"));
}

#[test]
fn faceted_line_allows_repeated_hue_x_keys_and_keeps_one_global_legend() {
    let frame = df!(
        "facet" => ["left", "left", "left", "left", "right", "right", "right", "right"],
        "x" => [1.0_f64, 2.0, 1.0, 2.0, 1.0, 2.0, 1.0, 2.0],
        "y" => [1.0_f64, 2.0, 2.0, 3.0, 3.0, 4.0, 4.0, 5.0],
        "trace" => ["active", "active", "control", "control", "active", "active", "control", "control"]
    )
    .unwrap();
    let svg = render_svg(
        Plot::line(data::numeric(&frame, "x"), data::numeric(&frame, "y"))
            .group(data::category(&frame, "trace"))
            .hue(data::category(&frame, "trace"))
            .facet_col(data::category(&frame, "facet"))
            .size(760, 360),
        "line-repeated-facet-keys",
    )
    .unwrap();
    assert_eq!(svg.matches(">Active</text>").count(), 1);
    assert_eq!(svg.matches(">Control</text>").count(), 1);
    assert!(svg.find(">Active</text>").unwrap() < svg.find(">Control</text>").unwrap());
}

#[test]
fn old_reserved_channel_names_are_ordinary_native_input_names() {
    let numeric = Plot::line(
        lichtung_plot::NumericValues::new("x", [1.0, 1.0, 2.0, 2.0]),
        lichtung_plot::NumericValues::new(
            "__lichtung_internal_source_row_7a",
            [10.0, 20.0, 11.0, 21.0],
        ),
    )
    .interval_bounds(
        lichtung_plot::NumericValues::new("lower", [9.0, 19.0, 10.0, 20.0]),
        lichtung_plot::NumericValues::new("upper", [11.0, 21.0, 12.0, 22.0]),
    )
    .facet_col(
        lichtung_plot::CategoryValues::new("facet", ["left", "right", "left", "right"]).unwrap(),
    );
    assert!(render_svg(numeric, "reserved-numeric")
        .unwrap()
        .contains("<svg "));

    let categorical = Plot::boxplot(
        lichtung_plot::CategoryValues::new(
            "__lichtung_internal_source_row_7a",
            ["alpha", "beta", "alpha", "beta"],
        )
        .unwrap(),
        lichtung_plot::NumericValues::new("value", [1.0, 2.0, 3.0, 4.0]),
    )
    .facet_col(
        lichtung_plot::CategoryValues::new("facet", ["left", "right", "left", "right"]).unwrap(),
    )
    .facet_scales(FacetScales::FreeX)
    .size(760, 360);
    let svg = render_svg(categorical, "reserved-categorical").unwrap();
    assert!(svg.contains(">Alpha</text>") && svg.contains(">Beta</text>"));
}

#[test]
fn faceted_explicit_category_order_validates_globally_and_filters_panel_subsets() {
    let frame = df!(
        "panel" => ["left", "right"],
        "category" => ["a", "b"],
        "value" => [1.0_f64, 2.0]
    )
    .unwrap();
    let plot = |order| {
        Plot::bar(
            data::category(&frame, "category"),
            data::numeric(&frame, "value"),
        )
        .bar_aggregation(BarAggregation::Identity)
        .category_order(CategoryOrder::explicit(order))
        .facet_col(data::category(&frame, "panel"))
        .facet_scales(FacetScales::FreeX)
    };

    plot(["b", "a"]).preflight().unwrap();
    let error = plot(["b", "unknown"]).preflight().unwrap_err();

    assert!(error.to_string().contains("unknown level"));
}

#[test]
fn faceted_bar_rejects_panel_local_value_order() {
    let frame = df!(
        "panel" => ["left", "left", "right", "right"],
        "category" => ["a", "b", "a", "b"],
        "value" => [1.0_f64, 2.0, 3.0, 1.0]
    )
    .unwrap();
    let plot = Plot::bar(
        data::category(&frame, "category"),
        data::numeric(&frame, "value"),
    )
    .bar_aggregation(BarAggregation::Identity)
    .category_order(lichtung_plot::ValueSort::Descending)
    .facet_col(data::category(&frame, "panel"));

    let error = plot.preflight().unwrap_err();

    assert!(error
        .to_string()
        .contains("value-derived category order is not supported with faceting"));
}

#[test]
fn faceted_stacked_bar_preflights_extreme_finite_panels_without_pooling() {
    let value = f64::MAX * 0.26;
    let frame = df!(
        "facet" => ["left", "left", "right", "right"],
        "category" => ["total", "total", "total", "total"],
        "series" => ["active", "control", "active", "control"],
        "value" => [value, value, value, value]
    )
    .unwrap();
    let base = || {
        Plot::stacked_bar(
            data::category(&frame, "category"),
            data::numeric(&frame, "value"),
            data::category(&frame, "series"),
        )
        .hue(data::category(&frame, "series"))
        .legend_position(LegendPosition::OutsideRight)
    };
    assert!(base()
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("positive stack geometry"));
    let svg = render_svg(
        base()
            .facet_col(data::category(&frame, "facet"))
            .size(760, 360),
        "facet-extreme-stacked-bar",
    )
    .unwrap();
    assert_eq!(svg.matches(">Active</text>").count(), 1);
    assert_eq!(svg.matches(">Control</text>").count(), 1);
}

#[test]
fn faceted_interaction_allows_repeated_trace_x_keys() {
    let frame = df!(
        "facet" => ["left", "left", "left", "left", "right", "right", "right", "right"],
        "factor" => ["before", "after", "before", "after", "before", "after", "before", "after"],
        "trace" => ["control", "control", "active", "active", "control", "control", "active", "active"],
        "estimate" => [1.0_f64, 2.0, 1.5, 2.5, 3.0, 4.0, 3.5, 4.5],
        "lower" => [0.5_f64, 1.5, 1.0, 2.0, 2.5, 3.5, 3.0, 4.0],
        "upper" => [1.5_f64, 2.5, 2.0, 3.0, 3.5, 4.5, 4.0, 5.0]
    )
    .unwrap();
    let plot = Plot::interaction(
        data::category(&frame, "factor"),
        data::numeric(&frame, "estimate"),
        data::category(&frame, "trace"),
        data::numeric(&frame, "lower"),
        data::numeric(&frame, "upper"),
    )
    .hue(data::category(&frame, "trace"))
    .facet_col(data::category(&frame, "facet"))
    .interval_semantics(confidence(InferentialScale::Response))
    .interaction_source(EstimateSource::EstimatedMarginalMean)
    .size(760, 360);
    assert!(render_svg(plot, "faceted-interaction")
        .unwrap()
        .contains("<svg "));
}

#[test]
fn faceted_errors_report_original_source_rows_before_output_creation() {
    let numeric = df!(
        "facet" => ["left", "right", "left", "right"],
        "x" => [1.0_f64, 1.0, 2.0, 2.0],
        "y" => [1.0_f64, 2.0, 3.0, f64::INFINITY]
    )
    .unwrap();
    let error = Plot::line(data::numeric(&numeric, "x"), data::numeric(&numeric, "y"))
        .facet_col(data::category(&numeric, "facet"))
        .preflight()
        .unwrap_err()
        .to_string();
    assert!(error.contains("value \"y\"") && error.contains("source row 3"));

    let categorical = df!(
        "facet" => ["left", "right", "left", "right"],
        "category" => [Some("a"), Some("b"), Some("a"), None],
        "value" => [1.0_f64, 2.0, 3.0, 4.0]
    )
    .unwrap();
    let error = Plot::boxplot(
        data::category(&categorical, "category"),
        data::numeric(&categorical, "value"),
    )
    .facet_col(data::category(&categorical, "facet"))
    .preflight()
    .unwrap_err()
    .to_string();
    assert!(error.contains("value \"category\"") && error.contains("source row 3"));
}

#[test]
fn facet_null_hue_contract_differs_between_scatter_and_strict_line() {
    let frame = df!(
        "facet" => ["one", "one", "two", "two"],
        "x" => [1.0_f64, 2.0, 1.0, 2.0],
        "y" => [1.0_f64, 2.0, 2.0, 3.0],
        "trace" => [Some("control"), None, Some("active"), Some("active")]
    )
    .unwrap();
    let scatter_error = Plot::scatter(data::numeric(&frame, "x"), data::numeric(&frame, "y"))
        .hue(data::category(&frame, "trace"))
        .facet_col(data::category(&frame, "facet"))
        .preflight()
        .unwrap_err();
    assert!(scatter_error
        .to_string()
        .contains("value \"trace\" contains null at source row 1"));
    let line_error = Plot::line(data::numeric(&frame, "x"), data::numeric(&frame, "y"))
        .hue(data::category(&frame, "trace"))
        .facet_col(data::category(&frame, "facet"))
        .preflight()
        .unwrap_err();
    assert!(line_error.to_string().contains("contains null"));
}
