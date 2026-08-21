use lichtung_plot::layout::PanelBox;
use lichtung_plot::render::backend::{
    Backend, BracketStyle, FillStyle, GradientStop, LineStyle, Point2D, PointStyle,
    SignificanceBracket, TextStyle,
};
use lichtung_plot::Error;
use lichtung_plot::{
    Chart, EstimateSource, InferentialScale, IntervalKind, IntervalSemantics, LinePattern,
    PointShape,
};
use polars::prelude::*;

#[derive(Default)]
struct BeginCountingBackend {
    begins: usize,
}

impl Backend for BeginCountingBackend {
    fn begin(&mut self, _width: u32, _height: u32) -> Result<(), Error> {
        self.begins += 1;
        Ok(())
    }

    fn begin_clip(&mut self, _panel: PanelBox) -> Result<(), Error> {
        Ok(())
    }

    fn end_clip(&mut self) -> Result<(), Error> {
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
        _x: u32,
        _y: u32,
        _text: &str,
        _style: &TextStyle,
    ) -> Result<(), Error> {
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
        Ok(())
    }
}

fn data() -> DataFrame {
    df!(
        "condition" => ["Before", "After", "Before", "After"],
        "series" => ["Control", "Control", "Active", "Active"],
        "estimate" => [1.0_f64, 1.5, 1.2, 2.0],
        "lower" => [0.8_f64, 1.3, 1.0, 1.8],
        "upper" => [1.2_f64, 1.7, 1.4, 2.2],
        "shape" => [PointShape::Circle.as_str(); 4],
        "pattern" => [LinePattern::Solid.as_str(); 4],
    )
    .unwrap()
}

fn connector_collision_data() -> DataFrame {
    df!(
        "condition" => ["one", "two", "three", "four"],
        "series" => ["A"; 4],
        "estimate" => [0.0_f64, 0.0, 1.0, 0.6],
        "lower" => [-0.01_f64, -0.01, 0.99, 0.59],
        "upper" => [0.01_f64, 0.01, 1.01, 0.61],
        "shape" => [PointShape::Circle.as_str(); 4],
        "pattern" => [LinePattern::Solid.as_str(); 4],
    )
    .unwrap()
}

fn chart(data: &DataFrame) -> Chart<'_> {
    let mut chart = Chart::new(data)
        .group("series")
        .hue("series")
        .interval_bounds("lower", "upper")
        .interval_semantics(
            IntervalSemantics::new(
                IntervalKind::Confidence,
                0.95,
                InferentialScale::Response,
                "connector fixture",
            )
            .unwrap(),
        )
        .interaction_source(EstimateSource::EstimatedMarginalMean);
    chart
        .x("condition")
        .y("estimate")
        .size(480, 320)
        .interaction();
    chart
}

fn svg(chart: &Chart<'_>) -> String {
    String::from_utf8(chart.render_to_svg_bytes().unwrap()).unwrap()
}

#[test]
fn explicit_visible_interaction_connectors_are_byte_identical_to_default() {
    let data = data();

    assert_eq!(
        chart(&data).render_to_svg_bytes().unwrap(),
        chart(&data)
            .interaction_connectors(true)
            .render_to_svg_bytes()
            .unwrap()
    );
}

#[test]
fn hidden_interaction_connectors_omit_only_between_category_lines() {
    let data = data();
    let visible = svg(&chart(&data));
    let hidden = svg(&chart(&data).interaction_connectors(false));
    let visible_marks = clipped_marks(&visible);
    let hidden_marks = clipped_marks(&hidden);

    assert_eq!(visible_marks.matches("<polyline ").count(), 6);
    assert_eq!(hidden_marks.matches("<polyline ").count(), 4);
    assert_eq!(visible_marks.matches("<circle ").count(), 4);
    assert_eq!(hidden_marks.matches("<circle ").count(), 4);
    assert_eq!(legend(&visible), legend(&hidden));
    assert!(hidden.contains(">Before</text>") && hidden.contains(">After</text>"));
    assert!(hidden.contains(">Control</text>") && hidden.contains(">Active</text>"));
    assert_eq!(without_connector_polylines(&visible), hidden);
}

#[test]
fn hidden_interaction_connectors_reject_line_pattern_styles() {
    let data = data();
    let invalid = chart(&data)
        .interaction_styles("shape", "pattern")
        .interaction_connectors(false);

    assert!(invalid
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("incompatible with line-pattern style columns"));
}

#[test]
fn interaction_connectors_reject_non_interaction_kinds() {
    let data = data();
    let mut scatter = Chart::new(&data).interaction_connectors(false);
    scatter.x("estimate").y("upper").scatter();

    assert!(scatter
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("supported only by Interaction"));
}

#[test]
fn interaction_rejects_hiding_both_markers_and_connectors_before_begin() {
    let data = data();
    let invalid = chart(&data).markers(false).interaction_connectors(false);

    assert!(invalid
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("markers and connectors cannot both be hidden"));
    let mut backend = BeginCountingBackend::default();
    assert!(invalid.render_with_backend(&mut backend).is_err());
    assert_eq!(backend.begins, 0);
}

#[test]
fn hidden_connectors_do_not_participate_in_legend_range_planning() {
    let data = connector_collision_data();
    let visible = svg(&chart(&data));
    let mut visible_without_legend = chart(&data);
    let _ = visible_without_legend.hide_legend();
    let visible_without_legend = svg(&visible_without_legend);
    assert_ne!(
        clipped_marks(&visible),
        clipped_marks(&visible_without_legend)
    );

    let hidden = svg(&chart(&data).interaction_connectors(false));
    let mut hidden_without_legend = chart(&data).interaction_connectors(false);
    let _ = hidden_without_legend.hide_legend();
    let hidden_without_legend = svg(&hidden_without_legend);

    assert_eq!(
        clipped_marks(&hidden),
        clipped_marks(&hidden_without_legend)
    );
}

fn clipped_marks(svg: &str) -> &str {
    svg.split_once("<g clip-path=\"url(#clip-0)\">")
        .unwrap()
        .1
        .split_once("</g>")
        .unwrap()
        .0
}

fn legend(svg: &str) -> &str {
    svg.split_once("<g id=\"legend\">")
        .unwrap()
        .1
        .split_once("</g>")
        .unwrap()
        .0
}

fn without_connector_polylines(svg: &str) -> String {
    let mut output = svg.to_owned();
    for line in clipped_marks(svg).lines().filter(|line| {
        if !line.starts_with("<polyline points=\"") {
            return false;
        }
        let points = line
            .split_once("points=\"")
            .unwrap()
            .1
            .split_once('"')
            .unwrap()
            .0
            .split_whitespace()
            .map(|point| point.split_once(',').unwrap().0)
            .collect::<Vec<_>>();
        points.windows(2).any(|pair| pair[0] != pair[1])
    }) {
        output = output.replacen(&format!("{line}\n"), "", 1);
    }
    output
}
