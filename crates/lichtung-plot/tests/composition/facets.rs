use super::*;

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
