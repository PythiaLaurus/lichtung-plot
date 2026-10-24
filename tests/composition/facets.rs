use super::*;

fn identity_bar(data: &EstimateData) -> Plot<lichtung_plot::kind::Bar> {
    Plot::bar(
        data.factor.clone(),
        BarInput::estimates(data.estimate.clone()),
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
fn significance_cell_shape_must_match_facet_layout() {
    let data = estimate_data();
    let comparison = Significance::between("before", "after", SignificanceMark::PValue(0.01))
        .facet(FacetCell::grid("one", "two"));
    let error = identity_bar(&data)
        .facet(FacetSpec::column(data.facet))
        .significance([comparison])
        .preflight()
        .unwrap_err();
    assert!(error
        .to_string()
        .contains("shape does not match the configured facet layout"));
}

#[test]
fn grid_facet_cell_owns_exactly_one_significance_panel() {
    let row = CategoryValues::new(
        "row",
        [
            "top", "top", "top", "top", "bottom", "bottom", "bottom", "bottom",
        ],
    )
    .unwrap();
    let column = CategoryValues::new(
        "column",
        [
            "left", "left", "right", "right", "left", "left", "right", "right",
        ],
    )
    .unwrap();
    let factor = CategoryValues::new(
        "factor",
        [
            "before", "after", "before", "after", "before", "after", "before", "after",
        ],
    )
    .unwrap();
    let estimate = NumericValues::new("estimate", [1.0, 2.0, 1.5, 2.5, 3.0, 4.0, 3.5, 4.5]);
    let plot = Plot::bar(factor, BarInput::estimates(estimate))
        .facet(FacetSpec::grid(row, column).unwrap())
        .significance([
            Significance::between("before", "after", SignificanceMark::OneStar)
                .facet(FacetCell::grid("top", "right")),
        ]);

    let svg = render_svg(plot, "grid-facet-cell").unwrap();

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
    let data = estimate_data();
    let error = identity_bar(&data)
        .facet(FacetSpec::column(data.facet))
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
    let data = estimate_data();
    let error = identity_bar(&data)
        .facet(FacetSpec::column(data.facet))
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
fn final_equal_series_colors_and_aliases_do_not_replace_typed_facet_targets() {
    let category = CategoryValues::new("category", ["same"; 4]).unwrap();
    let series = CategoryValues::new("series", ["g1", "g2", "g1", "g2"]).unwrap();
    let facet = CategoryValues::new("panel", ["left", "left", "right", "right"]).unwrap();
    let plot = Plot::bar(
        category,
        BarInput::estimates(NumericValues::new("value", [2.0, 3.0, 4.0, 5.0])),
    )
    .series(series)
    .categorical_color_scale(CategoricalColorScale::from_map(
        CategoricalColorMap::new([
            ("g1".into(), OpaqueColor::rgb(20, 40, 60)),
            ("g2".into(), OpaqueColor::rgb(20, 40, 60)),
            ("unused".into(), OpaqueColor::rgb(1, 2, 3)),
        ])
        .unwrap(),
    ))
    .labels(
        Labels::raw()
            .alias("g1", "First display")
            .alias("g2", "Second display"),
    )
    .legend(LegendPolicy::Hidden)
    .facet(FacetSpec::column(facet))
    .significance([Significance::between(
        SignificanceTarget::category_series("same", "g1"),
        SignificanceTarget::category_series("same", "g2"),
        SignificanceMark::OneStar,
    )
    .facet(FacetCell::single("right"))])
    .size(1000, 640);
    let svg = render_svg(plot, "facet-typed-color-target").unwrap();
    assert_eq!(svg.matches("class=\"significance-bracket\"").count(), 1);
    assert_eq!(svg.matches("fill=\"#14283C\"").count(), 4);
    let panels = clip_rects(&svg);
    assert_eq!(panels.len(), 2);
    let right = panels
        .iter()
        .max_by(|left, right| left.0.total_cmp(&right.0))
        .unwrap();
    let bracket = significance_bracket_bounds(&svg);
    assert!(bracket.0 >= right.0 && bracket.0 + bracket.2 <= right.0 + right.2);
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
    let stacked_facet = CategoryValues::new("facet", ["one", "one", "two", "two"]).unwrap();
    let stacked_category = CategoryValues::new("category", ["a", "a", "a", "a"]).unwrap();
    let stacked_hue = CategoryValues::new("hue", ["left", "right", "left", "right"]).unwrap();
    let stacked_value = NumericValues::new("value", [5.0, 5.0, 1.0, 1.0]);
    let svg = render_svg(
        Plot::stacked_bar(stacked_category, stacked_value, stacked_hue)
            .facet(FacetSpec::column(stacked_facet)),
        "facet-stacked-domain",
    )
    .unwrap();
    assert_eq!(svg.matches(">10</text>").count(), 2);
    let first_marks = svg.find("<g clip-path=").unwrap();
    assert_eq!(svg[..first_marks].matches("-grid\"").count(), 2);

    let incompatible_facet = CategoryValues::new("facet", ["one", "one", "two", "two"]).unwrap();
    let incompatible_category = CategoryValues::new("category", ["a", "b", "a", "c"]).unwrap();
    let incompatible_value = NumericValues::new("value", [1.0, 2.0, 3.0, 4.0]);
    let error = Plot::bar(
        incompatible_category,
        BarInput::estimates(incompatible_value),
    )
    .facet(FacetSpec::column(incompatible_facet))
    .preflight()
    .unwrap_err();
    assert!(error
        .to_string()
        .contains("identical categorical levels and ordering"));
}

#[test]
fn long_label_facets_measure_gutters_and_wrap_strips_inside_owned_cells() {
    let category = "categorical level requiring measured gutter";
    let other = "another very wide category label requiring gutter";
    let left = "first panel with a deliberately long descriptive facet value";
    let right = "second panel with another deliberately long descriptive facet value";
    let facet =
        CategoryValues::new("facet_dimension_with_long_name", [left, left, right, right]).unwrap();
    let category = CategoryValues::new("category", [category, other, category, other]).unwrap();
    let value = NumericValues::new("value", [1.0, 2.0, 1.5, 2.5]);
    let svg = render_svg(
        Plot::horizontal_bar(category, BarInput::estimates(value))
            .facet(FacetSpec::column(facet))
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
fn crossed_facets_order_valid_hues_by_original_source_rows() {
    let row = CategoryValues::new("row", ["r1", "r2", "r2", "r1"]).unwrap();
    let column = CategoryValues::new("column", ["c1", "c2", "c2", "c1"]).unwrap();
    let x = NumericValues::new("x", [1.0, f64::NAN, 3.0, 2.0]);
    let y = NumericValues::new("y", [1.0, 2.0, 3.0, 2.0]);
    let trace = CategoryValues::new("trace", ["control", "dropped", "active", "treated"]).unwrap();
    let svg = render_svg(
        Plot::scatter(x, y)
            .hue(trace)
            .facet(FacetSpec::grid(row, column).unwrap())
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
    let facet = CategoryValues::new("facet", ["left", "left", "right", "right"]).unwrap();
    let group = CategoryValues::new("group", ["g1", "g1", "g2", "g2"]).unwrap();
    let hue = CategoryValues::new("hue", ["h1", "h1", "h2", "h2"]).unwrap();
    let x = NumericValues::new("x", [0.0, 1.0, 0.0, 1.0]);
    let y = NumericValues::new("y", [1.0, 2.0, 3.0, 4.0]);
    let svg = render_svg(
        Plot::line(x, y)
            .group(group)
            .hue(hue)
            .facet(FacetSpec::column(facet))
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
fn row_and_column_facets_require_distinct_variable_names() {
    let error = FacetSpec::grid(
        lichtung_plot::CategoryValues::new("facet", ["r1", "r2"]).unwrap(),
        lichtung_plot::CategoryValues::new("facet", ["c1", "c2"]).unwrap(),
    )
    .unwrap_err();

    assert!(matches!(error, lichtung_plot::Error::Data(_)));
    assert!(
        error.to_string().contains("must use different columns"),
        "{error}"
    );
}

#[test]
fn wrapped_facets_require_at_least_one_column() {
    let error = FacetSpec::wrap(
        lichtung_plot::CategoryValues::new("facet", ["left", "right"]).unwrap(),
        0,
    )
    .unwrap_err();

    assert!(matches!(error, lichtung_plot::Error::Data(_)));
    assert!(error.to_string().contains("greater than zero"), "{error}");
}

#[test]
fn faceted_explicit_category_order_validates_globally_and_filters_panel_subsets() {
    let panel = CategoryValues::new("panel", ["left", "right"]).unwrap();
    let category = CategoryValues::new("category", ["a", "b"]).unwrap();
    let value = NumericValues::new("value", [1.0, 2.0]);
    let plot = |order| {
        Plot::bar(category.clone(), BarInput::estimates(value.clone()))
            .category_order(CategoryOrder::explicit(order))
            .facet(FacetSpec::column(panel.clone()).scales(FacetScales::FreeX))
    };

    plot(["b", "a"]).preflight().unwrap();
    let error = plot(["b", "unknown"]).preflight().unwrap_err();

    assert!(error.to_string().contains("unknown level"));
}

#[test]
fn faceted_bar_rejects_panel_local_value_order() {
    let panel = CategoryValues::new("panel", ["left", "left", "right", "right"]).unwrap();
    let category = CategoryValues::new("category", ["a", "b", "a", "b"]).unwrap();
    let value = NumericValues::new("value", [1.0, 2.0, 3.0, 1.0]);
    let plot = Plot::bar(category, BarInput::estimates(value))
        .category_order(lichtung_plot::ValueSort::Descending)
        .facet(FacetSpec::column(panel));

    let error = plot.preflight().unwrap_err();

    assert!(error
        .to_string()
        .contains("value-derived category order is not supported with faceting"));
}

#[test]
fn faceted_errors_report_original_source_rows_before_output_creation() {
    let facet = CategoryValues::new("facet", ["left", "right", "left", "right"]).unwrap();
    let x = NumericValues::new("x", [1.0, 1.0, 2.0, 2.0]);
    let y = NumericValues::new("y", [1.0, 2.0, 3.0, f64::INFINITY]);
    let error = Plot::line(x, y)
        .facet(FacetSpec::column(facet.clone()))
        .preflight()
        .unwrap_err()
        .to_string();
    assert!(error.contains("value \"y\"") && error.contains("source row 3"));

    let category =
        CategoryValues::nullable("category", [Some("a"), Some("b"), Some("a"), None]).unwrap();
    let value = NumericValues::new("value", [1.0, 2.0, 3.0, 4.0]);
    let error = Plot::boxplot(category, value)
        .facet(FacetSpec::column(facet))
        .preflight()
        .unwrap_err()
        .to_string();
    assert!(error.contains("value \"category\"") && error.contains("source row 3"));
}

#[test]
fn facet_null_hue_contract_differs_between_scatter_and_strict_line() {
    let facet = CategoryValues::new("facet", ["one", "one", "two", "two"]).unwrap();
    let x = NumericValues::new("x", [1.0, 2.0, 1.0, 2.0]);
    let y = NumericValues::new("y", [1.0, 2.0, 2.0, 3.0]);
    let trace = CategoryValues::nullable(
        "trace",
        [Some("control"), None, Some("active"), Some("active")],
    )
    .unwrap();
    let scatter_error = Plot::scatter(x.clone(), y.clone())
        .hue(trace.clone())
        .facet(FacetSpec::column(facet.clone()))
        .preflight()
        .unwrap_err();
    assert!(scatter_error
        .to_string()
        .contains("value \"trace\" contains null at source row 1"));
    let line_error = Plot::line(x, y)
        .hue(trace)
        .facet(FacetSpec::column(facet))
        .preflight()
        .unwrap_err();
    assert!(line_error.to_string().contains("contains null"));
}
