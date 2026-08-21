use lichtung_plot::layout::PanelBox;
use lichtung_plot::render::backend::{
    Backend, BracketStyle, FillStyle, GradientStop, LineStyle, Point2D, PointStyle,
    SignificanceBracket, TextStyle,
};
use lichtung_plot::Error;
use lichtung_plot::{Chart, LegendPosition, PanelCorner, PanelTextAnnotation, Theme};
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

fn scatter_data() -> DataFrame {
    df!(
        "x" => [1.0_f64, 2.0, 3.0],
        "y" => [2.0_f64, 1.0, 3.0],
        "group" => ["a", "b", "a"],
    )
    .unwrap()
}

fn scatter(data: &DataFrame) -> Chart<'_> {
    let mut chart = Chart::new(data);
    chart.x("x").y("y").size(420, 300).scatter();
    chart
}

fn svg(chart: &Chart<'_>) -> String {
    String::from_utf8(chart.render_to_svg_bytes().unwrap()).unwrap()
}

#[test]
fn panel_text_constructor_rejects_empty_multiline_and_untrimmed_text() {
    for text in ["", "   ", " leading", "trailing ", "first\nsecond"] {
        assert!(PanelTextAnnotation::new(PanelCorner::TopLeft, text).is_err());
    }
}

#[test]
fn panel_text_renders_in_named_clipped_layer_with_theme_style_and_corner_anchors() {
    let data = scatter_data();
    let mut chart = scatter(&data)
        .panel_text_annotation(
            PanelTextAnnotation::new(PanelCorner::TopLeft, "Upper note").unwrap(),
        )
        .panel_text_annotation(
            PanelTextAnnotation::new(PanelCorner::BottomRight, "Lower note").unwrap(),
        );
    let _ = chart.theme(Theme::Nature);
    let output = svg(&chart);
    let layer = output
        .split_once("<g id=\"panel-text\">")
        .unwrap()
        .1
        .split_once("</g>")
        .unwrap()
        .0;

    assert!(layer.contains("clip-path=\"url(#clip-") && layer.contains("fill=\"#282828\""));
    let upper = layer
        .lines()
        .find(|line| line.contains("Upper note"))
        .unwrap();
    let lower = layer
        .lines()
        .find(|line| line.contains("Lower note"))
        .unwrap();
    assert!(upper.contains("text-anchor=\"start\""));
    assert!(lower.contains("text-anchor=\"end\""));
}

#[test]
fn panel_text_default_is_byte_inert() {
    let data = scatter_data();
    let first = svg(&scatter(&data));
    let second = svg(&scatter(&data));

    assert_eq!(first, second);
    assert!(!first.contains("panel-text"));
}

#[test]
fn panel_text_rejects_duplicate_corner() {
    let data = scatter_data();
    let chart = scatter(&data)
        .panel_text_annotation(PanelTextAnnotation::new(PanelCorner::TopLeft, "First").unwrap())
        .panel_text_annotation(PanelTextAnnotation::new(PanelCorner::TopLeft, "Second").unwrap());

    assert!(chart
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("duplicate panel text annotation corner"));
}

#[test]
fn panel_text_rejects_facets_and_self_managed_matrix() {
    let data = scatter_data();
    let annotation = || PanelTextAnnotation::new(PanelCorner::TopLeft, "Note").unwrap();
    let faceted = scatter(&data)
        .panel_text_annotation(annotation())
        .facet_row("group");
    let mut matrix = Chart::new(&data).panel_text_annotation(annotation());
    matrix.matrix_variables(vec!["x", "y"]).matrix();

    assert!(faceted
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("faceting"));
    assert!(matrix
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("one standard chart panel"));
}

#[test]
fn panel_text_rejects_colliding_inside_legend() {
    let data = scatter_data();
    let mut chart = Chart::new(&data).hue("group").panel_text_annotation(
        PanelTextAnnotation::new(PanelCorner::TopRight, "Caller note").unwrap(),
    );
    chart
        .x("x")
        .y("y")
        .legend_position(LegendPosition::InsideTopRight)
        .scatter();

    assert!(chart
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("incompatible with the resolved inside legend"));
}

#[test]
fn automatic_legend_falls_back_around_panel_text() {
    let data = scatter_data();
    let mut chart = Chart::new(&data).hue("group").panel_text_annotation(
        PanelTextAnnotation::new(PanelCorner::TopRight, "Caller note").unwrap(),
    );
    chart.x("x").y("y").scatter();

    let output = svg(&chart);

    assert!(output.contains(">Caller note</text>"));
    assert!(output.contains("<g id=\"legend\">"));
}

#[test]
fn separately_valid_panel_annotations_reject_pairwise_overlap_before_begin() {
    let data = scatter_data();
    let mut chart = scatter(&data)
        .panel_text_annotation(
            PanelTextAnnotation::new(
                PanelCorner::TopLeft,
                "Left annotation extends far into this panel",
            )
            .unwrap(),
        )
        .panel_text_annotation(
            PanelTextAnnotation::new(
                PanelCorner::TopRight,
                "Right annotation extends far into this panel",
            )
            .unwrap(),
        );
    let _ = chart.size(460, 300);

    let error = chart.preflight().unwrap_err().to_string();
    assert!(
        error.contains("panel text annotations at TopLeft and TopRight overlap"),
        "unexpected preflight error: {error}"
    );
    let mut backend = BeginCountingBackend::default();
    assert!(chart.render_with_backend(&mut backend).is_err());
    assert_eq!(backend.begins, 0);
}
