#![allow(
    dead_code,
    reason = "helpers are shared by independently compiled integration test targets"
)]

use lichtung_plot::layout::PanelBox;
use lichtung_plot::render::backend::{
    Backend, BracketStyle, FillStyle, GradientStop, LayerRole, LineStyle, Point2D, PointStyle,
    SignificanceBracket, TextStyle,
};
use lichtung_plot::{Error, InferentialScale, IntervalKind, IntervalSemantics};
use polars::prelude::*;

#[derive(Default)]
pub(super) struct LifecycleBackend {
    pub(super) begins: usize,
    pub(super) ends: usize,
    pub(super) clips: usize,
    pub(super) clip_regions: Vec<PanelBox>,
    pub(super) clip_ends: usize,
    pub(super) events: Vec<RenderEvent>,
    pub(super) texts: Vec<(u32, u32, String)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RenderEvent {
    Grid,
    Marks,
}

impl Backend for LifecycleBackend {
    fn begin(&mut self, _width: u32, _height: u32) -> Result<(), Error> {
        self.begins += 1;
        Ok(())
    }

    fn begin_clip(&mut self, region: PanelBox) -> Result<(), Error> {
        self.clips += 1;
        self.clip_regions.push(region);
        self.events.push(RenderEvent::Marks);
        Ok(())
    }

    fn begin_layer_with_role(&mut self, _name: &str, role: LayerRole) -> Result<(), Error> {
        if role == LayerRole::Grid {
            self.events.push(RenderEvent::Grid);
        }
        Ok(())
    }

    fn end_clip(&mut self) -> Result<(), Error> {
        self.clip_ends += 1;
        Ok(())
    }

    fn draw_scatter(&mut self, _points: &[Point2D], _style: &PointStyle) -> Result<(), Error> {
        Ok(())
    }

    fn draw_line(&mut self, _points: &[Point2D], _style: &LineStyle) -> Result<(), Error> {
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
        x_pixel: u32,
        y_pixel: u32,
        text: &str,
        _style: &TextStyle,
    ) -> Result<(), Error> {
        self.texts.push((x_pixel, y_pixel, text.to_owned()));
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

pub(super) fn estimate_data() -> DataFrame {
    df!(
        "x" => [2.0_f64, 1.0, 2.0, 1.0],
        "estimate" => [3.0_f64, 2.0, 4.0, 2.5],
        "lower" => [2.5_f64, 1.5, 3.5, 2.0],
        "upper" => [3.5_f64, 2.5, 4.5, 3.0],
        "trace" => ["control", "control", "active", "active"],
        "factor" => ["after", "before", "after", "before"],
        "term" => ["intercept", "time", "group", "interaction"],
        "facet" => ["one", "one", "two", "two"]
    )
    .unwrap()
}

pub(super) fn confidence(scale: InferentialScale) -> IntervalSemantics {
    IntervalSemantics::new(IntervalKind::Confidence, 0.95, scale, "model fit").unwrap()
}

pub(super) fn assert_unique_svg_ids(svg: &str) {
    let mut ids = std::collections::HashSet::new();
    for remainder in svg.split(" id=\"").skip(1) {
        let id = remainder.split('"').next().unwrap();
        assert!(ids.insert(id), "duplicate SVG id {id:?}");
    }
}

pub(super) fn assert_grid_layers_before_marks(svg: &str, grid_ids: &[&str]) {
    let first_marks = svg.find("<g clip-path=").unwrap();
    for grid_id in grid_ids {
        let grid = svg.find(&format!("<g id=\"{grid_id}\">")).unwrap();
        assert!(
            grid < first_marks,
            "grid layer {grid_id:?} rendered after marks"
        );
    }
}

pub(super) fn attribute<'a>(line: &'a str, name: &str) -> &'a str {
    let marker = format!("{name}=\"");
    let start = line.find(&marker).unwrap() + marker.len();
    let end = line[start..].find('"').unwrap() + start;
    &line[start..end]
}

pub(super) fn tempdir(name: &str) -> std::path::PathBuf {
    let directory = std::env::temp_dir().join(format!(
        "lichtung-plot-composition-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&directory).unwrap();
    directory
}
