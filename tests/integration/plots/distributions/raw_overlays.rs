use std::path::PathBuf;

use lichtung_plot::layout::PanelBox;
use lichtung_plot::render::backend::{
    Backend, BracketStyle, FillStyle, GradientStop, LineStyle, Point2D, PointStyle,
    SignificanceBracket, TextStyle,
};
use lichtung_plot::Error;
use lichtung_plot::{Chart, Figure, PlotKind, RawOverlay, Theme};
use polars::prelude::{DataFrame, NamedFrom, Series};

#[derive(Clone)]
struct ScatterCall {
    points: Vec<Point2D>,
    style: PointStyle,
    clip: Option<PanelBox>,
}

#[derive(Default)]
struct RecordingBackend {
    begins: usize,
    ends: usize,
    clips: Vec<PanelBox>,
    scatters: Vec<ScatterCall>,
    events: Vec<(&'static str, usize)>,
    texts: Vec<String>,
    text_clip_depths: Vec<usize>,
}

impl Backend for RecordingBackend {
    fn begin(&mut self, _width: u32, _height: u32) -> Result<(), Error> {
        self.begins += 1;
        Ok(())
    }

    fn begin_clip(&mut self, panel: PanelBox) -> Result<(), Error> {
        self.clips.push(panel);
        Ok(())
    }

    fn end_clip(&mut self) -> Result<(), Error> {
        self.clips.pop();
        Ok(())
    }

    fn draw_scatter(&mut self, points: &[Point2D], style: &PointStyle) -> Result<(), Error> {
        self.events.push(("scatter", self.clips.len()));
        self.scatters.push(ScatterCall {
            points: points.to_vec(),
            style: style.clone(),
            clip: self.clips.last().copied(),
        });
        Ok(())
    }

    fn draw_line(&mut self, _points: &[Point2D], _style: &LineStyle) -> Result<(), Error> {
        self.events.push(("line", self.clips.len()));
        Ok(())
    }

    fn draw_polygon(&mut self, _points: &[Point2D], _style: &FillStyle) -> Result<(), Error> {
        self.events.push(("polygon", self.clips.len()));
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
        self.text_clip_depths.push(self.clips.len());
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

fn data(values: &[f64]) -> DataFrame {
    DataFrame::new(
        values.len(),
        vec![
            Series::new("category".into(), vec!["a"; values.len()]).into(),
            Series::new("value".into(), values).into(),
        ],
    )
    .unwrap()
}

fn chart(data: &DataFrame, kind: PlotKind, overlay: RawOverlay) -> Chart<'_> {
    let mut chart = Chart::new(data).raw_overlay(overlay);
    chart.x("category").y("value").select(kind);
    chart
}

fn temp_path(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "lichtung-raw-overlay-{}-{label}.svg",
        std::process::id()
    ))
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

#[test]
fn explicit_hidden_is_byte_identical_to_the_default_for_boxplot_and_violin() {
    let data = data(&[1.0, 2.0, 3.0, 4.0]);
    for kind in [PlotKind::Boxplot, PlotKind::Violin] {
        let default_path = temp_path(&format!("{kind:?}-default"));
        let hidden_path = temp_path(&format!("{kind:?}-hidden"));
        let mut default = Chart::new(&data);
        default.x("category").y("value").select(kind);
        default.render_to_svg(&default_path).unwrap();
        let mut explicit = Chart::new(&data).raw_overlay(RawOverlay::Hidden);
        explicit.x("category").y("value").select(kind);
        explicit.render_to_svg(&hidden_path).unwrap();
        assert_eq!(
            std::fs::read(&default_path).unwrap(),
            std::fs::read(&hidden_path).unwrap(),
            "{kind:?}"
        );
        let svg = std::fs::read_to_string(&hidden_path).unwrap();
        let expected = match kind {
            PlotKind::Boxplot => 0xf3ec_5b1a_728c_43d4,
            PlotKind::Violin => 0xfaeb_cacd_c6ef_3a6d,
            _ => unreachable!(),
        };
        assert_eq!(
            fnv1a64(svg.as_bytes()),
            expected,
            "{kind:?} pre-overlay hidden SVG golden changed"
        );
        std::fs::remove_file(default_path).unwrap();
        std::fs::remove_file(hidden_path).unwrap();
    }
}

#[test]
fn visible_modes_draw_exactly_one_fixed_style_point_per_observation() {
    let data = data(&[1.0, 2.0, 3.0, 4.0]);
    for kind in [PlotKind::Boxplot, PlotKind::Violin] {
        for overlay in [RawOverlay::Points, RawOverlay::Jittered] {
            let mut backend = RecordingBackend::default();
            chart(&data, kind, overlay)
                .render_with_backend(&mut backend)
                .unwrap();
            assert_eq!(backend.scatters.len(), 1, "{kind:?} {overlay:?}");
            assert_eq!(backend.scatters[0].points.len(), 4, "{kind:?} {overlay:?}");
            assert_eq!(backend.scatters[0].style.radius, 2);
            assert!(backend.scatters[0].style.outline.is_some());
            assert!(backend.scatters[0].clip.is_some());
        }
    }
}

#[test]
fn singleton_two_point_and_repeated_cells_preserve_one_overlay_point_per_row() {
    let data = DataFrame::new(
        6,
        vec![
            Series::new(
                "category".into(),
                [
                    "singleton",
                    "pair",
                    "pair",
                    "repeated",
                    "repeated",
                    "repeated",
                ],
            )
            .into(),
            Series::new("value".into(), [1.0_f64, 2.0, 4.0, 3.0, 3.0, 3.0]).into(),
        ],
    )
    .unwrap();
    for kind in [PlotKind::Boxplot, PlotKind::Violin] {
        let mut backend = RecordingBackend::default();
        chart(&data, kind, RawOverlay::Points)
            .render_with_backend(&mut backend)
            .unwrap();
        assert_eq!(
            backend
                .scatters
                .iter()
                .filter(|call| call.clip.is_some())
                .map(|call| call.points.len())
                .collect::<Vec<_>>(),
            [1, 2, 3],
            "{kind:?}"
        );
    }
}

#[test]
fn visible_overlay_rejects_every_other_kind_before_backend_begin() {
    let data = data(&[1.0, 2.0, 3.0, 4.0]);
    for kind in PlotKind::ALL
        .iter()
        .copied()
        .filter(|kind| !matches!(kind, PlotKind::Boxplot | PlotKind::Violin))
    {
        let mut backend = RecordingBackend::default();
        let error = chart(&data, kind, RawOverlay::Points)
            .render_with_backend(&mut backend)
            .unwrap_err();
        assert!(
            error.to_string().contains("visible RawOverlay"),
            "{kind:?}: {error}"
        );
        assert_eq!(backend.begins, 0, "{kind:?}");
    }
}

#[test]
fn jitter_bytes_are_deterministic_and_value_reordering_does_not_move_source_rows() {
    let first = data(&[1.0, 2.0, 3.0, 4.0]);
    let reordered = data(&[4.0, 3.0, 2.0, 1.0]);
    for kind in [PlotKind::Boxplot, PlotKind::Violin] {
        let first_path = temp_path(&format!("{kind:?}-deterministic-first"));
        let second_path = temp_path(&format!("{kind:?}-deterministic-second"));
        chart(&first, kind, RawOverlay::Jittered)
            .render_to_svg(&first_path)
            .unwrap();
        chart(&first, kind, RawOverlay::Jittered)
            .render_to_svg(&second_path)
            .unwrap();
        assert_eq!(
            std::fs::read(&first_path).unwrap(),
            std::fs::read(&second_path).unwrap(),
            "{kind:?}"
        );
        std::fs::remove_file(first_path).unwrap();
        std::fs::remove_file(second_path).unwrap();
    }

    let mut first_backend = RecordingBackend::default();
    let mut reordered_backend = RecordingBackend::default();
    chart(&first, PlotKind::Boxplot, RawOverlay::Jittered)
        .render_with_backend(&mut first_backend)
        .unwrap();
    chart(&reordered, PlotKind::Boxplot, RawOverlay::Jittered)
        .render_with_backend(&mut reordered_backend)
        .unwrap();
    assert_eq!(
        first_backend.scatters[0]
            .points
            .iter()
            .map(|point| point.x.to_bits())
            .collect::<Vec<_>>(),
        reordered_backend.scatters[0]
            .points
            .iter()
            .map(|point| point.x.to_bits())
            .collect::<Vec<_>>()
    );
}

#[test]
fn explicit_source_row_column_preserves_jitter_after_compaction_and_reordering() {
    let first = DataFrame::new(
        3,
        vec![
            Series::new("category".into(), ["a", "a", "a"]).into(),
            Series::new("value".into(), [1.0_f64, 2.0, 3.0]).into(),
            Series::new("source_row".into(), [5_u64, 1, 9]).into(),
        ],
    )
    .unwrap();
    let reordered = DataFrame::new(
        3,
        vec![
            Series::new("category".into(), ["a", "a", "a"]).into(),
            Series::new("value".into(), [3.0_f64, 1.0, 2.0]).into(),
            Series::new("source_row".into(), [9_u64, 5, 1]).into(),
        ],
    )
    .unwrap();
    let mut first_backend = RecordingBackend::default();
    let mut reordered_backend = RecordingBackend::default();
    let mut first_chart = Chart::new(&first)
        .raw_overlay(RawOverlay::Jittered)
        .raw_overlay_source_row("source_row");
    first_chart.x("category").y("value").boxplot();
    first_chart.render_with_backend(&mut first_backend).unwrap();
    let mut reordered_chart = Chart::new(&reordered)
        .raw_overlay(RawOverlay::Jittered)
        .raw_overlay_source_row("source_row");
    reordered_chart.x("category").y("value").boxplot();
    reordered_chart
        .render_with_backend(&mut reordered_backend)
        .unwrap();

    let first_x = first_backend.scatters[0]
        .points
        .iter()
        .map(|point| point.x.to_bits())
        .collect::<Vec<_>>();
    let reordered_x = reordered_backend.scatters[0]
        .points
        .iter()
        .map(|point| point.x.to_bits())
        .collect::<Vec<_>>();
    assert_eq!(reordered_x, [first_x[2], first_x[0], first_x[1]]);
}

#[test]
fn explicit_source_row_column_rejects_duplicate_identities() {
    let data = DataFrame::new(
        2,
        vec![
            Series::new("category".into(), ["a", "a"]).into(),
            Series::new("value".into(), [1.0_f64, 2.0]).into(),
            Series::new("source_row".into(), [4_u64, 4]).into(),
        ],
    )
    .unwrap();
    let mut chart = Chart::new(&data)
        .raw_overlay(RawOverlay::Jittered)
        .raw_overlay_source_row("source_row");
    chart.x("category").y("value").boxplot();

    let error = chart.preflight().unwrap_err();
    assert!(
        error.to_string().contains("duplicate identity 4"),
        "{error}"
    );
}

#[test]
fn facets_keep_original_source_rows_for_jitter() {
    let data = DataFrame::new(
        8,
        vec![
            Series::new(
                "facet".into(),
                [
                    "left", "right", "left", "right", "left", "right", "left", "right",
                ],
            )
            .into(),
            Series::new("category".into(), ["a"; 8]).into(),
            Series::new("value".into(), [1.0_f64, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]).into(),
        ],
    )
    .unwrap();
    let mut chart = Chart::new(&data)
        .raw_overlay(RawOverlay::Jittered)
        .facet_col("facet");
    let _ = chart.size(900, 480);
    chart.x("category").y("value").boxplot();
    let mut backend = RecordingBackend::default();
    chart.render_with_backend(&mut backend).unwrap();
    assert_eq!(backend.scatters.len(), 2);
    let normalized = backend
        .scatters
        .iter()
        .map(|call| {
            let panel = call.clip.unwrap();
            call.points
                .iter()
                .map(|point| {
                    ((point.x - f64::midpoint(panel.x0, panel.x1)) / panel.width()).to_bits()
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_ne!(normalized[0], normalized[1]);
}

fn faceted_chart(data: &DataFrame, width: u32, height: u32) -> Chart<'_> {
    let mut chart = Chart::new(data)
        .raw_overlay(RawOverlay::Jittered)
        .facet_col("facet");
    let _ = chart.size(width, height);
    chart.x("category").y("value").boxplot();
    chart
}

fn faceted_data() -> DataFrame {
    DataFrame::new(
        4,
        vec![
            Series::new("facet".into(), ["only"; 4]).into(),
            Series::new("category".into(), ["a"; 4]).into(),
            Series::new("value".into(), [1.0_f64, 2.0, 3.0, 4.0]).into(),
        ],
    )
    .unwrap()
}

#[test]
fn faceted_raw_overlay_does_not_draw_provenance_inside_panels() {
    let data = faceted_data();
    let mut backend = RecordingBackend::default();
    faceted_chart(&data, 640, 480)
        .render_with_backend(&mut backend)
        .unwrap();
    assert_eq!((backend.begins, backend.ends), (1, 1));
    assert!(backend
        .texts
        .iter()
        .all(|text| !text.contains("original strict-sample observations")));
}

#[test]
fn grouped_hue_points_follow_dodge_color_and_global_first_observed_order() {
    let data = DataFrame::new(
        8,
        vec![
            Series::new("category".into(), ["a"; 8]).into(),
            Series::new(
                "group".into(),
                ["g2", "g2", "g2", "g2", "g1", "g1", "g1", "g1"],
            )
            .into(),
            Series::new(
                "hue".into(),
                ["h2", "h2", "h2", "h2", "h1", "h1", "h1", "h1"],
            )
            .into(),
            Series::new("value".into(), [1.0_f64, 2.0, 3.0, 4.0, 2.0, 3.0, 4.0, 5.0]).into(),
        ],
    )
    .unwrap();
    let mut chart = Chart::new(&data)
        .group("group")
        .hue("hue")
        .raw_overlay(RawOverlay::Points);
    chart.x("category").y("value").violin();
    let palette = Theme::Publication.palette();
    let mut backend = RecordingBackend::default();
    chart.render_with_backend(&mut backend).unwrap();
    let overlays = backend
        .scatters
        .iter()
        .filter(|call| call.clip.is_some())
        .collect::<Vec<_>>();
    assert_eq!(overlays.len(), 2);
    assert_eq!(overlays[0].style.color, palette.colors[0]);
    assert_eq!(overlays[1].style.color, palette.colors[1]);
    assert!(overlays[0].points[0].x < overlays[1].points[0].x);
    assert!(backend.texts.iter().any(|text| text == "H 2"));
    assert!(backend.texts.iter().any(|text| text == "H 1"));
    assert_eq!(
        backend.texts.iter().filter(|text| *text == "H 2").count(),
        1
    );
    assert_eq!(
        backend.texts.iter().filter(|text| *text == "H 1").count(),
        1
    );
}

#[test]
fn visible_boxplot_overlay_suppresses_separate_fliers_but_keeps_all_rows() {
    let data = data(&[1.0, 1.0, 1.0, 9.0]);
    let mut hidden = RecordingBackend::default();
    chart(&data, PlotKind::Boxplot, RawOverlay::Hidden)
        .render_with_backend(&mut hidden)
        .unwrap();
    assert_eq!(hidden.scatters.len(), 1);
    assert_eq!(hidden.scatters[0].points.len(), 1);

    let mut visible = RecordingBackend::default();
    chart(&data, PlotKind::Boxplot, RawOverlay::Points)
        .render_with_backend(&mut visible)
        .unwrap();
    assert_eq!(visible.scatters.len(), 1);
    assert_eq!(visible.scatters[0].points.len(), 4);
}

#[test]
fn overlay_is_clipped_by_explicit_limits_and_drawn_between_summaries_and_axes() {
    let data = data(&[1.0, 2.0, 3.0, 4.0]);
    let chart = chart(&data, PlotKind::Boxplot, RawOverlay::Points).y_limits(0.0, 2.0);
    let mut backend = RecordingBackend::default();
    chart.render_with_backend(&mut backend).unwrap();
    let scatter = backend
        .events
        .iter()
        .position(|event| *event == ("scatter", 1))
        .unwrap();
    assert!(backend.events[..scatter].iter().any(|event| event.1 == 1));
    assert!(backend.events[scatter + 1..].contains(&("line", 0)));
    let panel = backend.scatters[0].clip.unwrap();
    assert!(backend.scatters[0]
        .points
        .iter()
        .any(|point| point.y > panel.y1));
}

#[test]
fn composed_figure_preserves_overlay_points_and_one_canvas_lifecycle() {
    let data = data(&[1.0, 2.0, 3.0, 4.0]);
    let figure = Figure::grid(1, 2)
        .unwrap()
        .panel(0, 0, chart(&data, PlotKind::Boxplot, RawOverlay::Points))
        .unwrap()
        .panel(0, 1, chart(&data, PlotKind::Violin, RawOverlay::Points))
        .unwrap();
    let mut backend = RecordingBackend::default();
    figure.render_with_backend(&mut backend).unwrap();
    assert_eq!((backend.begins, backend.ends), (1, 1));
    assert_eq!(
        backend
            .scatters
            .iter()
            .map(|call| call.points.len())
            .sum::<usize>(),
        8
    );
}

#[test]
fn narrow_composed_figure_rejects_before_lifecycle_or_artifact_creation() {
    let data = data(&[1.0, 2.0, 3.0, 4.0]);
    let figure = Figure::grid(1, 1)
        .unwrap()
        .panel(0, 0, chart(&data, PlotKind::Violin, RawOverlay::Jittered))
        .unwrap()
        .size(180, 160);
    let mut backend = RecordingBackend::default();
    let error = figure.render_with_backend(&mut backend).unwrap_err();
    assert!(
        error.to_string().contains("figure layout is too dense"),
        "{error}"
    );
    assert_eq!((backend.begins, backend.ends), (0, 0));

    let directory = std::env::temp_dir().join(format!(
        "lichtung-raw-overlay-narrow-figure-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&directory).unwrap();
    let output = directory.join("figure.svg");
    let error = figure.render_to_svg(&output).unwrap_err();
    assert!(
        error.to_string().contains("figure layout is too dense"),
        "{error}"
    );
    assert!(!output.exists());
    assert_eq!(std::fs::read_dir(&directory).unwrap().count(), 0);
    std::fs::remove_dir(directory).unwrap();
}

#[test]
fn adequate_composed_figure_accepts_raw_overlay() {
    let data = data(&[1.0, 2.0, 3.0, 4.0]);
    let figure = Figure::grid(1, 1)
        .unwrap()
        .panel(0, 0, chart(&data, PlotKind::Violin, RawOverlay::Jittered))
        .unwrap()
        .size(640, 480);
    let mut backend = RecordingBackend::default();
    figure.render_with_backend(&mut backend).unwrap();
    assert_eq!((backend.begins, backend.ends), (1, 1));
}

#[test]
fn raw_overlay_report_provenance_is_retained_but_not_drawn() {
    let data = data(&[1.0, 2.0, 3.0, 4.0]);
    let mut backend = RecordingBackend::default();
    chart(&data, PlotKind::Violin, RawOverlay::Jittered)
        .render_with_backend(&mut backend)
        .unwrap();
    let canvas_text = backend.texts.join(" ");
    assert!(!canvas_text.contains("original strict-sample observations"));
    let provenance = RawOverlay::Jittered.provenance();
    assert!(provenance.contains("deterministic source-row displacement"));
    assert!(provenance.contains("summaries unchanged"));
}
