use lichtung_plot::layout::PanelBox;
use lichtung_plot::render::backend::{
    Backend, BracketStyle, FillStyle, GradientStop, LineStyle, Point2D, PointStyle,
    SignificanceBracket, TextStyle,
};
use lichtung_plot::Error;
use lichtung_plot::{
    BarAggregation, Chart, DumbbellEndpoints, EstimateSource, InferentialScale, IntervalKind,
    IntervalSemantics, PlotKind,
};
use lichtung_plot::{ComparisonAnnotation, ComparisonEndpoint};
use polars::prelude::{DataFrame, NamedFrom, Series};

struct RecordedFill {
    panel: PanelBox,
    color: (u8, u8, u8),
    opacity: f64,
    center: f64,
}

#[derive(Default)]
struct RecordingBackend {
    begins: usize,
    ends: usize,
    data_lines: Vec<(u8, u8, u8)>,
    density_lines: Vec<(u8, u8, u8)>,
    clipped_scatters: Vec<((u8, u8, u8), u32)>,
    clipped_fills: Vec<RecordedFill>,
    clip_stack: Vec<PanelBox>,
    texts: Vec<String>,
    brackets: usize,
    bracket_orientations: Vec<lichtung_plot::render::backend::BracketOrientation>,
}

impl Backend for RecordingBackend {
    fn begin(&mut self, _width: u32, _height: u32) -> Result<(), Error> {
        self.begins += 1;
        Ok(())
    }

    fn begin_clip(&mut self, region: PanelBox) -> Result<(), Error> {
        self.clip_stack.push(region);
        Ok(())
    }

    fn end_clip(&mut self) -> Result<(), Error> {
        self.clip_stack.pop();
        Ok(())
    }

    fn draw_scatter(&mut self, _points: &[Point2D], style: &PointStyle) -> Result<(), Error> {
        if !self.clip_stack.is_empty() {
            self.clipped_scatters.push((style.color, style.radius));
        }
        Ok(())
    }

    fn draw_line(&mut self, points: &[Point2D], style: &LineStyle) -> Result<(), Error> {
        if points.len() >= 2 && style.width.to_bits() == 1.8_f32.to_bits() {
            self.data_lines.push(style.color);
        }
        if style.width.to_bits() == 1.6_f32.to_bits() {
            self.density_lines.push(style.color);
        }
        Ok(())
    }

    fn draw_polygon(&mut self, points: &[Point2D], style: &FillStyle) -> Result<(), Error> {
        if let Some(panel) = self.clip_stack.last().copied() {
            let point_count = f64::from(u32::try_from(points.len()).unwrap());
            let center = points.iter().map(|point| point.x).sum::<f64>() / point_count;
            self.clipped_fills.push(RecordedFill {
                panel,
                color: style.color,
                opacity: style.opacity,
                center,
            });
        }
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
        bracket: &SignificanceBracket,
        _style: &BracketStyle,
    ) -> Result<(), Error> {
        self.brackets += 1;
        self.bracket_orientations.push(bracket.orientation);
        Ok(())
    }

    fn end(&mut self) -> Result<(), Error> {
        self.ends += 1;
        Ok(())
    }
}

fn shared_hue_data() -> DataFrame {
    DataFrame::new(
        9,
        vec![
            Series::new(
                "group".into(),
                ["g1", "g1", "g1", "g2", "g2", "g2", "g3", "g3", "g3"],
            )
            .into(),
            Series::new(
                "hue".into(),
                [
                    "shared", "shared", "shared", "other", "other", "other", "shared", "shared",
                    "shared",
                ],
            )
            .into(),
            Series::new(
                "x".into(),
                [0.0_f64, 1.0, 2.0, 0.0, 1.0, 2.0, 0.0, 1.0, 2.0],
            )
            .into(),
            Series::new(
                "y".into(),
                [1.0_f64, 2.0, 4.0, 2.0, 4.0, 7.0, 3.0, 5.0, 9.0],
            )
            .into(),
        ],
    )
    .unwrap()
}

fn shared_hue_interaction_data() -> DataFrame {
    DataFrame::new(
        6,
        vec![
            Series::new(
                "condition".into(),
                ["before", "after", "before", "after", "before", "after"],
            )
            .into(),
            Series::new("group".into(), ["g1", "g1", "g2", "g2", "g3", "g3"]).into(),
            Series::new(
                "hue".into(),
                ["shared", "shared", "other", "other", "shared", "shared"],
            )
            .into(),
            Series::new("estimate".into(), [1.0_f64, 2.0, 2.0, 4.0, 3.0, 5.0]).into(),
            Series::new("lower".into(), [0.5_f64, 1.5, 1.5, 3.5, 2.5, 4.5]).into(),
            Series::new("upper".into(), [1.5_f64, 2.5, 2.5, 4.5, 3.5, 5.5]).into(),
        ],
    )
    .unwrap()
}

fn collective_data() -> DataFrame {
    DataFrame::new(
        12,
        vec![
            Series::new(
                "group".into(),
                [
                    "g1", "g1", "g1", "g1", "g1", "g1", "g2", "g2", "g2", "g2", "g2", "g2",
                ],
            )
            .into(),
            Series::new(
                "hue".into(),
                [
                    "h1", "h1", "h1", "h1", "h1", "h1", "h2", "h2", "h2", "h2", "h2", "h2",
                ],
            )
            .into(),
            Series::new(
                "category".into(),
                ["a", "a", "a", "b", "b", "b", "a", "a", "a", "b", "b", "b"],
            )
            .into(),
            Series::new(
                "x".into(),
                [
                    0.0_f64, 1.0, 2.0, 3.0, 4.0, 5.0, 0.0, 1.0, 2.0, 3.0, 4.0, 5.0,
                ],
            )
            .into(),
            Series::new(
                "y".into(),
                [
                    1.0_f64, 2.0, 4.0, 3.0, 5.0, 7.0, 2.0, 3.0, 5.0, 4.0, 6.0, 9.0,
                ],
            )
            .into(),
        ],
    )
    .unwrap()
}

fn interaction_data() -> DataFrame {
    DataFrame::new(
        4,
        vec![
            Series::new("condition".into(), ["before", "after", "before", "after"]).into(),
            Series::new("group".into(), ["g1", "g1", "g2", "g2"]).into(),
            Series::new("hue".into(), ["h1", "h1", "h2", "h2"]).into(),
            Series::new("estimate".into(), [1.0_f64, 2.0, 2.0, 4.0]).into(),
            Series::new("lower".into(), [0.5_f64, 1.5, 1.5, 3.5]).into(),
            Series::new("upper".into(), [1.5_f64, 2.5, 2.5, 4.5]).into(),
        ],
    )
    .unwrap()
}

fn configured_group_chart(data: &DataFrame, kind: PlotKind) -> Chart<'_> {
    let mut chart = if kind == PlotKind::Interaction {
        Chart::new(data)
            .group("group")
            .interval_bounds("lower", "upper")
            .interval_semantics(
                IntervalSemantics::new(
                    IntervalKind::Confidence,
                    0.95,
                    InferentialScale::Response,
                    "group aesthetics fixture",
                )
                .unwrap(),
            )
            .interaction_source(EstimateSource::EstimatedMarginalMean)
    } else if kind == PlotKind::Dumbbell {
        Chart::new(data)
            .group("group")
            .dumbbell_right("right")
            .dumbbell_endpoints(
                DumbbellEndpoints::new("Left", (20, 80, 140), "Right", (180, 60, 40)).unwrap(),
            )
    } else {
        Chart::new(data).group("group")
    };
    match kind {
        PlotKind::Density => {
            let _ = chart.x("y");
        }
        PlotKind::Matrix => {
            let _ = chart.matrix_variables(vec!["x", "y"]);
        }
        PlotKind::Interaction => {
            let _ = chart.x("condition");
            let _ = chart.y("estimate");
        }
        PlotKind::Dumbbell => {
            let _ = chart.x("category");
            let _ = chart.y("left");
        }
        PlotKind::Line | PlotKind::Marginal => {
            let _ = chart.x("x");
            let _ = chart.y("y");
        }
        _ => {
            let _ = chart.x("category");
            let _ = chart.y("y");
        }
    }
    let _ = chart.select(kind);
    chart
}

#[test]
fn exact_supported_families_render_group_only_without_group_legends() {
    let data = collective_data();
    let interaction = interaction_data();
    let dumbbell = DataFrame::new(
        4,
        vec![
            Series::new("category".into(), ["a", "a", "b", "b"]).into(),
            Series::new("group".into(), ["g1", "g2", "g1", "g2"]).into(),
            Series::new("left".into(), [1.0_f64, 1.5, 2.0, 2.5]).into(),
            Series::new("right".into(), [2.0_f64, 2.5, 3.0, 3.5]).into(),
        ],
    )
    .unwrap();
    for kind in [
        PlotKind::Line,
        PlotKind::Interaction,
        PlotKind::Density,
        PlotKind::Marginal,
        PlotKind::Matrix,
        PlotKind::Bar,
        PlotKind::HorizontalBar,
        PlotKind::StackedBar,
        PlotKind::StackedHorizontalBar,
        PlotKind::Boxplot,
        PlotKind::Violin,
        PlotKind::Dumbbell,
    ] {
        let source = if kind == PlotKind::Interaction {
            &interaction
        } else if kind == PlotKind::Dumbbell {
            &dumbbell
        } else {
            &data
        };
        let chart = configured_group_chart(source, kind);
        let mut backend = RecordingBackend::default();
        chart
            .render_with_backend(&mut backend)
            .unwrap_or_else(|error| {
                panic!("{kind:?} should support explicit graphical group: {error}")
            });
        assert_eq!((backend.begins, backend.ends), (1, 1), "{kind:?}");
        assert!(!backend
            .texts
            .iter()
            .any(|text| text == "g1" || text == "g2"));
    }
}

#[test]
fn every_other_plot_kind_rejects_group_before_backend_begin() {
    let data = collective_data();
    let supported = [
        PlotKind::Line,
        PlotKind::Interaction,
        PlotKind::Density,
        PlotKind::Marginal,
        PlotKind::Matrix,
        PlotKind::Bar,
        PlotKind::HorizontalBar,
        PlotKind::StackedBar,
        PlotKind::StackedHorizontalBar,
        PlotKind::Boxplot,
        PlotKind::Violin,
        PlotKind::Dumbbell,
    ];
    for kind in PlotKind::ALL
        .iter()
        .copied()
        .filter(|kind| !supported.contains(kind))
    {
        let mut chart = Chart::new(&data).group("group");
        let _ = chart.select(kind);
        let mut backend = RecordingBackend::default();
        let error = chart.render_with_backend(&mut backend).unwrap_err();
        assert!(
            error.to_string().contains("graphical group column"),
            "{kind:?}: {error}"
        );
        assert_eq!(backend.begins, 0, "{kind:?}");
    }
}

#[test]
fn explicit_group_controls_paths_while_hue_controls_color_and_legend() {
    let data = collective_data();
    let mut group_only = configured_group_chart(&data, PlotKind::Line);
    let mut group_only_backend = RecordingBackend::default();
    group_only
        .render_with_backend(&mut group_only_backend)
        .unwrap();
    assert_eq!(group_only_backend.data_lines.len(), 2);
    assert_eq!(
        group_only_backend.data_lines[0],
        group_only_backend.data_lines[1]
    );
    assert!(!group_only_backend
        .texts
        .iter()
        .any(|text| text == "g1" || text == "g2" || text == "G1" || text == "G2"));

    group_only = group_only.hue("hue");
    let mut both_backend = RecordingBackend::default();
    group_only.render_with_backend(&mut both_backend).unwrap();
    assert_eq!(both_backend.data_lines.len(), 2);
    assert_ne!(both_backend.data_lines[0], both_backend.data_lines[1]);
    assert!(both_backend.texts.iter().any(|text| text == "H 1"));
    assert!(both_backend.texts.iter().any(|text| text == "H 2"));
    assert!(!both_backend
        .texts
        .iter()
        .any(|text| text == "g1" || text == "g2" || text == "G1" || text == "G2"));
}

#[test]
fn explicit_group_rejects_multiple_hues_per_collective_mark() {
    let data = DataFrame::new(
        4,
        vec![
            Series::new("group".into(), ["g1", "g1", "g2", "g2"]).into(),
            Series::new("hue".into(), ["h1", "h2", "h2", "h2"]).into(),
            Series::new("x".into(), [0.0_f64, 1.0, 0.0, 1.0]).into(),
            Series::new("y".into(), [1.0_f64, 2.0, 3.0, 4.0]).into(),
        ],
    )
    .unwrap();
    let mut chart = Chart::new(&data).group("group").hue("hue");
    let _ = chart.x("x");
    let _ = chart.y("y");
    let _ = chart.select(PlotKind::Line);
    let mut backend = RecordingBackend::default();
    let error = chart.render_with_backend(&mut backend).unwrap_err();
    assert!(error
        .to_string()
        .contains("each collective mark must have exactly one hue"));
    assert!(error.to_string().contains("source row 1"));
    assert_eq!(backend.begins, 0);
}

#[test]
fn categorical_marks_allow_one_group_to_change_hue_between_x_cells() {
    let data = DataFrame::new(
        8,
        vec![
            Series::new("category".into(), ["a", "a", "a", "a", "b", "b", "b", "b"]).into(),
            Series::new(
                "group".into(),
                ["g1", "g1", "g2", "g2", "g1", "g1", "g2", "g2"],
            )
            .into(),
            Series::new(
                "hue".into(),
                ["h1", "h1", "h2", "h2", "h2", "h2", "h1", "h1"],
            )
            .into(),
            Series::new("y".into(), [1.0_f64, 2.0, 3.0, 4.0, 2.0, 3.0, 4.0, 5.0]).into(),
        ],
    )
    .unwrap();
    let mut chart = Chart::new(&data).group("group").hue("hue");
    let _ = chart.x("category");
    let _ = chart.y("y");
    let _ = chart.select(PlotKind::Bar);
    let mut backend = RecordingBackend::default();
    chart.render_with_backend(&mut backend).unwrap();
    assert_eq!((backend.begins, backend.ends), (1, 1));
}

#[test]
fn significance_endpoints_select_explicit_group_and_validate_hue() {
    let data = collective_data();
    let comparison = ComparisonAnnotation {
        endpoint_a: ComparisonEndpoint::new("a").with_group("g1").with_hue("h1"),
        endpoint_b: ComparisonEndpoint::new("a").with_group("g2").with_hue("h2"),
        facet: Vec::new(),
        raw_p_value: 0.01,
        adjusted_p_value: None,
        correction: None,
    };
    let mut chart = Chart::new(&data).group("group").hue("hue");
    let _ = chart.x("category");
    let _ = chart.y("y");
    let _ = chart.add_comparisons([comparison]);
    let _ = chart.select(PlotKind::Boxplot);
    let mut backend = RecordingBackend::default();
    chart.render_with_backend(&mut backend).unwrap();
    assert_eq!(backend.brackets, 1);
}

#[test]
fn faceted_group_hue_error_reports_original_source_row_before_begin() {
    let data = DataFrame::new(
        4,
        vec![
            Series::new("facet".into(), ["first", "first", "second", "second"]).into(),
            Series::new("group".into(), ["g1", "g1", "g1", "g1"]).into(),
            Series::new("hue".into(), ["h1", "h1", "h1", "h2"]).into(),
            Series::new("x".into(), [0.0_f64, 1.0, 0.0, 1.0]).into(),
            Series::new("y".into(), [1.0_f64, 2.0, 3.0, 4.0]).into(),
        ],
    )
    .unwrap();
    let mut chart = Chart::new(&data)
        .group("group")
        .hue("hue")
        .facet_col("facet");
    let _ = chart.x("x");
    let _ = chart.y("y");
    let _ = chart.select(PlotKind::Line);
    let mut backend = RecordingBackend::default();
    let error = chart.render_with_backend(&mut backend).unwrap_err();
    assert!(error.to_string().contains("source row 3"), "{error}");
    assert_eq!(backend.begins, 0);
}

#[test]
fn shared_hue_uses_one_chart_wide_color_across_collective_families() {
    let data = shared_hue_data();
    let expected = lichtung_plot::Theme::Publication.palette().colors;

    let mut line = Chart::new(&data).group("group").hue("hue");
    let _ = line.x("x");
    let _ = line.y("y");
    let _ = line.select(PlotKind::Line);
    let mut line_backend = RecordingBackend::default();
    line.render_with_backend(&mut line_backend).unwrap();
    assert_eq!(
        line_backend.data_lines,
        [expected[0], expected[1], expected[0]]
    );

    let mut density = Chart::new(&data).group("group").hue("hue");
    let _ = density.x("y");
    let _ = density.select(PlotKind::Density);
    let mut density_backend = RecordingBackend::default();
    density.render_with_backend(&mut density_backend).unwrap();
    let density_colors = density_backend
        .clipped_fills
        .iter()
        .filter(|fill| fill.opacity.to_bits() == 0.22_f64.to_bits())
        .map(|fill| fill.color)
        .collect::<Vec<_>>();
    assert_eq!(density_colors, [expected[0], expected[1], expected[0]]);

    let mut marginal = Chart::new(&data).group("group").hue("hue");
    let _ = marginal.x("x");
    let _ = marginal.y("y");
    let _ = marginal.select(PlotKind::Marginal);
    let mut marginal_backend = RecordingBackend::default();
    marginal.render_with_backend(&mut marginal_backend).unwrap();
    let marginal_colors = marginal_backend
        .clipped_scatters
        .iter()
        .filter(|(_, radius)| *radius == 3)
        .map(|(color, _)| *color)
        .collect::<Vec<_>>();
    assert_eq!(marginal_colors, [expected[0], expected[1], expected[0]]);

    let mut matrix = Chart::new(&data).group("group").hue("hue");
    let _ = matrix.matrix_variables(vec!["x", "y"]);
    let _ = matrix.select(PlotKind::Matrix);
    let mut matrix_backend = RecordingBackend::default();
    matrix.render_with_backend(&mut matrix_backend).unwrap();
    let matrix_colors = matrix_backend
        .clipped_scatters
        .iter()
        .filter(|(_, radius)| *radius == 2)
        .take(3)
        .map(|(color, _)| *color)
        .collect::<Vec<_>>();
    assert_eq!(matrix_colors, [expected[0], expected[1], expected[0]]);

    let interaction_data = shared_hue_interaction_data();
    let interaction = configured_group_chart(&interaction_data, PlotKind::Interaction).hue("hue");
    let mut interaction_backend = RecordingBackend::default();
    interaction
        .render_with_backend(&mut interaction_backend)
        .unwrap();
    assert_eq!(
        interaction_backend.data_lines,
        [expected[0], expected[1], expected[0]]
    );
}

#[test]
fn group_only_density_uses_default_color_for_every_curve_and_fill() {
    let data = collective_data();
    let mut chart = Chart::new(&data).group("group");
    let expected = chart.palette().colors[0];
    let _ = chart.x("y");
    let _ = chart.select(PlotKind::Density);
    let mut backend = RecordingBackend::default();
    chart.render_with_backend(&mut backend).unwrap();

    assert!(backend.density_lines.iter().all(|color| *color == expected));
    assert!(backend
        .clipped_fills
        .iter()
        .filter(|fill| fill.opacity.to_bits() == 0.22_f64.to_bits())
        .all(|fill| fill.color == expected));
}

#[test]
fn faceted_explicit_groups_keep_global_dodge_slots_when_a_panel_omits_a_group() {
    let data = DataFrame::new(
        3,
        vec![
            Series::new("facet".into(), ["both", "both", "only-g2"]).into(),
            Series::new("category".into(), ["a", "a", "a"]).into(),
            Series::new("group".into(), ["g1", "g2", "g2"]).into(),
            Series::new("value".into(), [1.0_f64, 2.0, 3.0]).into(),
        ],
    )
    .unwrap();
    let mut chart = Chart::new(&data)
        .group("group")
        .facet_col("facet")
        .bar_aggregation(BarAggregation::Identity);
    let _ = chart.x("category");
    let _ = chart.y("value");
    let _ = chart.select(PlotKind::Bar);
    let mut backend = RecordingBackend::default();
    chart.render_with_backend(&mut backend).unwrap();

    let bars = backend
        .clipped_fills
        .iter()
        .filter(|fill| fill.opacity.to_bits() == 1.0_f64.to_bits())
        .collect::<Vec<_>>();
    assert_eq!(bars.len(), 3);
    let first_panel = bars[0].panel;
    let first_centers = bars
        .iter()
        .filter(|fill| fill.panel.x0.to_bits() == first_panel.x0.to_bits())
        .map(|fill| fill.center)
        .collect::<Vec<_>>();
    assert_eq!(first_centers.len(), 2);
    assert!(first_centers[0] < f64::midpoint(first_panel.x0, first_panel.x1));
    assert!(first_centers[1] > f64::midpoint(first_panel.x0, first_panel.x1));
    let omitted_panel_bar = bars
        .iter()
        .find(|fill| fill.panel.x0.to_bits() != first_panel.x0.to_bits())
        .unwrap();
    assert!(
        omitted_panel_bar.center
            > f64::midpoint(omitted_panel_bar.panel.x0, omitted_panel_bar.panel.x1)
    );
}

#[test]
fn every_categorical_group_family_rejects_conflicting_hue_within_one_cell() {
    let data = DataFrame::new(
        2,
        vec![
            Series::new("category".into(), ["a", "a"]).into(),
            Series::new("group".into(), ["g1", "g1"]).into(),
            Series::new("hue".into(), ["h1", "h2"]).into(),
            Series::new("value".into(), [1.0_f64, 2.0]).into(),
        ],
    )
    .unwrap();
    for kind in [
        PlotKind::Bar,
        PlotKind::HorizontalBar,
        PlotKind::StackedBar,
        PlotKind::StackedHorizontalBar,
        PlotKind::Boxplot,
        PlotKind::Violin,
    ] {
        let mut chart = Chart::new(&data).group("group").hue("hue");
        let _ = chart.x("category");
        let _ = chart.y("value");
        let _ = chart.select(kind);
        let mut backend = RecordingBackend::default();
        let error = chart.render_with_backend(&mut backend).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("each collective mark must have exactly one hue"),
            "{kind:?}: {error}"
        );
        assert_eq!(backend.begins, 0, "{kind:?}");
    }
}

#[test]
fn faceted_collectives_keep_global_first_observed_hue_color_order() {
    let expected = lichtung_plot::Theme::Publication.palette().colors;
    let line_data = DataFrame::new(
        4,
        vec![
            Series::new("facet".into(), ["b", "b", "a", "a"]).into(),
            Series::new("hue".into(), ["h2", "h2", "h1", "h1"]).into(),
            Series::new("x".into(), [0.0_f64, 1.0, 0.0, 1.0]).into(),
            Series::new("y".into(), [1.0_f64, 2.0, 3.0, 4.0]).into(),
        ],
    )
    .unwrap();
    let mut line = Chart::new(&line_data).hue("hue").facet_col("facet");
    let _ = line.x("x");
    let _ = line.y("y");
    let _ = line.select(PlotKind::Line);
    let mut line_backend = RecordingBackend::default();
    line.render_with_backend(&mut line_backend).unwrap();
    assert_eq!(line_backend.data_lines, [expected[0], expected[1]]);

    let interaction_data = DataFrame::new(
        4,
        vec![
            Series::new("facet".into(), ["b", "b", "a", "a"]).into(),
            Series::new("condition".into(), ["before", "after", "before", "after"]).into(),
            Series::new("hue".into(), ["h2", "h2", "h1", "h1"]).into(),
            Series::new("estimate".into(), [1.0_f64, 2.0, 3.0, 4.0]).into(),
            Series::new("lower".into(), [0.5_f64, 1.5, 2.5, 3.5]).into(),
            Series::new("upper".into(), [1.5_f64, 2.5, 3.5, 4.5]).into(),
        ],
    )
    .unwrap();
    let mut interaction = Chart::new(&interaction_data)
        .group("hue")
        .hue("hue")
        .facet_col("facet")
        .interval_bounds("lower", "upper")
        .interval_semantics(
            IntervalSemantics::new(
                IntervalKind::Confidence,
                0.95,
                InferentialScale::Response,
                "hue order fixture",
            )
            .unwrap(),
        )
        .interaction_source(EstimateSource::EstimatedMarginalMean);
    let _ = interaction.x("condition");
    let _ = interaction.y("estimate");
    let _ = interaction.select(PlotKind::Interaction);
    let mut interaction_backend = RecordingBackend::default();
    interaction
        .render_with_backend(&mut interaction_backend)
        .unwrap();
    assert_eq!(interaction_backend.data_lines, [expected[0], expected[1]]);
}

#[test]
fn horizontal_significance_uses_group_anchor_and_mismatched_hue_rejects_before_begin() {
    let data = collective_data();
    let valid = ComparisonAnnotation {
        endpoint_a: ComparisonEndpoint::new("a").with_group("g1").with_hue("h1"),
        endpoint_b: ComparisonEndpoint::new("a").with_group("g2").with_hue("h2"),
        facet: Vec::new(),
        raw_p_value: 0.01,
        adjusted_p_value: None,
        correction: None,
    };
    let mut chart = Chart::new(&data).group("group").hue("hue");
    let _ = chart.x("category");
    let _ = chart.y("y");
    let _ = chart.add_comparisons([valid]);
    let _ = chart.select(PlotKind::HorizontalBar);
    let mut backend = RecordingBackend::default();
    chart.render_with_backend(&mut backend).unwrap();
    assert_eq!(
        backend.bracket_orientations,
        [lichtung_plot::render::backend::BracketOrientation::Horizontal]
    );

    let invalid = ComparisonAnnotation {
        endpoint_a: ComparisonEndpoint::new("a").with_group("g1").with_hue("h2"),
        endpoint_b: ComparisonEndpoint::new("a").with_group("g2").with_hue("h2"),
        facet: Vec::new(),
        raw_p_value: 0.01,
        adjusted_p_value: None,
        correction: None,
    };
    let mut chart = Chart::new(&data).group("group").hue("hue");
    let _ = chart.x("category");
    let _ = chart.y("y");
    let _ = chart.add_comparisons([invalid]);
    let _ = chart.select(PlotKind::HorizontalBar);
    let mut backend = RecordingBackend::default();
    let error = chart.render_with_backend(&mut backend).unwrap_err();
    assert!(error.to_string().contains("does not have hue \"h2\""));
    assert_eq!(backend.begins, 0);
}
