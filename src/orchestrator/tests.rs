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
    CategoryValues, HeatmapLayout, LegendPolicy, LegendPosition, NumericValues, Significance,
    SignificanceMark, ValueSort,
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
    begin_sizes: Vec<(u32, u32)>,
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
    texts: Vec<(f64, f64, String)>,
    text_draws: Vec<(f64, f64, String, TextStyle)>,
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
    fn begin(&mut self, width: u32, height: u32) -> Result<(), Error> {
        self.begins += 1;
        self.begin_sizes.push((width, height));
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

    fn draw_text(&mut self, position: Point2D, text: &str, style: &TextStyle) -> Result<(), Error> {
        self.texts.push((position.x, position.y, text.to_string()));
        self.text_draws
            .push((position.x, position.y, text.to_string(), style.clone()));
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
    plot.svg_preserve_text(&output).unwrap();
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
