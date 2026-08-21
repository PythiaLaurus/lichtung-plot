use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;

use polars::prelude::{DataFrame, NamedFrom, Series};

#[cfg(any(feature = "png", windows))]
use super::artifacts::finish_committed_publication;
#[cfg(feature = "png")]
use super::artifacts::reserve_staging_pair;
use super::artifacts::TEMP_FILE_COUNTER;
use super::backend::NullBackend;
use super::*;
use crate::layout::PanelBox;
use crate::render::backend::{
    BracketStyle, FillStyle, GradientStop, LineStyle, Point2D, PointStyle, SignificanceBracket,
    TextStyle,
};
use crate::ComparisonAnnotation;
use crate::{HeatmapLayout, LegendPosition, ValueSort};

#[path = "tests/artifacts.rs"]
mod artifacts;
#[path = "tests/data_contracts.rs"]
mod data_contracts;
#[path = "tests/facets.rs"]
mod facets;
#[path = "tests/focused_plot_options.rs"]
mod focused_plot_options;
#[path = "tests/lifecycle.rs"]
mod lifecycle;
#[path = "tests/specialized.rs"]
mod specialized;
#[path = "tests/validation.rs"]
mod validation;

#[derive(Clone, Copy, PartialEq, Eq)]
enum BackendFailure {
    Begin,
    Scatter,
    Line,
    ClipEnd,
    LayerEnd,
    PolygonInLayer,
    End,
}

#[derive(Default)]
struct RecordingBackend {
    begins: usize,
    ends: usize,
    clips: Vec<PanelBox>,
    clip_ends: usize,
    active_clips: usize,
    layer_ends: usize,
    active_layers: usize,
    scatter_points: Vec<Point2D>,
    scatter_draws: Vec<(Vec<Point2D>, PointStyle, usize)>,
    line_draws: Vec<(Vec<Point2D>, LineStyle, usize)>,
    gradients: Vec<Vec<GradientStop>>,
    texts: Vec<(u32, u32, String)>,
    text_draws: Vec<(u32, u32, String, TextStyle)>,
    rounded_rects: Vec<PanelBox>,
    bracket_clip_depths: Vec<usize>,
    failures: Vec<BackendFailure>,
}

impl RecordingBackend {
    fn should_fail(&self, failure: BackendFailure) -> bool {
        self.failures.contains(&failure)
    }
}

impl Backend for RecordingBackend {
    fn begin(&mut self, _width: u32, _height: u32) -> Result<(), Error> {
        self.begins += 1;
        if self.should_fail(BackendFailure::Begin) {
            Err(Error::Render("canvas begin failed".into()))
        } else {
            Ok(())
        }
    }

    fn begin_clip(&mut self, region: PanelBox) -> Result<(), Error> {
        crate::render::backend::validate_clip_region(region)?;
        self.clips.push(region);
        self.active_clips += 1;
        Ok(())
    }

    fn begin_layer(&mut self, _name: &str) -> Result<(), Error> {
        self.active_layers += 1;
        Ok(())
    }

    fn end_clip(&mut self) -> Result<(), Error> {
        self.clip_ends += 1;
        self.active_clips = self.active_clips.saturating_sub(1);
        if self.should_fail(BackendFailure::ClipEnd) {
            Err(Error::Render("clip cleanup failed".into()))
        } else {
            Ok(())
        }
    }

    fn end_layer(&mut self) -> Result<(), Error> {
        self.layer_ends += 1;
        self.active_layers = self.active_layers.saturating_sub(1);
        if self.should_fail(BackendFailure::LayerEnd) {
            Err(Error::Render("layer cleanup failed".into()))
        } else {
            Ok(())
        }
    }

    fn draw_scatter(&mut self, points: &[Point2D], style: &PointStyle) -> Result<(), Error> {
        if self.should_fail(BackendFailure::Scatter) {
            return Err(Error::Render("scatter draw failed".into()));
        }
        self.scatter_points.extend_from_slice(points);
        self.scatter_draws
            .push((points.to_vec(), style.clone(), self.active_clips));
        Ok(())
    }

    fn draw_line(&mut self, points: &[Point2D], style: &LineStyle) -> Result<(), Error> {
        if self.should_fail(BackendFailure::Line) {
            Err(Error::Render("line draw failed".into()))
        } else {
            self.line_draws
                .push((points.to_vec(), style.clone(), self.active_clips));
            Ok(())
        }
    }

    fn draw_polygon(&mut self, _points: &[Point2D], _style: &FillStyle) -> Result<(), Error> {
        if self.should_fail(BackendFailure::PolygonInLayer) && self.active_layers > 0 {
            Err(Error::Render("layer polygon draw failed".into()))
        } else {
            Ok(())
        }
    }

    fn draw_linear_gradient(
        &mut self,
        _bounds: PanelBox,
        _start: Point2D,
        _end: Point2D,
        stops: &[GradientStop],
    ) -> Result<(), Error> {
        self.gradients.push(stops.to_vec());
        Ok(())
    }

    fn draw_text(
        &mut self,
        x_pixel: u32,
        y_pixel: u32,
        text: &str,
        style: &TextStyle,
    ) -> Result<(), Error> {
        self.texts.push((x_pixel, y_pixel, text.to_string()));
        self.text_draws
            .push((x_pixel, y_pixel, text.to_string(), style.clone()));
        Ok(())
    }

    fn draw_rounded_rect(
        &mut self,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        _radius: f64,
        style: &FillStyle,
    ) -> Result<(), Error> {
        self.rounded_rects
            .push(PanelBox::new(x, x + width, y, y + height));
        self.draw_polygon(
            &[
                Point2D::new(x, y),
                Point2D::new(x + width, y),
                Point2D::new(x + width, y + height),
                Point2D::new(x, y + height),
            ],
            style,
        )
    }

    fn draw_bracket(
        &mut self,
        _bracket: &SignificanceBracket,
        _style: &BracketStyle,
    ) -> Result<(), Error> {
        self.bracket_clip_depths.push(self.active_clips);
        Ok(())
    }

    fn end(&mut self) -> Result<(), Error> {
        self.ends += 1;
        if self.should_fail(BackendFailure::End) {
            Err(Error::Render("canvas cleanup failed".into()))
        } else {
            Ok(())
        }
    }
}

fn fixture() -> DataFrame {
    let group = Series::new("group".into(), ["a", "a", "b", "b"]);
    let interval_group = Series::new("interval_group".into(), ["a", "b", "c", "d"]);
    let hue = Series::new("hue".into(), ["h1", "h2", "h1", "h2"]);
    let x = Series::new("x".into(), [1.0_f64, 2.0, 3.0, 4.0]);
    let y = Series::new("y".into(), [1.0_f64, 3.0, 2.0, 5.0]);
    let color = Series::new("color".into(), [0.1_f64, 0.3, 0.7, 1.0]);
    let lower = Series::new("lower".into(), [0.5_f64, 1.0, 2.0, 3.0]);
    let upper = Series::new("upper".into(), [1.5_f64, 3.0, 4.0, 5.0]);
    let delta = Series::new("delta".into(), [1.0_f64, -1.0, 2.0, 0.0]);
    let base = Series::new("base".into(), [10.0_f64; 4]);
    let terminal = Series::new("terminal".into(), [12.0_f64; 4]);
    DataFrame::new(
        group.len(),
        vec![
            group.into(),
            interval_group.into(),
            hue.into(),
            x.into(),
            y.into(),
            color.into(),
            lower.into(),
            upper.into(),
            delta.into(),
            base.into(),
            terminal.into(),
        ],
    )
    .unwrap()
}

fn valid_grouped_bar_fixture() -> DataFrame {
    DataFrame::new(
        8,
        vec![
            Series::new("group".into(), ["a", "a", "a", "a", "b", "b", "b", "b"]).into(),
            Series::new(
                "hue".into(),
                ["h1", "h1", "h2", "h2", "h1", "h1", "h2", "h2"],
            )
            .into(),
            Series::new("y".into(), [1.0_f64, 2.0, 3.0, 4.0, 2.0, 3.0, 4.0, 5.0]).into(),
        ],
    )
    .unwrap()
}

fn configured_chart(data: &DataFrame, kind: PlotKind) -> Chart<'_> {
    let mut chart = match kind {
        PlotKind::StackedBar | PlotKind::StackedHorizontalBar => {
            Chart::new(data).group("hue").hue("hue")
        }
        PlotKind::Interaction => Chart::new(data)
            .group("hue")
            .hue("hue")
            .interval_bounds("lower", "upper")
            .interval_semantics(
                crate::IntervalSemantics::new(
                    crate::IntervalKind::Confidence,
                    0.95,
                    crate::InferentialScale::Response,
                    "orchestrator fixture",
                )
                .unwrap(),
            )
            .interaction_source(crate::EstimateSource::EstimatedMarginalMean),
        PlotKind::Coefficient => Chart::new(data)
            .interval_bounds("lower", "upper")
            .interval_semantics(
                crate::IntervalSemantics::new(
                    crate::IntervalKind::Confidence,
                    0.95,
                    crate::InferentialScale::Coefficient,
                    "orchestrator fixture",
                )
                .unwrap(),
            ),
        PlotKind::PointInterval => Chart::new(data).interval_bounds("lower", "upper"),
        PlotKind::Forest => Chart::new(data)
            .interval_bounds("lower", "upper")
            .interval_semantics(
                crate::IntervalSemantics::new(
                    crate::IntervalKind::Confidence,
                    0.95,
                    crate::InferentialScale::Coefficient,
                    "orchestrator fixture",
                )
                .unwrap(),
            )
            .reference_line(0.0),
        PlotKind::Dumbbell => Chart::new(data)
            .group("interval_group")
            .dumbbell_right("y")
            .dumbbell_endpoints(
                crate::DumbbellEndpoints::new("Left", (20, 80, 140), "Right", (180, 60, 40))
                    .unwrap(),
            ),
        PlotKind::Beeswarm | PlotKind::Heatmap => Chart::new(data).color("color"),
        PlotKind::Waterfall => Chart::new(data).waterfall_totals("base", "terminal"),
        _ => Chart::new(data),
    };
    match kind {
        PlotKind::Bar
        | PlotKind::HorizontalBar
        | PlotKind::StackedBar
        | PlotKind::StackedHorizontalBar
        | PlotKind::Boxplot
        | PlotKind::Violin
        | PlotKind::Interaction => {
            let _ = chart.x("group");
            let _ = chart.y("y");
        }
        PlotKind::Beeswarm => {
            let _ = chart.x("x");
            let _ = chart.y("group");
        }
        PlotKind::PointInterval | PlotKind::Coefficient | PlotKind::Forest => {
            let _ = chart.x("x");
            let _ = chart.y("interval_group");
        }
        PlotKind::Dumbbell => {
            let _ = chart.x("group");
            let _ = chart.y("x");
        }
        PlotKind::Heatmap => {
            let _ = chart.x("interval_group");
            let _ = chart.y("group");
        }
        PlotKind::Waterfall => {
            let _ = chart.x("group");
            let _ = chart.y("delta");
        }
        PlotKind::Matrix => {
            let _ = chart.matrix_variables(vec!["x", "y"]);
        }
        _ => {
            let _ = chart.x("x");
            let _ = chart.y("y");
        }
    }
    if kind == PlotKind::MassSpectrum {
        let _ = chart.add_peaks(vec![(2.0, 3.0)]);
    }
    let _ = chart.select(kind);
    chart
}

fn configured_bar_chart(data: &DataFrame, kind: PlotKind) -> Chart<'_> {
    let mut chart = if matches!(kind, PlotKind::StackedBar | PlotKind::StackedHorizontalBar) {
        Chart::new(data).group("hue").hue("hue")
    } else {
        Chart::new(data)
    };
    let _ = chart.x("group");
    let _ = chart.y("value");
    let _ = chart.select(kind);
    chart
}

fn numeric_fault_fixture(column: &str, fault: Option<f64>) -> DataFrame {
    let mut data = fixture();
    let mut values = match column {
        "x" => vec![1.0_f64, 2.0, 3.0, 4.0],
        "y" => vec![1.0_f64, 3.0, 2.0, 5.0],
        "color" => vec![0.1_f64, 0.3, 0.7, 1.0],
        "lower" => vec![0.5_f64, 1.0, 2.0, 3.0],
        "upper" => vec![1.5_f64, 3.0, 4.0, 5.0],
        "delta" => vec![1.0_f64, -1.0, 2.0, 0.0],
        "base" => vec![10.0_f64; 4],
        "terminal" => vec![12.0_f64; 4],
        _ => panic!("unsupported numeric fixture column {column}"),
    }
    .into_iter()
    .map(Some)
    .collect::<Vec<_>>();
    values[1] = fault;
    data.with_column(Series::new(column.into(), values).into())
        .unwrap();
    data
}

fn categorical_fault_fixture(column: &str) -> DataFrame {
    let mut data = fixture();
    let mut values = match column {
        "group" => vec!["a", "a", "b", "b"],
        "interval_group" => vec!["a", "b", "c", "d"],
        "hue" => vec!["h1", "h2", "h1", "h2"],
        _ => panic!("unsupported categorical fixture column {column}"),
    }
    .into_iter()
    .map(Some)
    .collect::<Vec<_>>();
    values[1] = None;
    data.with_column(Series::new(column.into(), values).into())
        .unwrap();
    data
}

fn assert_rejected_before_begin(chart: &Chart<'_>, kind: PlotKind) -> String {
    let mut backend = RecordingBackend::default();
    let error = chart.render_kind(kind, &mut backend).unwrap_err();
    assert_eq!((backend.begins, backend.ends), (0, 0), "{kind:?}");
    error.to_string()
}

fn test_directory(name: &str) -> PathBuf {
    let sequence = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "lichtung-plot-{name}-{}-{sequence}",
        std::process::id()
    ));
    std::fs::create_dir(&path).unwrap();
    path
}

fn assert_no_staging_files(directory: &Path) {
    let leaked = std::fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|name| name.contains(".lichtung-tmp-"))
        .collect::<Vec<_>>();
    assert!(leaked.is_empty(), "leaked staging files: {leaked:?}");
}
