use lichtung_plot::layout::PanelBox;
use lichtung_plot::render::backend::{
    Backend, BracketStyle, FillStyle, GradientStop, LineStyle, Point2D, PointStyle,
    SignificanceBracket, TextStyle,
};
use lichtung_plot::Error;
use lichtung_plot::{Chart, Figure, LineMode, PlotKind, Theme};
use polars::prelude::{DataFrame, NamedFrom, Series};

#[derive(Debug, Clone, PartialEq)]
struct RecordedLine {
    points: Vec<Point2D>,
    color: (u8, u8, u8),
    width: f32,
}

#[derive(Default)]
struct RecordingBackend {
    begins: usize,
    ends: usize,
    clip_depth: usize,
    lines: Vec<RecordedLine>,
    scatters: Vec<(Point2D, (u8, u8, u8), u32)>,
    texts: Vec<String>,
}

impl Backend for RecordingBackend {
    fn begin(&mut self, _width: u32, _height: u32) -> Result<(), Error> {
        self.begins += 1;
        Ok(())
    }

    fn begin_clip(&mut self, _region: PanelBox) -> Result<(), Error> {
        self.clip_depth += 1;
        Ok(())
    }

    fn end_clip(&mut self) -> Result<(), Error> {
        self.clip_depth -= 1;
        Ok(())
    }

    fn draw_line(&mut self, points: &[Point2D], style: &LineStyle) -> Result<(), Error> {
        if self.clip_depth > 0 {
            self.lines.push(RecordedLine {
                points: points.to_vec(),
                color: style.color,
                width: style.width,
            });
        }
        Ok(())
    }

    fn draw_scatter(&mut self, points: &[Point2D], style: &PointStyle) -> Result<(), Error> {
        if self.clip_depth > 0 {
            self.scatters.extend(
                points
                    .iter()
                    .copied()
                    .map(|point| (point, style.color, style.radius)),
            );
        }
        Ok(())
    }

    fn draw_polygon(&mut self, _points: &[Point2D], _style: &FillStyle) -> Result<(), Error> {
        Ok(())
    }

    fn draw_linear_gradient(
        &mut self,
        _bounds: PanelBox,
        _start: Point2D,
        _end: Point2D,
        _stops: &[GradientStop],
    ) -> Result<(), Error> {
        Ok(())
    }

    fn draw_text(
        &mut self,
        _x_pixel: u32,
        _y_pixel: u32,
        text: &str,
        _style: &TextStyle,
    ) -> Result<(), Error> {
        self.texts.push(text.to_owned());
        Ok(())
    }

    fn draw_bracket(
        &mut self,
        _bracket: &SignificanceBracket,
        _style: &BracketStyle,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn end(&mut self) -> Result<(), Error> {
        self.ends += 1;
        Ok(())
    }
}

fn numeric_trajectory_data() -> DataFrame {
    DataFrame::new(
        4,
        vec![
            Series::new("subject".into(), ["s1", "s2", "s1", "s2"]).into(),
            Series::new("condition".into(), [2.0_f64, 2.0, 1.0, 1.0]).into(),
            Series::new("value".into(), [20.0_f64, 40.0, 10.0, 30.0]).into(),
            Series::new("status".into(), ["case", "control", "control", "case"]).into(),
        ],
    )
    .unwrap()
}

fn trajectory_chart(data: &DataFrame) -> Chart<'_> {
    let mut chart = Chart::new(data)
        .group("subject")
        .hue("status")
        .line_mode(LineMode::Trajectory);
    let _ = chart.x("condition");
    let _ = chart.y("value");
    let _ = chart.select(PlotKind::Line);
    chart
}

fn categorical_trajectory_data() -> DataFrame {
    DataFrame::new(
        2,
        vec![
            Series::new("subject".into(), ["s1", "s1"]).into(),
            Series::new("condition".into(), ["before", "after"]).into(),
            Series::new("value".into(), [100.0_f64, 200.0]).into(),
        ],
    )
    .unwrap()
}

#[test]
fn numeric_trajectory_sorts_within_groups_and_never_connects_groups() {
    let data = numeric_trajectory_data();
    let chart = trajectory_chart(&data);
    let mut backend = RecordingBackend::default();

    chart.render_with_backend(&mut backend).unwrap();

    let connectors = backend
        .lines
        .iter()
        .filter(|line| line.width.to_bits() == 1.4_f32.to_bits())
        .collect::<Vec<_>>();
    assert_eq!(connectors.len(), 2);
    assert!(connectors.iter().all(|line| line.points.len() == 2));
    assert!(connectors
        .iter()
        .all(|line| line.points[0].x < line.points[1].x));
    assert_eq!((backend.begins, backend.ends), (1, 1));
}

#[test]
fn numeric_path_preserves_source_order_and_repeated_x_values() {
    let data = DataFrame::new(
        4,
        vec![
            Series::new("curve".into(), ["roc", "roc", "roc", "roc"]).into(),
            Series::new("rate".into(), [0.0_f64, 0.0, 1.0, 0.5]).into(),
            Series::new("sensitivity".into(), [0.0_f64, 0.4, 1.0, 0.8]).into(),
        ],
    )
    .unwrap();
    let mut chart = Chart::new(&data).group("curve").line_mode(LineMode::Path);
    let _ = chart.x("rate");
    let _ = chart.y("sensitivity");
    let _ = chart.select(PlotKind::Line);
    let mut backend = RecordingBackend::default();

    chart.render_with_backend(&mut backend).unwrap();

    let connector = backend
        .lines
        .iter()
        .find(|line| line.width.to_bits() == 1.4_f32.to_bits())
        .unwrap();
    assert_eq!(connector.points.len(), 4);
    assert_eq!(
        connector.points[0].x.to_bits(),
        connector.points[1].x.to_bits()
    );
    assert!(connector.points[2].x > connector.points[3].x);
    assert_eq!(backend.scatters.len(), 4);
}

#[test]
fn path_rejects_categorical_x_before_backend_begin() {
    let data = categorical_trajectory_data();
    let mut chart = Chart::new(&data).group("subject").line_mode(LineMode::Path);
    let _ = chart.x("condition");
    let _ = chart.y("value");
    let _ = chart.select(PlotKind::Line);
    let mut backend = RecordingBackend::default();

    let error = chart.render_with_backend(&mut backend).unwrap_err();

    assert!(error.to_string().contains("line path: x must be numeric"));
    assert_eq!(backend.begins, 0);
}

#[test]
fn changing_hue_within_one_group_colors_points_but_not_connector_or_group_legend() {
    let data = DataFrame::new(
        3,
        vec![
            Series::new(
                "subject".into(),
                ["subject-one", "subject-one", "subject-one"],
            )
            .into(),
            Series::new("condition".into(), [1.0_f64, 2.0, 3.0]).into(),
            Series::new("value".into(), [2.0_f64, 5.0, 3.0]).into(),
            Series::new("status".into(), ["control", "case", "control"]).into(),
        ],
    )
    .unwrap();
    let chart = trajectory_chart(&data);
    let mut backend = RecordingBackend::default();

    chart.render_with_backend(&mut backend).unwrap();

    let palette = Theme::Publication.palette();
    let connector = backend
        .lines
        .iter()
        .find(|line| line.width.to_bits() == 1.4_f32.to_bits())
        .unwrap();
    assert_eq!(connector.color, palette.foreground);
    assert_eq!(
        backend
            .scatters
            .iter()
            .map(|(_, color, _)| *color)
            .collect::<Vec<_>>(),
        [palette.colors[0], palette.colors[1], palette.colors[0]]
    );
    assert!(backend.texts.iter().any(|text| text == "Control"));
    assert!(backend.texts.iter().any(|text| text == "Case"));
    assert!(!backend
        .texts
        .iter()
        .any(|text| text == "subject-one" || text == "Subject one"));
}

#[test]
fn categorical_trajectory_uses_global_first_observed_condition_order() {
    let data = DataFrame::new(
        3,
        vec![
            Series::new("subject".into(), ["s1", "s1", "s1"]).into(),
            Series::new("condition".into(), ["after", "before", "during"]).into(),
            Series::new("value".into(), [30.0_f64, 10.0, 20.0]).into(),
        ],
    )
    .unwrap();
    let mut chart = Chart::new(&data)
        .group("subject")
        .line_mode(LineMode::Trajectory);
    let _ = chart.x("condition");
    let _ = chart.y("value");
    let _ = chart.select(PlotKind::Line);
    let mut backend = RecordingBackend::default();

    chart.render_with_backend(&mut backend).unwrap();

    let connector = backend
        .lines
        .iter()
        .find(|line| line.width.to_bits() == 1.4_f32.to_bits())
        .unwrap();
    assert!(connector.points[0].x < connector.points[1].x);
    assert!(connector.points[1].x < connector.points[2].x);
    assert!(connector.points[0].y > connector.points[2].y);
    assert!(connector.points[2].y > connector.points[1].y);
    let condition_ticks = backend
        .texts
        .iter()
        .filter(|text| ["After", "Before", "During"].contains(&text.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(condition_ticks, ["After", "Before", "During"]);
}

#[test]
fn categorical_conditions_collapse_whitespace_before_ordering_and_ticks() {
    let data = DataFrame::new(
        4,
        vec![
            Series::new("subject".into(), ["s1", "s1", "s2", "s2"]).into(),
            Series::new(
                "condition".into(),
                [
                    "  before\tvisit ",
                    "after visit",
                    "before visit",
                    " after   visit ",
                ],
            )
            .into(),
            Series::new("value".into(), [1.0_f64, 2.0, 3.0, 4.0]).into(),
        ],
    )
    .unwrap();
    let mut chart = Chart::new(&data)
        .group("subject")
        .line_mode(LineMode::Trajectory);
    let _ = chart.x("condition");
    let _ = chart.y("value");
    let _ = chart.select(PlotKind::Line);
    let mut backend = RecordingBackend::default();

    chart.render_with_backend(&mut backend).unwrap();

    let ticks = backend
        .texts
        .iter()
        .filter(|text| ["Before visit", "After visit"].contains(&text.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(ticks, ["Before visit", "After visit"]);
}

#[test]
fn categorical_condition_duplicates_reject_after_whitespace_normalization() {
    let data = DataFrame::new(
        2,
        vec![
            Series::new("subject".into(), ["s1", "s1"]).into(),
            Series::new("condition".into(), ["before visit", " before\tvisit "]).into(),
            Series::new("value".into(), [1.0_f64, 2.0]).into(),
        ],
    )
    .unwrap();
    let mut chart = Chart::new(&data)
        .group("subject")
        .line_mode(LineMode::Trajectory);
    let _ = chart.x("condition");
    let _ = chart.y("value");
    let _ = chart.select(PlotKind::Line);
    let mut backend = RecordingBackend::default();

    let error = chart.render_with_backend(&mut backend).unwrap_err();

    assert!(
        error.to_string().contains("duplicate normalized"),
        "{error}"
    );
    assert!(error.to_string().contains("source row 1"), "{error}");
    assert_eq!(backend.begins, 0);
}

#[test]
fn categorical_condition_rejects_empty_after_whitespace_normalization() {
    let data = DataFrame::new(
        2,
        vec![
            Series::new("subject".into(), ["s1", "s1"]).into(),
            Series::new("condition".into(), ["before", " \t\n "]).into(),
            Series::new("value".into(), [1.0_f64, 2.0]).into(),
        ],
    )
    .unwrap();
    let mut chart = Chart::new(&data)
        .group("subject")
        .line_mode(LineMode::Trajectory);
    let _ = chart.x("condition");
    let _ = chart.y("value");
    let _ = chart.select(PlotKind::Line);
    let mut backend = RecordingBackend::default();

    let error = chart.render_with_backend(&mut backend).unwrap_err();

    assert!(
        error
            .to_string()
            .contains("condition label is empty after whitespace normalization at source row 1"),
        "{error}"
    );
    assert_eq!(backend.begins, 0);
}

#[test]
fn numeric_looking_string_conditions_remain_categorical_and_first_observed() {
    let data = DataFrame::new(
        2,
        vec![
            Series::new("subject".into(), ["s1", "s1"]).into(),
            Series::new("condition".into(), ["10", "2"]).into(),
            Series::new("value".into(), [100.0_f64, 200.0]).into(),
        ],
    )
    .unwrap();
    let mut chart = Chart::new(&data)
        .group("subject")
        .line_mode(LineMode::Trajectory);
    let _ = chart.x("condition");
    let _ = chart.y("value");
    let _ = chart.select(PlotKind::Line);
    let mut backend = RecordingBackend::default();

    chart.render_with_backend(&mut backend).unwrap();

    let ticks = backend
        .texts
        .iter()
        .filter(|text| ["10", "2"].contains(&text.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(ticks, ["10", "2"]);
}

#[test]
fn faceted_categorical_trajectories_keep_global_condition_order_and_source_rows() {
    let data = DataFrame::new(
        8,
        vec![
            Series::new(
                "facet".into(),
                [
                    "left", "left", "left", "left", "right", "right", "right", "right",
                ],
            )
            .into(),
            Series::new(
                "subject".into(),
                ["s1", "s1", "s2", "s2", "s1", "s1", "s2", "s2"],
            )
            .into(),
            Series::new(
                "condition".into(),
                [
                    " after ",
                    "before",
                    "after",
                    " before ",
                    " during ",
                    "before",
                    "during",
                    "\tbefore ",
                ],
            )
            .into(),
            Series::new("value".into(), [4.0_f64, 1.0, 5.0, 2.0, 6.0, 3.0, 7.0, 4.0]).into(),
        ],
    )
    .unwrap();
    let mut chart = Chart::new(&data)
        .group("subject")
        .line_mode(LineMode::Trajectory)
        .facet_col("facet");
    let _ = chart.x("condition");
    let _ = chart.y("value");
    let _ = chart.select(PlotKind::Line);
    let mut backend = RecordingBackend::default();

    chart.render_with_backend(&mut backend).unwrap();

    let connectors = backend
        .lines
        .iter()
        .filter(|line| line.width.to_bits() == 1.4_f32.to_bits())
        .count();
    assert_eq!(connectors, 4);
    for condition in ["After", "Before", "During"] {
        assert_eq!(
            backend
                .texts
                .iter()
                .filter(|text| *text == condition)
                .count(),
            2
        );
    }
}

#[test]
fn faceted_hues_use_global_first_observed_colors_and_one_complete_legend() {
    let data = DataFrame::new(
        8,
        vec![
            Series::new(
                "facet".into(),
                [
                    "left", "left", "left", "left", "right", "right", "right", "right",
                ],
            )
            .into(),
            Series::new(
                "subject".into(),
                ["s1", "s1", "s2", "s2", "s1", "s1", "s2", "s2"],
            )
            .into(),
            Series::new(
                "condition".into(),
                [
                    "before", "after", "before", "after", "before", "after", "before", "after",
                ],
            )
            .into(),
            Series::new("value".into(), [1.0_f64, 2.0, 2.0, 3.0, 3.0, 4.0, 4.0, 5.0]).into(),
            Series::new(
                "status".into(),
                [
                    "late", "late", "late", "late", "early", "early", "early", "early",
                ],
            )
            .into(),
        ],
    )
    .unwrap();
    let mut chart = Chart::new(&data)
        .group("subject")
        .hue("status")
        .line_mode(LineMode::Trajectory)
        .facet_col("facet");
    let _ = chart.x("condition");
    let _ = chart.y("value");
    let _ = chart.select(PlotKind::Line);
    let mut backend = RecordingBackend::default();

    chart.render_with_backend(&mut backend).unwrap();

    let palette = Theme::Publication.palette();
    assert_eq!(
        backend
            .scatters
            .iter()
            .map(|(_, color, _)| *color)
            .collect::<Vec<_>>(),
        [
            palette.colors[0],
            palette.colors[0],
            palette.colors[0],
            palette.colors[0],
            palette.colors[1],
            palette.colors[1],
            palette.colors[1],
            palette.colors[1],
        ]
    );
    assert_eq!(
        backend.texts.iter().filter(|text| *text == "Late").count(),
        1
    );
    assert_eq!(
        backend.texts.iter().filter(|text| *text == "Early").count(),
        1
    );
}

#[test]
fn facet_local_singleton_group_rejects_before_begin() {
    let data = DataFrame::new(
        5,
        vec![
            Series::new("facet".into(), ["left", "left", "right", "right", "right"]).into(),
            Series::new("subject".into(), ["s1", "s1", "s1", "s2", "s2"]).into(),
            Series::new(
                "condition".into(),
                ["before", "after", "before", "before", "after"],
            )
            .into(),
            Series::new("value".into(), [1.0_f64, 2.0, 3.0, 4.0, 5.0]).into(),
        ],
    )
    .unwrap();
    let mut chart = Chart::new(&data)
        .group("subject")
        .line_mode(LineMode::Trajectory)
        .facet_col("facet");
    let _ = chart.x("condition");
    let _ = chart.y("value");
    let _ = chart.select(PlotKind::Line);
    let mut backend = RecordingBackend::default();

    let error = chart.render_with_backend(&mut backend).unwrap_err();

    assert!(
        error
            .to_string()
            .contains("group \"s1\" requires at least two retained conditions"),
        "{error}"
    );
    assert_eq!(backend.begins, 0);
}

#[test]
fn faceted_duplicate_reports_original_source_row_before_begin() {
    let data = DataFrame::new(
        4,
        vec![
            Series::new("facet".into(), ["left", "left", "right", "right"]).into(),
            Series::new("subject".into(), ["s1", "s1", "s2", "s2"]).into(),
            Series::new("condition".into(), ["a", "b", "a", "a"]).into(),
            Series::new("value".into(), [1.0_f64, 2.0, 3.0, 4.0]).into(),
        ],
    )
    .unwrap();
    let mut chart = Chart::new(&data)
        .group("subject")
        .line_mode(LineMode::Trajectory)
        .facet_col("facet");
    let _ = chart.x("condition");
    let _ = chart.y("value");
    let _ = chart.select(PlotKind::Line);
    let mut backend = RecordingBackend::default();

    let error = chart.render_with_backend(&mut backend).unwrap_err();

    assert!(error.to_string().contains("source row 3"), "{error}");
    assert_eq!(backend.begins, 0);
}

#[test]
fn composed_figure_renders_trajectory_without_cross_panel_lifecycle() {
    let trajectory_data = numeric_trajectory_data();
    let series_data = DataFrame::new(
        2,
        vec![
            Series::new("x".into(), [1.0_f64, 2.0]).into(),
            Series::new("y".into(), [1.0_f64, 3.0]).into(),
        ],
    )
    .unwrap();
    let trajectory = trajectory_chart(&trajectory_data);
    let mut series = Chart::new(&series_data);
    let _ = series.x("x");
    let _ = series.y("y");
    let _ = series.select(PlotKind::Line);
    let figure = Figure::grid(1, 2)
        .unwrap()
        .panel(0, 0, trajectory)
        .unwrap()
        .panel(0, 1, series)
        .unwrap();
    let mut backend = RecordingBackend::default();

    figure.render_with_backend(&mut backend).unwrap();

    assert_eq!((backend.begins, backend.ends), (1, 1));
    assert_eq!(
        backend
            .lines
            .iter()
            .filter(|line| line.width.to_bits() == 1.4_f32.to_bits())
            .count(),
        2
    );
}

#[test]
fn categorical_trajectory_rejects_numeric_x_axis_overrides_before_begin() {
    let data = categorical_trajectory_data();
    let configurations = [
        (
            Chart::new(&data)
                .group("subject")
                .line_mode(LineMode::Trajectory)
                .x_limits(0.0, 2.0),
            "categorical condition x does not support numeric x limits",
        ),
        (
            Chart::new(&data)
                .group("subject")
                .line_mode(LineMode::Trajectory)
                .x_tick_count(5),
            "categorical condition x does not support numeric x tick-count overrides",
        ),
    ];
    for (mut chart, expected) in configurations {
        let _ = chart.x("condition");
        let _ = chart.y("value");
        let _ = chart.select(PlotKind::Line);
        let mut backend = RecordingBackend::default();

        let error = chart.render_with_backend(&mut backend).unwrap_err();

        assert!(error.to_string().contains(expected), "{error}");
        assert_eq!(backend.begins, 0);
    }
}

#[test]
fn composed_categorical_trajectory_rejects_x_axis_override_before_begin() {
    let data = categorical_trajectory_data();
    let configurations = [
        (
            Chart::new(&data)
                .group("subject")
                .line_mode(LineMode::Trajectory)
                .x_limits(0.0, 2.0),
            "categorical condition x does not support numeric x limits",
        ),
        (
            Chart::new(&data)
                .group("subject")
                .line_mode(LineMode::Trajectory)
                .x_tick_count(3),
            "categorical condition x does not support numeric x tick-count overrides",
        ),
    ];
    for (mut chart, expected) in configurations {
        let _ = chart.x("condition");
        let _ = chart.y("value");
        let _ = chart.select(PlotKind::Line);
        let figure = Figure::grid(1, 1).unwrap().panel(0, 0, chart).unwrap();
        let mut backend = RecordingBackend::default();

        let error = figure.render_with_backend(&mut backend).unwrap_err();

        assert!(error.to_string().contains(expected), "{error}");
        assert_eq!(backend.begins, 0);
    }
}

#[test]
fn numeric_trajectory_preserves_x_limits_and_tick_count_behavior() {
    let data = numeric_trajectory_data();
    let chart = trajectory_chart(&data).x_limits(0.5, 2.5).x_tick_count(3);
    let mut backend = RecordingBackend::default();

    chart.render_with_backend(&mut backend).unwrap();

    assert_eq!((backend.begins, backend.ends), (1, 1));
    assert_eq!(
        backend
            .lines
            .iter()
            .filter(|line| line.width.to_bits() == 1.4_f32.to_bits())
            .count(),
        2
    );
}

#[test]
fn trajectory_rejects_signed_zero_duplicate_before_begin() {
    let data = DataFrame::new(
        2,
        vec![
            Series::new("subject".into(), ["s1", "s1"]).into(),
            Series::new("condition".into(), [0.0_f64, -0.0]).into(),
            Series::new("value".into(), [1.0_f64, 2.0]).into(),
        ],
    )
    .unwrap();
    let mut chart = Chart::new(&data)
        .group("subject")
        .line_mode(LineMode::Trajectory);
    let _ = chart.x("condition");
    let _ = chart.y("value");
    let _ = chart.select(PlotKind::Line);
    let mut backend = RecordingBackend::default();

    let error = chart.render_with_backend(&mut backend).unwrap_err();

    assert!(
        error.to_string().contains("duplicate normalized"),
        "{error}"
    );
    assert!(error.to_string().contains("source row 1"), "{error}");
    assert_eq!(backend.begins, 0);
}

#[test]
fn trajectory_rejects_null_nan_and_infinite_group_labels_with_source_rows() {
    for (subject, expected) in [
        (
            Series::new("subject".into(), [Some(1.0_f64), None]),
            "null at source row 1",
        ),
        (
            Series::new("subject".into(), [1.0_f64, f64::NAN]),
            "NaN at source row 1",
        ),
        (
            Series::new("subject".into(), [1.0_f64, f64::INFINITY]),
            "infinite value at source row 1",
        ),
    ] {
        let data = DataFrame::new(
            2,
            vec![
                subject.into(),
                Series::new("condition".into(), [1.0_f64, 2.0]).into(),
                Series::new("value".into(), [1.0_f64, 2.0]).into(),
            ],
        )
        .unwrap();
        let mut chart = Chart::new(&data)
            .group("subject")
            .line_mode(LineMode::Trajectory);
        let _ = chart.x("condition");
        let _ = chart.y("value");
        let _ = chart.select(PlotKind::Line);
        let mut backend = RecordingBackend::default();

        let error = chart.render_with_backend(&mut backend).unwrap_err();

        assert!(error.to_string().contains(expected), "{error}");
        assert_eq!(backend.begins, 0);
    }
}

#[test]
fn trajectory_rejects_null_nan_and_infinite_conditions_with_source_rows() {
    for (condition, expected) in [
        (
            Series::new("condition".into(), [Some(1.0_f64), None]),
            "null at source row 1",
        ),
        (
            Series::new("condition".into(), [1.0_f64, f64::INFINITY]),
            "non-finite value inf at source row 1",
        ),
        (
            Series::new("condition".into(), [1.0_f64, f64::NAN]),
            "non-finite value NaN at source row 1",
        ),
    ] {
        let data = DataFrame::new(
            2,
            vec![
                Series::new("subject".into(), ["s1", "s1"]).into(),
                condition.into(),
                Series::new("value".into(), [1.0_f64, 2.0]).into(),
            ],
        )
        .unwrap();
        let mut chart = Chart::new(&data)
            .group("subject")
            .line_mode(LineMode::Trajectory);
        let _ = chart.x("condition");
        let _ = chart.y("value");
        let _ = chart.select(PlotKind::Line);
        let mut backend = RecordingBackend::default();

        let error = chart.render_with_backend(&mut backend).unwrap_err();

        assert!(error.to_string().contains(expected), "{error}");
        assert_eq!(backend.begins, 0);
    }
}

#[test]
fn trajectory_requires_two_conditions_per_group() {
    let data = DataFrame::new(
        3,
        vec![
            Series::new("subject".into(), ["s1", "s1", "s2"]).into(),
            Series::new("condition".into(), [1.0_f64, 2.0, 1.0]).into(),
            Series::new("value".into(), [1.0_f64, 2.0, 3.0]).into(),
        ],
    )
    .unwrap();
    let mut chart = Chart::new(&data)
        .group("subject")
        .line_mode(LineMode::Trajectory);
    let _ = chart.x("condition");
    let _ = chart.y("value");
    let _ = chart.select(PlotKind::Line);
    let mut backend = RecordingBackend::default();

    let error = chart.render_with_backend(&mut backend).unwrap_err();

    assert!(
        error
            .to_string()
            .contains("group \"s2\" requires at least two retained conditions"),
        "{error}"
    );
    assert_eq!(backend.begins, 0);
}

#[test]
fn trajectory_requires_explicit_group_and_visible_markers_without_ribbons() {
    let data = numeric_trajectory_data();
    let configurations = [
        Chart::new(&data)
            .hue("status")
            .line_mode(LineMode::Trajectory),
        Chart::new(&data)
            .group("subject")
            .line_mode(LineMode::Trajectory)
            .markers(false),
        Chart::new(&data)
            .group("subject")
            .line_mode(LineMode::Trajectory)
            .interval_bounds("value", "value"),
    ];
    for mut chart in configurations {
        let _ = chart.x("condition");
        let _ = chart.y("value");
        let _ = chart.select(PlotKind::Line);
        let mut backend = RecordingBackend::default();

        chart.render_with_backend(&mut backend).unwrap_err();

        assert_eq!(backend.begins, 0);
    }
}

#[test]
fn every_non_line_kind_rejects_trajectory_before_begin() {
    let data = numeric_trajectory_data();
    for kind in PlotKind::ALL
        .iter()
        .copied()
        .filter(|kind| *kind != PlotKind::Line)
    {
        let mut chart = Chart::new(&data)
            .group("subject")
            .line_mode(LineMode::Trajectory);
        let _ = chart.x("condition");
        let _ = chart.y("value");
        let _ = chart.select(kind);
        let mut backend = RecordingBackend::default();

        let error = chart.render_with_backend(&mut backend).unwrap_err();

        assert!(
            error.to_string().contains("only supported by Line"),
            "{kind:?}: {error}"
        );
        assert_eq!(backend.begins, 0, "{kind:?}");
    }
}

#[test]
fn series_is_the_default_and_remains_identical_when_explicit() {
    let data = numeric_trajectory_data();
    let mut default = Chart::new(&data).group("subject");
    let _ = default.x("condition");
    let _ = default.y("value");
    let _ = default.select(PlotKind::Line);
    let mut explicit = Chart::new(&data)
        .group("subject")
        .line_mode(LineMode::Series);
    let _ = explicit.x("condition");
    let _ = explicit.y("value");
    let _ = explicit.select(PlotKind::Line);
    let mut default_backend = RecordingBackend::default();
    let mut explicit_backend = RecordingBackend::default();

    default.render_with_backend(&mut default_backend).unwrap();
    explicit.render_with_backend(&mut explicit_backend).unwrap();

    assert_eq!(default_backend.lines, explicit_backend.lines);
    assert_eq!(default_backend.scatters, explicit_backend.scatters);
    assert_eq!(default_backend.texts, explicit_backend.texts);
}

#[test]
fn trajectory_svg_is_deterministic_and_exposes_neutral_connector_and_hue_points() {
    let data = DataFrame::new(
        2,
        vec![
            Series::new("subject".into(), ["subject-one", "subject-one"]).into(),
            Series::new("condition".into(), ["before", "after"]).into(),
            Series::new("value".into(), [1.0_f64, 3.0]).into(),
            Series::new("status".into(), ["control", "case"]).into(),
        ],
    )
    .unwrap();
    let chart = trajectory_chart(&data);
    let directory = std::env::temp_dir().join(format!(
        "lichtung-plot-line-path-svg-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&directory).unwrap();
    let first = directory.join("first.svg");
    let second = directory.join("second.svg");

    chart.render_to_svg(&first).unwrap();
    chart.render_to_svg(&second).unwrap();
    let first_svg = std::fs::read_to_string(&first).unwrap();
    let second_svg = std::fs::read_to_string(&second).unwrap();

    assert_eq!(first_svg, second_svg);
    assert!(first_svg.contains("stroke=\"#000000\" stroke-width=\"1.4\""));
    assert!(first_svg.contains("fill=\"#0072B2\""));
    assert!(first_svg.contains("fill=\"#D55E00\""));
    assert!(!first_svg.contains("subject-one"));
    std::fs::remove_dir_all(directory).unwrap();
}
