use lichtung_plot::Error;
use lichtung_plot::{
    BarAggregation, LegendPosition, Plot, Significance, SignificanceMark, SignificanceMarkStyle,
    SignificanceTarget, TextWeight,
};

use super::{data, df, render_svg, DataFrame};

#[test]
fn rendered_brackets_stay_inside_axes_clear_data_and_share_only_disjoint_levels() {
    let df = df!(
        "group" => ["a", "b", "c", "d"],
        "value" => [2.0_f64, 4.0, 3.0, 5.0]
    )
    .unwrap();
    let comparisons = vec![
        comparison("c", "d", 0.01),
        comparison("a", "c", 0.01),
        comparison("a", "b", 0.01),
    ];
    let dir = tempdir("inside-and-levels");
    let annotated_path = dir.join("annotated.svg");
    let plain_path = dir.join("plain.svg");

    let annotated = identity_bar_chart(&df)
        .size(500, 360)
        .significance(comparisons);
    let annotated_panel = annotated.figure_layout().panel_box;
    annotated.svg(&annotated_path).unwrap();

    let plain = identity_bar_chart(&df).size(500, 360);
    let plain_panel = plain.figure_layout().panel_box;
    plain.svg(&plain_path).unwrap();

    let annotated_svg = std::fs::read_to_string(annotated_path).unwrap();
    let plain_svg = std::fs::read_to_string(plain_path).unwrap();
    let bracket_y = bracket_line_y_values(&annotated_svg);
    let data_top = polygon_top(&annotated_svg);
    let panel_top = 360.0 - annotated_panel.y1;

    assert_eq!(
        annotated_panel, plain_panel,
        "brackets must not shrink the panel"
    );
    assert_eq!(bracket_y.len(), 3);
    assert!((bracket_y[0] - bracket_y[1]).abs() < 0.001);
    assert!(bracket_y[2] < bracket_y[0] - 1.0);
    assert!(
        bracket_y.iter().all(|y| *y >= panel_top + 19.0),
        "bracket labels must remain inside the panel: panel_top={panel_top}, y={bracket_y:?}"
    );
    assert!(
        bracket_y[0] + 7.0 <= data_top - 5.0,
        "lowest bracket ticks must clear rendered bars: bracket_y={bracket_y:?}, data_top={data_top}"
    );
    assert!(
        data_top > polygon_top(&plain_svg) + 5.0,
        "automatic y range must add headroom rather than shrink the panel"
    );
}

#[test]
fn stacked_bar_renders_non_significant_pairwise_bracket() {
    let df = df!(
        "group" => ["a", "a", "b", "b"],
        "series" => ["positive", "negative", "positive", "negative"],
        "value" => [5.0_f64, -2.0, 3.0, -4.0]
    )
    .unwrap();
    let dir = tempdir("stacked-bar");
    let path = dir.join("stacked.svg");
    let chart = Plot::stacked_bar(
        data::category(&df, "group"),
        data::numeric(&df, "value"),
        data::category(&df, "series"),
    )
    .significance([comparison("a", "b", 0.05)])
    .size(420, 300);

    chart.svg(&path).unwrap();
    let svg = std::fs::read_to_string(path).unwrap();

    assert_eq!(svg.matches("class=\"significance-bracket\"").count(), 1);
    assert!(svg.contains(">ns</text>"));
}

#[test]
fn boxplot_bracket_clears_high_outlier_geometry() {
    let df = df!(
        "group" => ["a", "a", "a", "a", "a", "b", "b", "b", "b", "b"],
        "value" => [1.0_f64, 1.1, 1.2, 1.3, 10.0, 2.0, 2.1, 2.2, 2.3, 2.4]
    )
    .unwrap();
    let dir = tempdir("boxplot-outlier");
    let path = dir.join("boxplot.svg");
    let chart = Plot::boxplot(data::category(&df, "group"), data::numeric(&df, "value"))
        .significance([comparison("a", "b", 0.01)])
        .size(420, 300);

    chart.svg(&path).unwrap();
    let svg = std::fs::read_to_string(path).unwrap();
    let bracket_y = bracket_line_y_values(&svg)[0];
    let outlier_top = circle_top(&svg);

    assert!(
        bracket_y + 7.0 <= outlier_top - 3.0,
        "boxplot bracket must clear high flier: bracket_y={bracket_y}, outlier_top={outlier_top}"
    );
}

#[test]
fn violin_bracket_clears_rendered_kde_support() {
    let df = df!(
        "group" => ["a", "a", "a", "a", "b", "b", "b", "b"],
        "value" => [1.0_f64, 1.2, 1.4, 1.6, 3.0, 3.3, 3.6, 3.9]
    )
    .unwrap();
    let dir = tempdir("violin-support");
    let path = dir.join("violin.svg");
    let chart = Plot::violin(data::category(&df, "group"), data::numeric(&df, "value"))
        .significance([comparison("a", "b", 0.01)])
        .size(420, 300);

    chart.svg(&path).unwrap();
    let svg = std::fs::read_to_string(path).unwrap();
    let bracket_y = bracket_line_y_values(&svg)[0];
    let support_top = polygon_top(&svg);

    assert!(
        bracket_y + 7.0 <= support_top - 5.0,
        "violin bracket must clear KDE support: bracket_y={bracket_y}, support_top={support_top}"
    );
}

#[test]
fn inside_legend_resolved_box_does_not_overlap_bracket_or_label() {
    let svg = render_grouped_bar_with_legend(None, "inside", true);
    let (bracket_left, bracket_top, bracket_right, bracket_bottom) = bracket_bounds(&svg);
    let (legend_x, legend_y, legend_width, legend_height) = legend_rect(&svg);
    let legend_right = legend_x + legend_width;
    let legend_bottom = legend_y + legend_height;

    assert!(
        bracket_right <= legend_x
            || bracket_left >= legend_right
            || bracket_bottom <= legend_y
            || bracket_top >= legend_bottom,
        "bracket and label bounds must not overlap the inside legend"
    );
}

#[test]
fn outside_and_hidden_legends_do_not_add_annotation_headroom() {
    let hidden = render_grouped_bar_with_legend(Some(LegendPosition::Hidden), "hidden", true);
    let outside =
        render_grouped_bar_with_legend(Some(LegendPosition::OutsideRight), "outside-right", true);
    let inside = render_grouped_bar_with_legend(None, "inside-headroom", true);
    let hidden_y = bracket_line_y_values(&hidden)[0];
    let outside_y = bracket_line_y_values(&outside)[0];

    assert!((hidden_y - outside_y).abs() < 0.01);
    assert!(!hidden.contains("<g id=\"legend\">"));
    assert!(outside.contains("<g id=\"legend\">"));
    assert!(inside.contains("<g id=\"legend\">"));
}

#[test]
fn auto_bar_legend_keeps_top_right_beside_tall_uncompared_category() {
    let df = df!(
        "group" => ["a", "a", "b", "b", "c", "c"],
        "series" => ["control", "treatment", "control", "treatment", "control", "treatment"],
        "value" => [2.0_f64, 3.0, 4.0, 5.0, 100.0, 90.0]
    )
    .unwrap();
    let dir = tempdir("uncompared-global-maximum");
    let path = dir.join("bar.svg");
    let chart = identity_bar_chart(&df)
        .group(data::category(&df, "series"))
        .significance([comparison("a", "b", 0.01)])
        .size(460, 320);

    chart.svg(&path).unwrap();
    let svg = std::fs::read_to_string(path).unwrap();
    let (legend_x, _, _, _) = legend_rect(&svg);

    assert!(
        legend_x > 300.0,
        "bar legend should retain its semantic top-right default"
    );
}

#[test]
fn centered_inside_legend_uses_generic_geometry_with_comparisons() {
    let svg = render_grouped_bar_with_legend(
        Some(LegendPosition::InsideRight),
        "centered-inside-legend",
        true,
    );

    assert!(svg.contains("class=\"significance-bracket\""));
    assert!(svg.contains("<g id=\"legend\">"));
}

#[test]
fn bottom_inside_legend_never_moves_positive_bar_zero_minimum() {
    for with_comparison in [false, true] {
        let df = df!(
            "group" => ["a", "a", "b", "b"],
            "series" => ["control", "treatment", "control", "treatment"],
            "value" => [2.0_f64, 4.0, 3.0, 5.0]
        )
        .unwrap();
        let dir = tempdir("bottom-inside-zero-anchor");
        let path = dir.join("bar.svg");
        let mut chart = identity_bar_chart(&df)
            .hue(data::category(&df, "series"))
            .legend_position(LegendPosition::InsideBottomRight);
        if with_comparison {
            chart = chart.significance([comparison("a", "b", 0.01)]);
        }

        let error = chart.svg(&path).err().unwrap();
        assert!(error.to_string().contains("cannot fit inside the plot"));
    }
}

#[test]
fn categorical_x_limits_reject_visible_comparisons() {
    let df = df!(
        "group" => ["a", "b"],
        "value" => [2.0_f64, 4.0]
    )
    .unwrap();
    let dir = tempdir("comparison-x-limits");
    let path = dir.join("bar.svg");
    let chart = identity_bar_chart(&df)
        .x_limits(0.0, 2.0)
        .significance([comparison("a", "b", 0.01)]);

    let error = chart.svg(&path).err().unwrap();
    assert!(matches!(error, Error::Render(_)));
    assert!(error.to_string().contains("categorical x_limits"));
}

#[test]
fn horizontal_inside_legend_is_a_transposed_annotation_obstacle() {
    let svg = render_grouped_horizontal_bar_with_legend(None, "horizontal-inside");
    assert!(svg.contains("class=\"significance-bracket horizontal\""));
    assert!(svg.contains("<g id=\"legend\">"));
}

#[test]
fn horizontal_centered_inside_legend_uses_generic_geometry() {
    let svg = render_grouped_horizontal_bar_with_legend(
        Some(LegendPosition::InsideTop),
        "horizontal-centered",
    );

    assert!(svg.contains("class=\"significance-bracket horizontal\""));
    assert!(svg.contains("<g id=\"legend\">"));
}

#[test]
fn categorical_y_limits_reject_horizontal_visible_comparisons() {
    let df = df!(
        "group" => ["a", "b"],
        "value" => [2.0_f64, 4.0]
    )
    .unwrap();
    let dir = tempdir("horizontal-comparison-y-limits");
    let path = dir.join("horizontal.svg");
    let chart = identity_horizontal_bar_chart(&df)
        .y_limits(0.0, 2.0)
        .significance([comparison("a", "b", 0.01)]);

    let error = chart.svg(&path).err().unwrap();

    assert!(error.to_string().contains("categorical y_limits"));
}

#[test]
fn category_series_comparisons_render_exact_spans_in_both_orientations() {
    let df = df!(
        "outcome" => ["d-prime", "d-prime", "criterion", "criterion"],
        "condition" => ["baseline", "intervention", "baseline", "intervention"],
        "value" => [1.2_f64, 1.8, 0.4, 0.7]
    )
    .unwrap();
    let annotation = |category: &str, mark| {
        Significance::between(
            SignificanceTarget::category_series(category, "baseline"),
            SignificanceTarget::category_series(category, "intervention"),
            mark,
        )
    };

    let vertical = identity_bar_chart(&df)
        .group(data::category(&df, "condition"))
        .bar_aggregation(BarAggregation::Identity)
        .significance([
            annotation("d-prime", SignificanceMark::TwoStars),
            annotation("criterion", SignificanceMark::OneStar),
        ]);
    let vertical = render_svg(vertical, "grouped-factor-vertical").unwrap();

    let horizontal = identity_horizontal_bar_chart(&df)
        .hue(data::category(&df, "condition"))
        .bar_aggregation(BarAggregation::Identity)
        .significance([
            annotation("d-prime", SignificanceMark::TwoStars),
            annotation("criterion", SignificanceMark::OneStar),
        ]);
    let horizontal = render_svg(horizontal, "grouped-factor-horizontal").unwrap();

    assert_eq!(
        vertical.matches("class=\"significance-bracket\"").count(),
        2
    );
    assert_eq!(
        horizontal
            .matches("class=\"significance-bracket horizontal\"")
            .count(),
        2
    );
    assert!(vertical.contains(">**</text>") && vertical.contains(">*</text>"));
    assert!(horizontal.contains(">**</text>") && horizontal.contains(">*</text>"));
}

#[test]
fn attached_category_marks_do_not_create_missing_series_targets() {
    let df = df!(
        "outcome" => ["d-prime", "d-prime", "criterion"],
        "condition" => ["baseline", "intervention", "intervention"],
        "value" => [1.2_f64, 1.8, 0.7]
    )
    .unwrap();
    let annotation = Significance::between(
        SignificanceTarget::category_series("criterion", "baseline"),
        SignificanceTarget::category_series("criterion", "intervention"),
        SignificanceMark::OneStar,
    );
    let chart = identity_bar_chart(&df)
        .group(data::category(&df, "condition"))
        .bar_aggregation(BarAggregation::Identity)
        .significance([
            Significance::at("criterion", SignificanceMark::TwoStars),
            annotation,
        ]);

    let error = render_svg(chart, "missing-grouped-factor")
        .unwrap_err()
        .to_string();
    assert!(error.contains("has no rendered mark"), "{error}");
}

#[test]
fn category_and_center_series_cannot_form_a_zero_width_bracket() {
    let df = df!(
        "outcome" => ["d-prime", "d-prime", "d-prime"],
        "condition" => ["left", "center", "right"],
        "value" => [1.2_f64, 1.8, 1.4]
    )
    .unwrap();
    let annotation = Significance::between(
        "d-prime",
        SignificanceTarget::category_series("d-prime", "center"),
        SignificanceMark::OneStar,
    );
    let chart = identity_bar_chart(&df)
        .group(data::category(&df, "condition"))
        .bar_aggregation(BarAggregation::Identity)
        .significance([annotation]);

    let error = render_svg(chart, "same-grouped-anchor")
        .unwrap_err()
        .to_string();
    assert!(error.contains("same rendered anchor"), "{error}");
}

#[test]
fn exact_per_bar_marks_render_with_independent_weight_in_both_orientations() {
    let df = df!(
        "outcome" => ["positive", "negative"],
        "value" => [1.0_f64, -0.8]
    )
    .unwrap();
    let style = SignificanceMarkStyle::new(18, TextWeight::Semibold, 9.0).unwrap();
    let marks = || {
        [
            Significance::at("positive", SignificanceMark::TwoStars),
            Significance::at("negative", SignificanceMark::OneStar),
        ]
    };

    let vertical = identity_bar_chart(&df)
        .bar_aggregation(BarAggregation::Identity)
        .significance(marks())
        .significance_mark_style(style);
    let vertical = render_svg(vertical, "bar-marks-vertical").unwrap();

    let horizontal = identity_horizontal_bar_chart(&df)
        .bar_aggregation(BarAggregation::Identity)
        .significance(marks())
        .significance_mark_style(style);
    let horizontal = render_svg(horizontal, "bar-marks-horizontal").unwrap();

    for svg in [&vertical, &horizontal] {
        assert!(svg.contains(">**</text>") && svg.contains(">*</text>"));
        assert!(svg.contains("font-size=\"18\" font-weight=\"600\""));
    }
}

#[test]
fn default_per_bar_mark_weight_is_normal() {
    assert_eq!(
        SignificanceMarkStyle::default().weight(),
        TextWeight::Normal
    );
}

#[test]
fn unsupported_plot_kind_rejects_significance() {
    let df = df!(
        "x" => [1.0_f64, 2.0],
        "y" => [2.0_f64, 3.0]
    )
    .unwrap();
    let dir = tempdir("unsupported-comparison-kind");
    let path = dir.join("scatter.svg");
    let chart = Plot::scatter(data::numeric(&df, "x"), data::numeric(&df, "y"))
        .significance([comparison("a", "b", 0.01)]);

    let error = chart.svg(&path).err().unwrap();
    assert!(matches!(error, Error::Data(_)));
    assert!(error.to_string().contains("not supported for Scatter"));
}

#[test]
fn explicit_y_limits_succeed_when_sufficient_and_error_when_insufficient() {
    let df = df!(
        "group" => ["a", "b"],
        "value" => [2.0_f64, 4.0]
    )
    .unwrap();
    let comparison = comparison("a", "b", 0.01);
    let dir = tempdir("explicit-limits");

    let sufficient_path = dir.join("sufficient.svg");
    let sufficient = identity_bar_chart(&df)
        .y_limits(0.0, 10.0)
        .significance([comparison.clone()])
        .size(420, 300);
    sufficient.svg(&sufficient_path).unwrap();
    let sufficient_svg = std::fs::read_to_string(sufficient_path).unwrap();
    assert!(sufficient_svg.contains("class=\"significance-bracket\""));
    assert!(sufficient_svg.contains(">10</text>"));

    let insufficient_path = dir.join("insufficient.svg");
    let insufficient = identity_bar_chart(&df)
        .y_limits(0.0, 4.0)
        .significance([comparison])
        .size(420, 300);
    let error = insufficient.svg(&insufficient_path).err().unwrap();

    assert!(matches!(error, Error::Render(_)));
    assert!(error.to_string().contains("increase the upper limit"));
}

fn bracket_line_y_values(svg: &str) -> Vec<f64> {
    svg.lines()
        .filter(|line| line.contains("class=\"significance-bracket\""))
        .map(|line| {
            let path = attribute(line, "d");
            path.split_whitespace()
                .nth(2)
                .unwrap()
                .parse::<f64>()
                .unwrap()
        })
        .collect()
}

fn bracket_bounds(svg: &str) -> (f64, f64, f64, f64) {
    let line = svg
        .lines()
        .find(|line| line.contains("class=\"significance-bracket\""))
        .unwrap();
    let coordinates = attribute(line, "d")
        .split_whitespace()
        .filter_map(|token| token.parse::<f64>().ok())
        .collect::<Vec<_>>();
    let left = coordinates
        .iter()
        .step_by(2)
        .copied()
        .fold(f64::INFINITY, f64::min);
    let right = coordinates
        .iter()
        .step_by(2)
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    let bracket_y = coordinates[1];
    (left, bracket_y - 17.0, right, bracket_y + 7.0)
}

fn polygon_top(svg: &str) -> f64 {
    svg.lines()
        .filter(|line| line.starts_with("<polygon points="))
        .flat_map(|line| {
            attribute(line, "points")
                .split_whitespace()
                .map(|point| point.split_once(',').unwrap().1.parse::<f64>().unwrap())
                .collect::<Vec<_>>()
        })
        .fold(f64::INFINITY, f64::min)
}

fn circle_top(svg: &str) -> f64 {
    svg.lines()
        .filter(|line| line.starts_with("<circle cx="))
        .map(|line| {
            let y = attribute(line, "cy").parse::<f64>().unwrap();
            let radius = attribute(line, "r").parse::<f64>().unwrap();
            y - radius
        })
        .fold(f64::INFINITY, f64::min)
}

fn legend_rect(svg: &str) -> (f64, f64, f64, f64) {
    let line = svg
        .lines()
        .find(|line| line.starts_with("<rect x="))
        .unwrap();
    (
        attribute(line, "x").parse().unwrap(),
        attribute(line, "y").parse().unwrap(),
        attribute(line, "width").parse().unwrap(),
        attribute(line, "height").parse().unwrap(),
    )
}

fn render_grouped_bar_with_legend(
    position: Option<LegendPosition>,
    name: &str,
    with_comparison: bool,
) -> String {
    let df = df!(
        "group" => ["a", "a", "b", "b"],
        "series" => ["control", "treatment", "control", "treatment"],
        "value" => [2.0_f64, 4.0, 3.0, 5.0]
    )
    .unwrap();
    let dir = tempdir(name);
    let path = dir.join("bar.svg");
    let mut chart = identity_bar_chart(&df).group(data::category(&df, "series"));
    if let Some(position) = position {
        chart = chart.legend_position(position);
    }
    chart = chart.size(460, 320);
    if with_comparison {
        chart = chart.significance([comparison("a", "b", 0.01)]);
    }
    chart.svg(&path).unwrap();
    std::fs::read_to_string(path).unwrap()
}

fn render_grouped_horizontal_bar_with_legend(
    position: Option<LegendPosition>,
    name: &str,
) -> String {
    let df = df!(
        "group" => ["a", "a", "b", "b"],
        "series" => ["control", "treatment", "control", "treatment"],
        "value" => [2.0_f64, 4.0, 3.0, 5.0]
    )
    .unwrap();
    let dir = tempdir(name);
    let path = dir.join("horizontal.svg");
    let mut chart = identity_horizontal_bar_chart(&df).hue(data::category(&df, "series"));
    if let Some(position) = position {
        chart = chart.legend_position(position);
    }
    chart
        .significance([comparison("a", "b", 0.01)])
        .size(460, 320)
        .svg(&path)
        .unwrap();
    std::fs::read_to_string(path).unwrap()
}

fn attribute<'a>(line: &'a str, name: &str) -> &'a str {
    let start_marker = format!("{name}=\"");
    let start = line.find(&start_marker).unwrap() + start_marker.len();
    let end = line[start..].find('"').unwrap() + start;
    &line[start..end]
}

fn comparison(endpoint_a: &str, endpoint_b: &str, p_value: f64) -> Significance {
    Significance::between(endpoint_a, endpoint_b, SignificanceMark::PValue(p_value))
}

fn identity_bar_chart(data: &DataFrame) -> Plot {
    let category = if data.has_column("group") {
        "group"
    } else {
        "outcome"
    };
    Plot::bar(data::category(data, category), data::numeric(data, "value"))
        .bar_aggregation(BarAggregation::Identity)
}

fn identity_horizontal_bar_chart(data: &DataFrame) -> Plot {
    let category = if data.has_column("group") {
        "group"
    } else {
        "outcome"
    };
    Plot::horizontal_bar(data::category(data, category), data::numeric(data, "value"))
        .bar_aggregation(BarAggregation::Identity)
}

fn tempdir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "lichtung-significance-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}
