use lichtung_plot::Error;
use lichtung_plot::{
    BarInput, CategoryValues, LegendPolicy, LegendPosition, NumericValues, Pairwise, Plot,
    Significance, SignificanceMark, SignificanceMarkStyle, SignificanceTarget, TextWeight,
};

use super::render_svg;

#[derive(Clone)]
struct SignificanceBarData {
    category: CategoryValues,
    value: NumericValues,
    series: Option<CategoryValues>,
}

impl SignificanceBarData {
    fn new(category: CategoryValues, value: NumericValues) -> Self {
        Self {
            category,
            value,
            series: None,
        }
    }

    fn with_series(mut self, series: CategoryValues) -> Self {
        self.series = Some(series);
        self
    }
}

#[test]
fn rendered_brackets_stay_inside_axes_clear_data_and_share_only_disjoint_levels() {
    let data = SignificanceBarData::new(
        CategoryValues::new("group", ["a", "b", "c", "d"]).unwrap(),
        NumericValues::new("value", [2.0, 4.0, 3.0, 5.0]),
    );
    let comparisons = vec![
        comparison("c", "d", 0.01),
        comparison("a", "c", 0.01),
        comparison("a", "b", 0.01),
    ];
    let dir = tempdir("inside-and-levels");
    let annotated_path = dir.join("annotated.svg");
    let plain_path = dir.join("plain.svg");

    let annotated = identity_bar_chart(&data)
        .size(500, 360)
        .significance(comparisons);
    annotated.svg_preserve_text(&annotated_path).unwrap();

    let plain = identity_bar_chart(&data).size(500, 360);
    plain.svg_preserve_text(&plain_path).unwrap();

    let annotated_svg = std::fs::read_to_string(annotated_path).unwrap();
    let plain_svg = std::fs::read_to_string(plain_path).unwrap();
    let annotated_panel = clip_rect(&annotated_svg);
    let plain_panel = clip_rect(&plain_svg);
    let bracket_y = bracket_line_y_values(&annotated_svg);
    let data_top = polygon_top(&annotated_svg);
    let panel_top = annotated_panel.1;

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
    let group = CategoryValues::new("group", ["a", "a", "b", "b"]).unwrap();
    let series =
        CategoryValues::new("series", ["positive", "negative", "positive", "negative"]).unwrap();
    let value = NumericValues::new("value", [5.0, -2.0, 3.0, -4.0]);
    let dir = tempdir("stacked-bar");
    let path = dir.join("stacked.svg");
    let chart = Plot::stacked_bar(group, value, series)
        .significance([comparison("a", "b", 0.05)])
        .size(420, 300);

    chart.svg_preserve_text(&path).unwrap();
    let svg = std::fs::read_to_string(path).unwrap();

    assert_eq!(svg.matches("class=\"significance-bracket\"").count(), 1);
    assert!(svg.contains(">ns</text>"));
}

#[test]
fn boxplot_bracket_clears_high_outlier_geometry() {
    let group =
        CategoryValues::new("group", ["a", "a", "a", "a", "a", "b", "b", "b", "b", "b"]).unwrap();
    let value = NumericValues::new("value", [1.0, 1.1, 1.2, 1.3, 10.0, 2.0, 2.1, 2.2, 2.3, 2.4]);
    let dir = tempdir("boxplot-outlier");
    let path = dir.join("boxplot.svg");
    let chart = Plot::boxplot(group, value)
        .significance([comparison("a", "b", 0.01)])
        .size(420, 300);

    chart.svg_preserve_text(&path).unwrap();
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
    let group = CategoryValues::new("group", ["a", "a", "a", "a", "b", "b", "b", "b"]).unwrap();
    let value = NumericValues::new("value", [1.0, 1.2, 1.4, 1.6, 3.0, 3.3, 3.6, 3.9]);
    let dir = tempdir("violin-support");
    let path = dir.join("violin.svg");
    let chart = Plot::violin(group, value)
        .significance([comparison("a", "b", 0.01)])
        .size(420, 300);

    chart.svg_preserve_text(&path).unwrap();
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
    let hidden = render_grouped_bar_with_legend(Some(LegendPolicy::Hidden), "hidden", true);
    let outside = render_grouped_bar_with_legend(
        Some(LegendPolicy::Visible(LegendPosition::OutsideRight)),
        "outside-right",
        true,
    );
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
    let data = SignificanceBarData::new(
        CategoryValues::new("group", ["a", "a", "b", "b", "c", "c"]).unwrap(),
        NumericValues::new("value", [2.0, 3.0, 4.0, 5.0, 100.0, 90.0]),
    )
    .with_series(
        CategoryValues::new(
            "series",
            [
                "control",
                "treatment",
                "control",
                "treatment",
                "control",
                "treatment",
            ],
        )
        .unwrap(),
    );
    let dir = tempdir("uncompared-global-maximum");
    let path = dir.join("bar.svg");
    let chart = identity_bar_chart(&data)
        .series(data.series.clone().unwrap())
        .significance([comparison("a", "b", 0.01)])
        .size(460, 320);

    chart.svg_preserve_text(&path).unwrap();
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
        Some(LegendPolicy::Visible(LegendPosition::InsideRight)),
        "centered-inside-legend",
        true,
    );

    assert!(svg.contains("class=\"significance-bracket\""));
    assert!(svg.contains("<g id=\"legend\">"));
}

#[test]
fn bottom_inside_legend_never_moves_positive_bar_zero_minimum() {
    for with_comparison in [false, true] {
        let data = grouped_bar_data();
        let dir = tempdir("bottom-inside-zero-anchor");
        let path = dir.join("bar.svg");
        let mut chart = identity_bar_chart(&data)
            .series(data.series.clone().unwrap())
            .legend(LegendPolicy::Visible(LegendPosition::InsideBottomRight));
        if with_comparison {
            chart = chart.significance([comparison("a", "b", 0.01)]);
        }

        let error = chart.svg(&path).err().unwrap();
        assert!(error.to_string().contains("cannot fit inside the plot"));
    }
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
        Some(LegendPolicy::Visible(LegendPosition::InsideTop)),
        "horizontal-centered",
    );

    assert!(svg.contains("class=\"significance-bracket horizontal\""));
    assert!(svg.contains("<g id=\"legend\">"));
}

#[test]
fn category_series_comparisons_render_exact_spans_in_both_orientations() {
    let data = SignificanceBarData::new(
        CategoryValues::new("outcome", ["d-prime", "d-prime", "criterion", "criterion"]).unwrap(),
        NumericValues::new("value", [1.2, 1.8, 0.4, 0.7]),
    )
    .with_series(
        CategoryValues::new(
            "condition",
            ["baseline", "intervention", "baseline", "intervention"],
        )
        .unwrap(),
    );
    let annotation = |category: &str, mark| {
        Significance::between(
            SignificanceTarget::category_series(category, "baseline"),
            SignificanceTarget::category_series(category, "intervention"),
            mark,
        )
    };

    let vertical = identity_bar_chart(&data)
        .series(data.series.clone().unwrap())
        .significance([
            annotation("d-prime", SignificanceMark::TwoStars),
            annotation("criterion", SignificanceMark::OneStar),
        ]);
    let vertical = render_svg(vertical, "grouped-factor-vertical").unwrap();

    let horizontal = identity_horizontal_bar_chart(&data)
        .series(data.series.clone().unwrap())
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
    let data = SignificanceBarData::new(
        CategoryValues::new("outcome", ["d-prime", "d-prime", "criterion"]).unwrap(),
        NumericValues::new("value", [1.2, 1.8, 0.7]),
    )
    .with_series(
        CategoryValues::new("condition", ["baseline", "intervention", "intervention"]).unwrap(),
    );
    let annotation = Significance::between(
        SignificanceTarget::category_series("criterion", "baseline"),
        SignificanceTarget::category_series("criterion", "intervention"),
        SignificanceMark::OneStar,
    );
    let chart = identity_bar_chart(&data)
        .series(data.series.clone().unwrap())
        .significance([Significance::at("criterion", SignificanceMark::TwoStars)])
        .significance([annotation]);

    let error = render_svg(chart, "missing-grouped-factor")
        .unwrap_err()
        .to_string();
    assert!(error.contains("has no rendered mark"), "{error}");
}

#[test]
fn category_and_center_series_cannot_form_a_zero_width_bracket() {
    let data = SignificanceBarData::new(
        CategoryValues::new("outcome", ["d-prime", "d-prime", "d-prime"]).unwrap(),
        NumericValues::new("value", [1.2, 1.8, 1.4]),
    )
    .with_series(CategoryValues::new("condition", ["left", "center", "right"]).unwrap());
    let annotation = Significance::between(
        "d-prime",
        SignificanceTarget::category_series("d-prime", "center"),
        SignificanceMark::OneStar,
    );
    let chart = identity_bar_chart(&data)
        .series(data.series.clone().unwrap())
        .significance([annotation]);

    let error = render_svg(chart, "same-grouped-anchor")
        .unwrap_err()
        .to_string();
    assert!(error.contains("same rendered anchor"), "{error}");
}

#[test]
fn exact_per_bar_marks_render_with_independent_weight_in_both_orientations() {
    let data = SignificanceBarData::new(
        CategoryValues::new("outcome", ["positive", "negative"]).unwrap(),
        NumericValues::new("value", [1.0, -0.8]),
    );
    let style = SignificanceMarkStyle::default()
        .font_size(18.0)
        .unwrap()
        .weight(TextWeight::Semibold)
        .unwrap()
        .gap(9.0)
        .unwrap();
    let marks = || {
        [
            Significance::at("positive", SignificanceMark::TwoStars),
            Significance::at("negative", SignificanceMark::OneStar),
        ]
    };

    let vertical = identity_bar_chart(&data)
        .significance(marks())
        .significance_mark_style(style);
    let vertical = render_svg(vertical, "bar-marks-vertical").unwrap();

    let horizontal = identity_horizontal_bar_chart(&data)
        .significance(marks())
        .significance_mark_style(style);
    let horizontal = render_svg(horizontal, "bar-marks-horizontal").unwrap();

    for svg in [&vertical, &horizontal] {
        assert!(svg.contains(">**</text>") && svg.contains(">*</text>"));
        assert!(svg.contains("font-size=\"18\" font-weight=\"600\""));
    }
}

#[test]
fn ordinary_bar_accumulates_attached_and_pairwise_placements_across_calls() {
    let data = SignificanceBarData::new(
        CategoryValues::new("group", ["a", "b"]).unwrap(),
        NumericValues::new("value", [1.0, 2.0]),
    );
    let chart = identity_bar_chart(&data)
        .significance([Significance::at("a", SignificanceMark::OneStar)])
        .significance([Significance::between("a", "b", SignificanceMark::TwoStars)])
        .size(640, 480);

    let svg = render_svg(chart, "bar-mixed-significance-calls").unwrap();

    assert!(svg.contains(">*</text>"));
    assert!(svg.contains(">**</text>"));
    assert_eq!(svg.matches("class=\"significance-bracket\"").count(), 1);
}

#[test]
fn explicit_y_limits_succeed_when_sufficient_and_error_when_insufficient() {
    let data = SignificanceBarData::new(
        CategoryValues::new("group", ["a", "b"]).unwrap(),
        NumericValues::new("value", [2.0, 4.0]),
    );
    let comparison = comparison("a", "b", 0.01);
    let dir = tempdir("explicit-limits");

    let sufficient_path = dir.join("sufficient.svg");
    let sufficient = identity_bar_chart(&data)
        .y_limits(0.0, 10.0)
        .significance([comparison.clone()])
        .size(420, 300);
    sufficient.svg_preserve_text(&sufficient_path).unwrap();
    let sufficient_svg = std::fs::read_to_string(sufficient_path).unwrap();
    assert!(sufficient_svg.contains("class=\"significance-bracket\""));
    assert!(sufficient_svg.contains(">10</text>"));

    let insufficient_path = dir.join("insufficient.svg");
    let insufficient = identity_bar_chart(&data)
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
    let legend = svg
        .split_once("<g id=\"legend\">")
        .expect("legend layer")
        .1
        .split_once("</g>")
        .expect("legend layer end")
        .0;
    let line = legend
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
    policy: Option<LegendPolicy>,
    name: &str,
    with_comparison: bool,
) -> String {
    let data = grouped_bar_data();
    let dir = tempdir(name);
    let path = dir.join("bar.svg");
    let mut chart = identity_bar_chart(&data).series(data.series.clone().unwrap());
    if let Some(policy) = policy {
        chart = chart.legend(policy);
    }
    chart = chart.size(460, 320);
    if with_comparison {
        chart = chart.significance([comparison("a", "b", 0.01)]);
    }
    chart.svg_preserve_text(&path).unwrap();
    std::fs::read_to_string(path).unwrap()
}

fn render_grouped_horizontal_bar_with_legend(policy: Option<LegendPolicy>, name: &str) -> String {
    let data = grouped_bar_data();
    let dir = tempdir(name);
    let path = dir.join("horizontal.svg");
    let mut chart = identity_horizontal_bar_chart(&data).series(data.series.clone().unwrap());
    if let Some(policy) = policy {
        chart = chart.legend(policy);
    }
    chart
        .significance([comparison("a", "b", 0.01)])
        .size(460, 320)
        .svg_preserve_text(&path)
        .unwrap();
    std::fs::read_to_string(path).unwrap()
}

fn attribute<'a>(line: &'a str, name: &str) -> &'a str {
    let start_marker = format!("{name}=\"");
    let start = line.find(&start_marker).unwrap() + start_marker.len();
    let end = line[start..].find('"').unwrap() + start;
    &line[start..end]
}

fn clip_rect(svg: &str) -> (f64, f64, f64, f64) {
    let clip = &svg[svg.find("<clipPath id=").unwrap()..];
    let rect = clip.lines().find(|line| line.contains("<rect ")).unwrap();
    (
        attribute(rect, "x").parse().unwrap(),
        attribute(rect, "y").parse().unwrap(),
        attribute(rect, "width").parse().unwrap(),
        attribute(rect, "height").parse().unwrap(),
    )
}

fn comparison(endpoint_a: &str, endpoint_b: &str, p_value: f64) -> Significance<Pairwise> {
    Significance::between(endpoint_a, endpoint_b, SignificanceMark::PValue(p_value))
}

fn grouped_bar_data() -> SignificanceBarData {
    SignificanceBarData::new(
        CategoryValues::new("group", ["a", "a", "b", "b"]).unwrap(),
        NumericValues::new("value", [2.0, 4.0, 3.0, 5.0]),
    )
    .with_series(
        CategoryValues::new("series", ["control", "treatment", "control", "treatment"]).unwrap(),
    )
}

fn identity_bar_chart(data: &SignificanceBarData) -> Plot<lichtung_plot::kind::Bar> {
    Plot::bar(
        data.category.clone(),
        BarInput::estimates(data.value.clone()),
    )
}

fn identity_horizontal_bar_chart(
    data: &SignificanceBarData,
) -> Plot<lichtung_plot::kind::HorizontalBar> {
    Plot::horizontal_bar(
        data.category.clone(),
        BarInput::estimates(data.value.clone()),
    )
}

static TEMP_DIRECTORY_SEQUENCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn tempdir(name: &str) -> std::path::PathBuf {
    loop {
        let sequence = TEMP_DIRECTORY_SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let directory = std::env::temp_dir().join(format!(
            "lichtung-significance-{name}-{}-{}",
            std::process::id(),
            sequence
        ));
        match std::fs::create_dir(&directory) {
            Ok(()) => return directory,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => panic!("tempdir: {error}"),
        }
    }
}
