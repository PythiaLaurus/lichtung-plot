use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;

use super::artifacts::TEMP_FILE_COUNTER;
use super::backend::NullBackend;
use super::*;
use crate::layout::PanelBox;
use crate::render::backend::{
    BracketStyle, FillStyle, GradientStop, LineStyle, Point2D, PointStyle, SignificanceBracket,
    TextStyle,
};
use crate::{
    AxisValues, CategoryValues, HeatmapLayout, LegendPolicy, LegendPosition, NumericValues,
    Significance, SignificanceMark, ValueSort,
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

#[derive(Clone)]
struct CanonicalPlotValues {
    group: CategoryValues,
    interval_group: CategoryValues,
    hue: CategoryValues,
    x: NumericValues,
    y: NumericValues,
    color: NumericValues,
    lower: NumericValues,
    upper: NumericValues,
    delta: NumericValues,
    base: f64,
}

fn canonical_plot_values() -> CanonicalPlotValues {
    CanonicalPlotValues {
        group: CategoryValues::new("group", ["a", "a", "b", "b"]).unwrap(),
        interval_group: CategoryValues::new("interval_group", ["a", "b", "c", "d"]).unwrap(),
        hue: CategoryValues::new("hue", ["h1", "h2", "h1", "h2"]).unwrap(),
        x: NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
        y: NumericValues::new("y", [1.0, 3.0, 2.0, 5.0]),
        color: NumericValues::new("color", [0.1, 0.3, 0.7, 1.0]),
        lower: NumericValues::new("lower", [0.5, 1.0, 2.0, 3.0]),
        upper: NumericValues::new("upper", [1.5, 3.0, 4.0, 5.0]),
        delta: NumericValues::new("delta", [1.0, -1.0, 2.0, 0.0]),
        base: 10.0,
    }
}

#[derive(Clone)]
struct BarValues {
    category: CategoryValues,
    value: NumericValues,
    series: Option<CategoryValues>,
}

fn valid_grouped_bar_values() -> BarValues {
    BarValues {
        category: CategoryValues::new("group", ["a", "a", "a", "a", "b", "b", "b", "b"]).unwrap(),
        series: Some(
            CategoryValues::new("hue", ["h1", "h1", "h2", "h2", "h1", "h1", "h2", "h2"]).unwrap(),
        ),
        value: NumericValues::new("y", [1.0, 2.0, 3.0, 4.0, 2.0, 3.0, 4.0, 5.0]),
    }
}

fn configured_chart(data: &CanonicalPlotValues, kind: PlotKind) -> Plot {
    match kind {
        PlotKind::Scatter => Plot::scatter(data.x.clone(), data.y.clone()).erase(),
        PlotKind::Histogram => {
            Plot::histogram(crate::HistogramInput::samples(data.x.clone())).erase()
        }
        PlotKind::Density => Plot::density(data.x.clone()).erase(),
        PlotKind::Qq => Plot::qq(data.x.clone()).erase(),
        PlotKind::PointInterval => Plot::point_interval(
            data.interval_group.clone(),
            data.x.clone(),
            crate::Bounds::new(data.lower.clone(), data.upper.clone()),
        )
        .erase(),
        PlotKind::Line => Plot::line(data.x.clone(), data.y.clone()).erase(),
        PlotKind::Trajectory => Plot::trajectory(
            AxisValues::Numeric(data.x.clone()),
            data.y.clone(),
            data.group.clone(),
        )
        .erase(),
        PlotKind::LinePath => {
            Plot::line_path(data.x.clone(), data.y.clone(), data.group.clone()).erase()
        }
        PlotKind::Interaction => Plot::interaction(
            data.group.clone(),
            data.y.clone(),
            data.hue.clone(),
            crate::Bounds::new(data.lower.clone(), data.upper.clone()),
        )
        .hue(data.hue.clone())
        .erase(),
        PlotKind::Coefficient => Plot::coefficient(
            data.interval_group.clone(),
            crate::CoefficientInput::estimates_with_intervals(
                data.x.clone(),
                crate::Bounds::new(data.lower.clone(), data.upper.clone()),
            ),
        )
        .erase(),
        PlotKind::Forest => Plot::forest(
            data.interval_group.clone(),
            data.x.clone(),
            crate::Bounds::new(data.lower.clone(), data.upper.clone()),
            0.0,
        )
        .erase(),
        PlotKind::Dumbbell => Plot::dumbbell(
            data.group.clone(),
            data.x.clone(),
            data.y.clone(),
            crate::DumbbellEndpoints::new("Left", (20, 80, 140), "Right", (180, 60, 40)).unwrap(),
        )
        .group(data.interval_group.clone())
        .erase(),
        PlotKind::Heatmap => Plot::heatmap(
            data.interval_group.clone(),
            data.group.clone(),
            data.color.clone(),
        )
        .erase(),
        PlotKind::Bar => {
            Plot::bar(data.group.clone(), crate::BarInput::samples(data.y.clone())).erase()
        }
        PlotKind::HorizontalBar => {
            Plot::horizontal_bar(data.group.clone(), crate::BarInput::samples(data.y.clone()))
                .erase()
        }
        PlotKind::StackedBar => {
            Plot::stacked_bar(data.group.clone(), data.y.clone(), data.hue.clone()).erase()
        }
        PlotKind::StackedHorizontalBar => {
            Plot::stacked_horizontal_bar(data.group.clone(), data.y.clone(), data.hue.clone())
                .erase()
        }
        PlotKind::Boxplot => Plot::boxplot(data.group.clone(), data.y.clone()).erase(),
        PlotKind::Violin => Plot::violin(data.group.clone(), data.y.clone()).erase(),
        PlotKind::Raincloud => Plot::raincloud(crate::RaincloudInput::samples(
            data.group.clone(),
            data.y.clone(),
        ))
        .erase(),
        PlotKind::Regression => Plot::regression(data.x.clone(), data.y.clone()).erase(),
        PlotKind::Marginal => Plot::marginal(data.x.clone(), data.y.clone()).erase(),
        PlotKind::Matrix => Plot::matrix(vec![data.x.clone(), data.y.clone()]).erase(),
        PlotKind::Beeswarm => {
            Plot::beeswarm(data.group.clone(), data.x.clone(), data.color.clone()).erase()
        }
        PlotKind::Waterfall => {
            Plot::waterfall(data.group.clone(), data.delta.clone(), data.base).erase()
        }
    }
}

fn configured_bar_chart(data: &BarValues, kind: PlotKind) -> Plot {
    match kind {
        PlotKind::Bar => Plot::bar(
            data.category.clone(),
            crate::BarInput::samples(data.value.clone()),
        )
        .erase(),
        PlotKind::HorizontalBar => Plot::horizontal_bar(
            data.category.clone(),
            crate::BarInput::samples(data.value.clone()),
        )
        .erase(),
        PlotKind::StackedBar => Plot::stacked_bar(
            data.category.clone(),
            data.value.clone(),
            data.series.as_ref().unwrap().clone(),
        )
        .erase(),
        PlotKind::StackedHorizontalBar => Plot::stacked_horizontal_bar(
            data.category.clone(),
            data.value.clone(),
            data.series.as_ref().unwrap().clone(),
        )
        .erase(),
        _ => panic!("unsupported bar kind {kind:?}"),
    }
}

#[derive(Clone, Copy)]
enum NumericField {
    X,
    Y,
    Color,
    Lower,
    Upper,
    Delta,
}

impl NumericField {
    const fn name(self) -> &'static str {
        match self {
            Self::X => "x",
            Self::Y => "y",
            Self::Color => "color",
            Self::Lower => "lower",
            Self::Upper => "upper",
            Self::Delta => "delta",
        }
    }

    const fn valid_values(self) -> [f64; 4] {
        match self {
            Self::X => [1.0, 2.0, 3.0, 4.0],
            Self::Y => [1.0, 3.0, 2.0, 5.0],
            Self::Color => [0.1, 0.3, 0.7, 1.0],
            Self::Lower => [0.5, 1.0, 2.0, 3.0],
            Self::Upper => [1.5, 3.0, 4.0, 5.0],
            Self::Delta => [1.0, -1.0, 2.0, 0.0],
        }
    }
}

fn numeric_fault_values(field: NumericField, fault: Option<f64>) -> CanonicalPlotValues {
    let mut data = canonical_plot_values();
    let mut values = field.valid_values().map(Some);
    values[1] = fault;
    let values = NumericValues::nullable(field.name(), values);
    match field {
        NumericField::X => data.x = values,
        NumericField::Y => data.y = values,
        NumericField::Color => data.color = values,
        NumericField::Lower => data.lower = values,
        NumericField::Upper => data.upper = values,
        NumericField::Delta => data.delta = values,
    }
    data
}

#[derive(Clone, Copy)]
enum CategoryField {
    Group,
    IntervalGroup,
    Hue,
}

impl CategoryField {
    const fn name(self) -> &'static str {
        match self {
            Self::Group => "group",
            Self::IntervalGroup => "interval_group",
            Self::Hue => "hue",
        }
    }

    const fn valid_values(self) -> [&'static str; 4] {
        match self {
            Self::Group => ["a", "a", "b", "b"],
            Self::IntervalGroup => ["a", "b", "c", "d"],
            Self::Hue => ["h1", "h2", "h1", "h2"],
        }
    }
}

fn categorical_fault_values(field: CategoryField) -> CanonicalPlotValues {
    let mut data = canonical_plot_values();
    let mut values = field.valid_values().map(Some);
    values[1] = None;
    let values = CategoryValues::nullable(field.name(), values).unwrap();
    match field {
        CategoryField::Group => data.group = values,
        CategoryField::IntervalGroup => data.interval_group = values,
        CategoryField::Hue => data.hue = values,
    }
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
    let output = directory.join("canvas.svg");
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
