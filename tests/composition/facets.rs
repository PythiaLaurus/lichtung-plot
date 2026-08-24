use super::*;

fn line(data_frame: &DataFrame) -> Plot<lichtung_plot::kind::Line> {
    Plot::line(
        data::numeric(data_frame, "x"),
        data::numeric(data_frame, "estimate"),
    )
}

fn identity_bar(data_frame: &DataFrame) -> Plot<lichtung_plot::kind::Bar> {
    Plot::bar(
        data::category(data_frame, "factor"),
        BarInput::estimates(data::numeric(data_frame, "estimate")),
    )
}

fn clip_rects(svg: &str) -> Vec<(f64, f64, f64, f64)> {
    svg.lines()
        .filter(|line| line.starts_with("<clipPath "))
        .map(|line| {
            (
                attribute(line, "x").parse().unwrap(),
                attribute(line, "y").parse().unwrap(),
                attribute(line, "width").parse().unwrap(),
                attribute(line, "height").parse().unwrap(),
            )
        })
        .collect()
}

fn significance_bracket_bounds(svg: &str) -> (f64, f64, f64, f64) {
    let path = svg
        .lines()
        .find(|line| line.starts_with("<path class=\"significance-bracket\""))
        .expect("significance bracket path");
    let coordinates = attribute(path, "d")
        .split_whitespace()
        .filter_map(|token| token.parse::<f64>().ok())
        .collect::<Vec<_>>();
    let mut points = coordinates.chunks_exact(2);
    let first = points.next().expect("significance bracket coordinates");
    let (mut min_x, mut max_x, mut min_y, mut max_y) = (first[0], first[0], first[1], first[1]);
    for point in points {
        min_x = min_x.min(point[0]);
        max_x = max_x.max(point[0]);
        min_y = min_y.min(point[1]);
        max_y = max_y.max(point[1]);
    }
    (min_x, min_y, max_x - min_x, max_y - min_y)
}

#[test]
fn faceting_uses_unique_clips_and_panel_owned_annotations() {
    let frame = estimate_data();
    let svg = render_svg(
        line(&frame).facet(FacetSpec::column(data::category(&frame, "facet"))),
        "faceted-line",
    )
    .unwrap();
    assert_eq!(svg.matches("<clipPath id=").count(), 2);
    assert_unique_svg_ids(&svg);

    let bar = identity_bar(&frame)
        .facet(FacetSpec::column(data::category(&frame, "facet")).scales(FacetScales::Free))
        .significance([
            Significance::between("before", "after", SignificanceMark::PValue(0.01))
                .facet(FacetCell::single("one")),
        ]);
    let svg = render_svg(bar, "faceted-annotation").unwrap();
    assert_eq!(svg.matches("class=\"significance-bracket\"").count(), 1);
}

#[test]
fn significance_cell_shape_must_match_facet_layout() {
    let frame = estimate_data();
    let comparison = Significance::between("before", "after", SignificanceMark::PValue(0.01))
        .facet(FacetCell::grid("one", "two"));
    let error = identity_bar(&frame)
        .facet(FacetSpec::column(data::category(&frame, "facet")))
        .significance([comparison])
        .preflight()
        .unwrap_err();
    assert!(error
        .to_string()
        .contains("shape does not match the configured facet layout"));
}

#[test]
fn grid_facet_cell_owns_exactly_one_significance_panel() {
    let frame = df!(
        "row" => ["top", "top", "top", "top", "bottom", "bottom", "bottom", "bottom"],
        "column" => ["left", "left", "right", "right", "left", "left", "right", "right"],
        "factor" => ["before", "after", "before", "after", "before", "after", "before", "after"],
        "estimate" => [1.0_f64, 2.0, 1.5, 2.5, 3.0, 4.0, 3.5, 4.5]
    )
    .unwrap();
    let chart = identity_bar(&frame)
        .facet(FacetSpec::grid(
            data::category(&frame, "row"),
            data::category(&frame, "column"),
        ))
        .significance([
            Significance::between("before", "after", SignificanceMark::OneStar)
                .facet(FacetCell::grid("top", "right")),
        ]);

    let svg = render_svg(chart, "grid-facet-cell").unwrap();

    assert_eq!(svg.matches("class=\"significance-bracket\"").count(), 1);
    let panels = clip_rects(&svg);
    assert_eq!(panels.len(), 4);
    let top_y = panels
        .iter()
        .map(|panel| panel.1)
        .min_by(f64::total_cmp)
        .unwrap();
    let top_right = panels
        .iter()
        .filter(|panel| (panel.1 - top_y).abs() < 0.001)
        .max_by(|left, right| left.0.total_cmp(&right.0))
        .copied()
        .unwrap();
    let bracket = significance_bracket_bounds(&svg);
    assert!(
        bracket.0 >= top_right.0
            && bracket.0 + bracket.2 <= top_right.0 + top_right.2
            && bracket.1 >= top_right.1
            && bracket.1 + bracket.3 <= top_right.1 + top_right.3
    );
}

#[test]
fn faceted_significance_requires_explicit_cell_ownership() {
    let frame = estimate_data();
    let error = identity_bar(&frame)
        .facet(FacetSpec::column(data::category(&frame, "facet")))
        .significance([Significance::between(
            "before",
            "after",
            SignificanceMark::OneStar,
        )])
        .preflight()
        .unwrap_err();

    assert!(error.to_string().contains("must identify one FacetCell"));
}

#[test]
fn significance_cell_must_identify_an_observed_panel() {
    let frame = estimate_data();
    let error = identity_bar(&frame)
        .facet(FacetSpec::column(data::category(&frame, "facet")))
        .significance([
            Significance::between("before", "after", SignificanceMark::OneStar)
                .facet(FacetCell::single("missing")),
        ])
        .preflight()
        .unwrap_err();

    assert!(error
        .to_string()
        .contains("does not identify an observed facet panel"));
}

#[test]
fn facet_series_preflight_keeps_repeated_endpoint_pairs_panel_local() {
    let frame = df!(
        "facet" => ["one", "one", "two", "two"],
        "category" => ["before", "before", "before", "before"],
        "hue" => ["control", "active", "control", "active"],
        "value" => [2.0_f64, 3.0, 4.0, 5.0]
    )
    .unwrap();
    let comparisons = ["one", "two"].map(|facet| {
        Significance::between(
            SignificanceTarget::category_series("before", "control"),
            SignificanceTarget::category_series("before", "active"),
            SignificanceMark::PValue(0.01),
        )
        .facet(FacetCell::single(facet))
    });
    let svg = render_svg(
        Plot::bar(
            data::category(&frame, "category"),
            BarInput::estimates(data::numeric(&frame, "value")),
        )
        .series(data::category(&frame, "hue"))
        .facet(FacetSpec::column(data::category(&frame, "facet")))
        .significance(comparisons),
        "facet-series-panel-comparisons",
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
        BarInput::estimates(lichtung_plot::NumericValues::new("estimate", [1.0, 2.0])),
    )
    .significance([
        Significance::between("before", "after", SignificanceMark::PValue(0.01))
            .facet(FacetCell::single("one")),
    ])
    .preflight()
    .unwrap_err();
    assert!(error
        .to_string()
        .contains("facet-owned significance annotations require"));
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
        .facet(FacetSpec::column(data::category(&stacked, "facet"))),
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
        BarInput::estimates(data::numeric(&incompatible, "value")),
    )
    .facet(FacetSpec::column(data::category(&incompatible, "facet")))
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
            .facet(FacetSpec::grid(
                data::category(&frame, "row"),
                data::category(&frame, "column"),
            ))
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
            BarInput::estimates(data::numeric(&frame, "value")),
        )
        .facet(FacetSpec::column(data::category(
            &frame,
            "facet_dimension_with_long_name",
        )))
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
            .facet(FacetSpec::column(data::category(&frame, "facet")))
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
                .facet(FacetSpec::column(data::category(&frame, "facet")))
                .x_label("Shared horizontal")
                .y_label("Shared vertical")
                .legend(LegendPolicy::Visible(position))
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
            .facet(FacetSpec::grid(
                data::category(&frame, "row"),
                data::category(&frame, "column"),
            ))
            .size(760, 520),
        "crossed-facet-source-hue-order",
    )
    .unwrap();
    assert!(svg.find(">Control</text>").unwrap() < svg.find(">Active</text>").unwrap());
    assert!(svg.find(">Active</text>").unwrap() < svg.find(">Treated</text>").unwrap());
    assert!(!svg.contains(">Dropped</text>"));
}

#[test]
fn sparse_inside_facet_legend_merges_exact_observed_pairs() {
    let frame = df!(
        "facet" => ["left", "left", "right", "right"],
        "group" => ["g1", "g1", "g2", "g2"],
        "hue" => ["h1", "h1", "h2", "h2"],
        "x" => [0.0_f64, 1.0, 0.0, 1.0],
        "y" => [1.0_f64, 2.0, 3.0, 4.0],
    )
    .unwrap();
    let svg = render_svg(
        Plot::line(data::numeric(&frame, "x"), data::numeric(&frame, "y"))
            .group(data::category(&frame, "group"))
            .hue(data::category(&frame, "hue"))
            .facet(FacetSpec::column(data::category(&frame, "facet")))
            .legend(LegendPolicy::Visible(LegendPosition::InsideTopRight))
            .size(920, 380),
        "sparse-inside-pair-legend",
    )
    .unwrap();

    assert_eq!(svg.matches(">Group × Hue</text>").count(), 1);
    assert_eq!(svg.matches(">G 1, H 1</text>").count(), 1);
    assert_eq!(svg.matches(">G 2, H 2</text>").count(), 1);
    let legend = svg
        .split("<g id=\"")
        .skip(1)
        .find(|layer| {
            layer
                .split_once('\"')
                .is_some_and(|(id, _)| id.contains("legend"))
        })
        .unwrap()
        .split_once('>')
        .unwrap()
        .1
        .split_once("</g>")
        .unwrap()
        .0;
    assert_eq!(legend.matches("<polyline ").count(), 2);
    assert_eq!(legend.matches("<circle ").count(), 2);
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
            .facet(FacetSpec::column(data::category(&frame, "facet")))
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
    .band(Bounds::new(
        lichtung_plot::NumericValues::new("lower", [9.0, 19.0, 10.0, 20.0]),
        lichtung_plot::NumericValues::new("upper", [11.0, 21.0, 12.0, 22.0]),
    ))
    .facet(FacetSpec::column(
        lichtung_plot::CategoryValues::new("facet", ["left", "right", "left", "right"]).unwrap(),
    ));
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
    .facet(
        FacetSpec::column(
            lichtung_plot::CategoryValues::new("facet", ["left", "right", "left", "right"])
                .unwrap(),
        )
        .scales(FacetScales::FreeX),
    )
    .size(760, 360);
    let svg = render_svg(categorical, "reserved-categorical").unwrap();
    assert!(svg.contains(">Alpha</text>") && svg.contains(">Beta</text>"));

    let current_internal_name = Plot::bar(
        lichtung_plot::CategoryValues::new("category", ["a", "a", "a", "a"]).unwrap(),
        BarInput::estimates(lichtung_plot::NumericValues::new(
            "value",
            [1.0, 2.0, 3.0, 4.0],
        )),
    )
    .series(lichtung_plot::CategoryValues::new("__series", ["g1", "g2", "g1", "g2"]).unwrap())
    .facet(FacetSpec::column(
        lichtung_plot::CategoryValues::new("__series", ["left", "left", "right", "right"]).unwrap(),
    ))
    .size(760, 360);
    let svg = render_svg(current_internal_name, "current-internal-facet-name").unwrap();
    assert_eq!(svg.matches("<clipPath id=").count(), 2);
    let bar_count = svg
        .split("<g clip-path=")
        .skip(1)
        .filter_map(|group| group.split_once("</g>"))
        .map(|(marks, _)| marks.matches("<polygon points=").count())
        .sum::<usize>();
    assert_eq!(bar_count, 4);
    assert!(svg.contains(">Series = Left</text>") && svg.contains(">Series = Right</text>"));
}

#[test]
fn row_and_column_facets_require_distinct_variable_names() {
    let plot = Plot::bar(
        lichtung_plot::CategoryValues::new("category", ["a", "b"]).unwrap(),
        BarInput::estimates(lichtung_plot::NumericValues::new("value", [1.0, 2.0])),
    )
    .facet(FacetSpec::grid(
        lichtung_plot::CategoryValues::new("facet", ["r1", "r2"]).unwrap(),
        lichtung_plot::CategoryValues::new("facet", ["c1", "c2"]).unwrap(),
    ));

    let error = plot.preflight().unwrap_err();
    assert!(matches!(error, lichtung_plot::Error::Data(_)));
    assert!(
        error.to_string().contains("must use different columns"),
        "{error}"
    );
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
            BarInput::estimates(data::numeric(&frame, "value")),
        )
        .category_order(CategoryOrder::explicit(order))
        .facet(FacetSpec::column(data::category(&frame, "panel")).scales(FacetScales::FreeX))
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
        BarInput::estimates(data::numeric(&frame, "value")),
    )
    .category_order(lichtung_plot::ValueSort::Descending)
    .facet(FacetSpec::column(data::category(&frame, "panel")));

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
        .legend(LegendPolicy::Visible(LegendPosition::OutsideRight))
    };
    assert!(base()
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("positive stack geometry"));
    let svg = render_svg(
        base()
            .facet(FacetSpec::column(data::category(&frame, "facet")))
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
        Bounds::new(
            data::numeric(&frame, "lower"),
            data::numeric(&frame, "upper"),
        ),
    )
    .hue(data::category(&frame, "trace"))
    .facet(FacetSpec::column(data::category(&frame, "facet")))
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
        .facet(FacetSpec::column(data::category(&numeric, "facet")))
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
    .facet(FacetSpec::column(data::category(&categorical, "facet")))
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
        .facet(FacetSpec::column(data::category(&frame, "facet")))
        .preflight()
        .unwrap_err();
    assert!(scatter_error
        .to_string()
        .contains("value \"trace\" contains null at source row 1"));
    let line_error = Plot::line(data::numeric(&frame, "x"), data::numeric(&frame, "y"))
        .hue(data::category(&frame, "trace"))
        .facet(FacetSpec::column(data::category(&frame, "facet")))
        .preflight()
        .unwrap_err();
    assert!(line_error.to_string().contains("contains null"));
}
