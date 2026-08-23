use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;

#[cfg(windows)]
use super::artifacts::finish_committed_publication;
use super::artifacts::TEMP_FILE_COUNTER;
use super::backend::NullBackend;
use super::*;
use crate::layout::PanelBox;
use crate::render::backend::{
    BracketStyle, FillStyle, GradientStop, LineStyle, Point2D, PointStyle, SignificanceBracket,
    TextStyle,
};
use crate::test_support::{DataFrame, Fixture, Series};
use crate::{
    HeatmapLayout, LegendPolicy, LegendPosition, Significance, SignificanceMark, ValueSort,
};

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

fn fixture() -> Fixture {
    crate::test_support::test_data!(
        "group" => ["a", "a", "b", "b"],
        "interval_group" => ["a", "b", "c", "d"],
        "hue" => ["h1", "h2", "h1", "h2"],
        "x" => [1.0_f64, 2.0, 3.0, 4.0],
        "y" => [1.0_f64, 3.0, 2.0, 5.0],
        "color" => [0.1_f64, 0.3, 0.7, 1.0],
        "lower" => [0.5_f64, 1.0, 2.0, 3.0],
        "upper" => [1.5_f64, 3.0, 4.0, 5.0],
        "delta" => [1.0_f64, -1.0, 2.0, 0.0],
        "base" => [10.0_f64; 4],
    )
    .unwrap()
}

fn valid_grouped_bar_fixture() -> Fixture {
    crate::test_support::test_data!(
        "group" => ["a", "a", "a", "a", "b", "b", "b", "b"],
        "hue" => ["h1", "h1", "h2", "h2", "h1", "h1", "h2", "h2"],
        "y" => [1.0_f64, 2.0, 3.0, 4.0, 2.0, 3.0, 4.0, 5.0],
    )
    .unwrap()
}

#[expect(
    clippy::too_many_lines,
    reason = "one exhaustive test factory keeps every PlotKind arm visible"
)]
fn configured_chart(data: &Fixture, kind: PlotKind) -> Plot {
    use crate::test_support::{axis, category, constant_numeric, numeric};

    match kind {
        PlotKind::Scatter => Plot::scatter(numeric(data, "x"), numeric(data, "y")).erase(),
        PlotKind::Histogram => {
            Plot::histogram(crate::HistogramInput::samples(numeric(data, "x"))).erase()
        }
        PlotKind::Density => Plot::density(numeric(data, "x")).erase(),
        PlotKind::Qq => Plot::qq(numeric(data, "x")).erase(),
        PlotKind::PointInterval => Plot::point_interval(
            category(data, "interval_group"),
            numeric(data, "x"),
            crate::Bounds::new(numeric(data, "lower"), numeric(data, "upper")),
        )
        .erase(),
        PlotKind::Line => Plot::line(numeric(data, "x"), numeric(data, "y")).erase(),
        PlotKind::Trajectory => {
            Plot::trajectory(axis(data, "x"), numeric(data, "y"), category(data, "group")).erase()
        }
        PlotKind::LinePath => Plot::line_path(
            numeric(data, "x"),
            numeric(data, "y"),
            category(data, "group"),
        )
        .erase(),
        PlotKind::Interaction => Plot::interaction(
            category(data, "group"),
            numeric(data, "y"),
            category(data, "hue"),
            crate::Bounds::new(numeric(data, "lower"), numeric(data, "upper")),
        )
        .hue(category(data, "hue"))
        .erase(),
        PlotKind::Coefficient => Plot::coefficient(
            category(data, "interval_group"),
            crate::CoefficientInput::estimates_with_intervals(
                numeric(data, "x"),
                crate::Bounds::new(numeric(data, "lower"), numeric(data, "upper")),
            ),
        )
        .erase(),
        PlotKind::Forest => Plot::forest(
            category(data, "interval_group"),
            numeric(data, "x"),
            crate::Bounds::new(numeric(data, "lower"), numeric(data, "upper")),
            0.0,
        )
        .erase(),
        PlotKind::Dumbbell => Plot::dumbbell(
            category(data, "group"),
            numeric(data, "x"),
            numeric(data, "y"),
            crate::DumbbellEndpoints::new("Left", (20, 80, 140), "Right", (180, 60, 40)).unwrap(),
        )
        .group(category(data, "interval_group"))
        .erase(),
        PlotKind::Heatmap => Plot::heatmap(
            category(data, "interval_group"),
            category(data, "group"),
            numeric(data, "color"),
        )
        .erase(),
        PlotKind::Bar => Plot::bar(
            category(data, "group"),
            crate::BarInput::samples(numeric(data, "y")),
        )
        .erase(),
        PlotKind::HorizontalBar => Plot::horizontal_bar(
            category(data, "group"),
            crate::BarInput::samples(numeric(data, "y")),
        )
        .erase(),
        PlotKind::StackedBar => Plot::stacked_bar(
            category(data, "group"),
            numeric(data, "y"),
            category(data, "hue"),
        )
        .hue(category(data, "hue"))
        .erase(),
        PlotKind::StackedHorizontalBar => Plot::stacked_horizontal_bar(
            category(data, "group"),
            numeric(data, "y"),
            category(data, "hue"),
        )
        .hue(category(data, "hue"))
        .erase(),
        PlotKind::Boxplot => Plot::boxplot(category(data, "group"), numeric(data, "y")).erase(),
        PlotKind::Violin => Plot::violin(category(data, "group"), numeric(data, "y")).erase(),
        PlotKind::Raincloud => Plot::raincloud(crate::RaincloudInput::samples(
            category(data, "group"),
            numeric(data, "y"),
        ))
        .erase(),
        PlotKind::Regression => Plot::regression(numeric(data, "x"), numeric(data, "y")).erase(),
        PlotKind::Marginal => Plot::marginal(numeric(data, "x"), numeric(data, "y")).erase(),
        PlotKind::Matrix => Plot::matrix(vec![numeric(data, "x"), numeric(data, "y")]).erase(),
        PlotKind::Beeswarm => Plot::beeswarm(
            category(data, "group"),
            numeric(data, "x"),
            numeric(data, "color"),
        )
        .erase(),
        PlotKind::Waterfall => Plot::waterfall(
            category(data, "group"),
            numeric(data, "delta"),
            constant_numeric(data, "base"),
        )
        .erase(),
    }
}

fn configured_bar_chart(data: &Fixture, kind: PlotKind) -> Plot {
    use crate::test_support::{category, numeric};

    match kind {
        PlotKind::Bar => Plot::bar(
            category(data, "group"),
            crate::BarInput::samples(numeric(data, "value")),
        )
        .erase(),
        PlotKind::HorizontalBar => Plot::horizontal_bar(
            category(data, "group"),
            crate::BarInput::samples(numeric(data, "value")),
        )
        .erase(),
        PlotKind::StackedBar => Plot::stacked_bar(
            category(data, "group"),
            numeric(data, "value"),
            category(data, "hue"),
        )
        .hue(category(data, "hue"))
        .erase(),
        PlotKind::StackedHorizontalBar => Plot::stacked_horizontal_bar(
            category(data, "group"),
            numeric(data, "value"),
            category(data, "hue"),
        )
        .hue(category(data, "hue"))
        .erase(),
        _ => panic!("unsupported bar fixture kind {kind:?}"),
    }
}

fn numeric_fault_fixture(column: &str, fault: Option<f64>) -> Fixture {
    let mut data = fixture();
    let mut values = match column {
        "x" => vec![1.0_f64, 2.0, 3.0, 4.0],
        "y" => vec![1.0_f64, 3.0, 2.0, 5.0],
        "color" => vec![0.1_f64, 0.3, 0.7, 1.0],
        "lower" => vec![0.5_f64, 1.0, 2.0, 3.0],
        "upper" => vec![1.5_f64, 3.0, 4.0, 5.0],
        "delta" => vec![1.0_f64, -1.0, 2.0, 0.0],
        "base" => vec![10.0_f64; 4],
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

fn categorical_fault_fixture(column: &str) -> Fixture {
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

fn assert_rejected_before_begin<K: crate::kind::Marker>(chart: &Plot<K>, kind: PlotKind) -> String {
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

fn svg_bytes<K: crate::kind::Marker>(plot: Plot<K>) -> Vec<u8> {
    let directory = test_directory("svg-bytes");
    let output = directory.join("figure.svg");
    plot.svg(&output).unwrap();
    let bytes = std::fs::read(&output).unwrap();
    std::fs::remove_dir_all(directory).unwrap();
    bytes
}

fn assert_no_staging_files(directory: &Path) {
    let leaked = std::fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|name| name.contains(".lichtung-tmp-"))
        .collect::<Vec<_>>();
    assert!(leaked.is_empty(), "leaked staging files: {leaked:?}");
}
