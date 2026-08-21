use super::*;

#[test]
fn scatter_smoke_snapshot() {
    let df = fixture_df();
    let dir = tempdir();
    let out = dir.join("scatter.svg");
    let mut chart = Chart::new(&df);
    chart.x("x").y("y").size(320, 240).scatter();
    chart.render_to_svg(&out).expect("render must succeed");
    let bytes = std::fs::read(&out).expect("file must exist");
    insta::assert_snapshot!(normalize_embedded_font(&String::from_utf8_lossy(&bytes)));
}

#[test]
fn matrix_clip_structure_snapshot() {
    let df = df!(
        "x" => [1.0_f64, 2.0, 3.0, 4.0],
        "y" => [4.0_f64, 3.0, 2.0, 1.0],
        "group" => ["a", "a", "b", "b"]
    )
    .unwrap();
    let dir = tempdir();
    let out = dir.join("matrix-structure.svg");
    let mut chart = Chart::new(&df)
        .group("group")
        .hue("group")
        .title("Matrix structural title");
    chart
        .size(360, 280)
        .matrix_variables(vec!["x", "y"])
        .matrix();

    chart.render_to_svg(&out).unwrap();
    let svg = std::fs::read_to_string(out).unwrap();

    insta::assert_snapshot!(specialized_svg_structure(
        &svg,
        &["Matrix structural title", "Group"]
    ));
}

#[test]
fn beeswarm_clip_structure_snapshot() {
    let df = df!(
        "category" => ["first", "first", "second", "second"],
        "value" => [-2.0_f64, -1.0, 1.0, 2.0],
        "color" => [0.1_f64, 0.3, 0.7, 1.0]
    )
    .unwrap();
    let dir = tempdir();
    let out = dir.join("beeswarm-structure.svg");
    let mut chart = Chart::new(&df)
        .color("color")
        .title("Beeswarm structural title");
    chart.x("value").y("category").size(360, 280).beeswarm();

    chart.render_to_svg(&out).unwrap();
    let svg = std::fs::read_to_string(out).unwrap();

    insta::assert_snapshot!(specialized_svg_structure(
        &svg,
        &["Beeswarm structural title", "Color", "0.100", "1.000"]
    ));
}

#[test]
fn horizontal_bar_clip_structure_snapshot() {
    let df = df!(
        "category" => ["a deliberately long category", "short"],
        "value" => [2.0_f64, 4.0]
    )
    .unwrap();
    let dir = tempdir();
    let out = dir.join("horizontal-structure.svg");
    let mut chart = Chart::new(&df)
        .title("Horizontal structural title")
        .bar_aggregation(BarAggregation::Identity);
    chart
        .x("category")
        .y("value")
        .size(360, 280)
        .horizontal_bar();

    chart.render_to_svg(&out).unwrap();
    let svg = std::fs::read_to_string(out).unwrap();

    insta::assert_snapshot!(specialized_svg_structure(
        &svg,
        &[
            "Horizontal structural title",
            "A deliberately long category"
        ]
    ));
}

#[test]
fn marginal_clip_structure_snapshot() {
    let df = df!(
        "x" => [1.0_f64, 2.0, 3.0, 4.0],
        "y" => [1.0_f64, 4.0, 2.0, 3.0]
    )
    .unwrap();
    let dir = tempdir();
    let out = dir.join("marginal-structure.svg");
    let mut chart = Chart::new(&df).title("Marginal structural title");
    chart.x("x").y("y").size(360, 280).marginal();

    chart.render_to_svg(&out).unwrap();
    let svg = std::fs::read_to_string(out).unwrap();

    insta::assert_snapshot!(specialized_svg_structure(
        &svg,
        &["Marginal structural title"]
    ));
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one visual manifest keeps representative plot families and features together"
)]
fn visual_reference_manifest() {
    let data = df!(
        "x" => [1.0_f64, 2.0, 1.0, 2.0],
        "estimate" => [1.1_f64, 1.3, 1.2, 1.5],
        "lower" => [1.0_f64, 1.2, 1.1, 1.4],
        "upper" => [1.2_f64, 1.4, 1.3, 1.6],
        "factor" => ["before", "after", "before", "after"],
        "trace" => ["control", "control", "active", "active"],
        "term" => ["intercept", "time", "group", "interaction"],
        "facet" => ["one", "one", "two", "two"]
    )
    .unwrap();
    let interval = || {
        IntervalSemantics::new(
            IntervalKind::Confidence,
            0.95,
            InferentialScale::Response,
            "visual fixture",
        )
        .unwrap()
    };
    let dir = tempdir();
    let mut references = Vec::new();

    let mut line = Chart::new(&data)
        .group("trace")
        .hue("trace")
        .interval_bounds("lower", "upper");
    line.x("x").y("estimate").size(420, 300).line();
    references.push(render_visual_reference("line", &line, &dir));

    let mut interaction = Chart::new(&data)
        .group("trace")
        .hue("trace")
        .interval_bounds("lower", "upper")
        .interval_semantics(interval())
        .interaction_source(EstimateSource::EstimatedMarginalMean);
    interaction
        .x("factor")
        .y("estimate")
        .size(420, 300)
        .interaction();
    references.push(render_visual_reference("interaction", &interaction, &dir));

    let mut coefficient = Chart::new(&data)
        .interval_bounds("lower", "upper")
        .interval_semantics(
            IntervalSemantics::new(
                IntervalKind::Confidence,
                0.95,
                InferentialScale::Coefficient,
                "visual fixture",
            )
            .unwrap(),
        );
    coefficient
        .x("estimate")
        .y("term")
        .size(420, 300)
        .coefficient();
    references.push(render_visual_reference("coefficient", &coefficient, &dir));

    let mut forest = Chart::new(&data)
        .interval_bounds("lower", "upper")
        .interval_semantics(
            IntervalSemantics::new(
                IntervalKind::Confidence,
                0.95,
                InferentialScale::Ratio,
                "visual fixture",
            )
            .unwrap(),
        )
        .reference_line(1.0)
        .axis_scale(AxisScale::Log10);
    forest.x("estimate").y("term").size(420, 300).forest();
    references.push(render_visual_reference("forest", &forest, &dir));

    let comparison = ComparisonAnnotation {
        endpoint_a: ComparisonEndpoint::new("before")
            .with_group("control")
            .with_hue("control"),
        endpoint_b: ComparisonEndpoint::new("before")
            .with_group("active")
            .with_hue("active"),
        ..ComparisonAnnotation::new("before", "before", 0.01)
    };
    let mut grouped = Chart::new(&data)
        .group("trace")
        .hue("trace")
        .bar_aggregation(BarAggregation::Identity);
    grouped
        .x("factor")
        .y("estimate")
        .add_comparisons([comparison.clone()])
        .size(420, 300)
        .bar();
    references.push(render_visual_reference(
        "grouped-significance",
        &grouped,
        &dir,
    ));

    let mut horizontal = Chart::new(&data)
        .group("trace")
        .hue("trace")
        .bar_aggregation(BarAggregation::Identity);
    horizontal
        .x("factor")
        .y("estimate")
        .add_comparisons([comparison])
        .size(420, 300)
        .horizontal_bar();
    references.push(render_visual_reference(
        "horizontal-significance",
        &horizontal,
        &dir,
    ));

    let mut fixed_facets = Chart::new(&data)
        .group("trace")
        .hue("trace")
        .facet_col("facet");
    fixed_facets.x("x").y("estimate").size(700, 320).line();
    references.push(render_visual_reference(
        "faceting-fixed",
        &fixed_facets,
        &dir,
    ));

    let mut free_facets = Chart::new(&data)
        .group("trace")
        .hue("trace")
        .facet_col("facet")
        .facet_scales(FacetScales::Free);
    free_facets.x("x").y("estimate").size(700, 320).line();
    references.push(render_visual_reference("faceting-free", &free_facets, &dir));

    let mut figure_line = Chart::new(&data).group("trace").hue("trace");
    figure_line.x("x").y("estimate").line();
    let mut figure_coefficient = Chart::new(&data);
    figure_coefficient.x("estimate").y("term").coefficient();
    let figure = Figure::grid(1, 2)
        .unwrap()
        .panel(0, 0, figure_line)
        .unwrap()
        .panel(0, 1, figure_coefficient)
        .unwrap()
        .legend_policy(FigureLegendPolicy::Shared)
        .size(700, 320);
    let output = dir.join("composition.svg");
    figure.render_to_svg(&output).unwrap();
    references.push(format!(
        "composition\n{}",
        normalize_reference_svg(&std::fs::read_to_string(output).unwrap())
    ));

    insta::assert_snapshot!(references.join("\n\n"));
}
