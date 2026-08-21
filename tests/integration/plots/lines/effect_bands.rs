use lichtung_plot::layout::PanelBox;
use lichtung_plot::render::backend::{
    Backend, BracketStyle, FillStyle, GradientStop, LineStyle, Point2D, PointStyle,
    SignificanceBracket, TextStyle,
};
use lichtung_plot::Error;
use lichtung_plot::{
    Chart, EffectBandSemantics, InferentialScale, IntervalKind, IntervalSemantics, LineMode,
};
use polars::prelude::{DataFrame, NamedFrom, Series};

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
    DataFrame::new(
        3,
        vec![
            Series::new("x".into(), [2.0_f64, 0.0, 1.0]).into(),
            Series::new("estimate".into(), [6.0_f64, 2.0, 4.0]).into(),
            Series::new("lower".into(), [5.0_f64, 1.0, 3.0]).into(),
            Series::new("upper".into(), [7.0_f64, 3.0, 5.0]).into(),
        ],
    )
    .unwrap()
}

fn interval() -> IntervalSemantics {
    IntervalSemantics::new(
        IntervalKind::Confidence,
        0.95,
        InferentialScale::Response,
        "precomputed projected covariance",
    )
    .unwrap()
}

fn pointwise() -> EffectBandSemantics {
    EffectBandSemantics::pointwise(
        interval(),
        "ordinary Gaussian model fit",
        "age grid with adjusted nuisance values",
    )
    .unwrap()
}

#[test]
fn pointwise_and_external_simultaneous_bands_use_exact_precomputed_line_geometry() {
    let data = data();
    let directory = std::env::temp_dir().join(format!(
        "lichtung-effect-curve-render-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&directory).unwrap();
    let pointwise_path = directory.join("pointwise.svg");
    let simultaneous_path = directory.join("simultaneous.svg");

    let mut pointwise_chart = Chart::new(&data)
        .interval_bounds("lower", "upper")
        .effect_band_semantics(pointwise());
    pointwise_chart.x("x").y("estimate").line();
    pointwise_chart.render_to_svg(&pointwise_path).unwrap();

    let simultaneous = EffectBandSemantics::simultaneous(
        interval(),
        "Scheffe projection",
        "all declared x coordinates",
        "external Gaussian model fit",
        "external adjusted grid",
    )
    .unwrap();
    let mut simultaneous_chart = Chart::new(&data)
        .interval_bounds("lower", "upper")
        .effect_band_semantics(simultaneous);
    simultaneous_chart.x("x").y("estimate").line();
    simultaneous_chart
        .render_to_svg(&simultaneous_path)
        .unwrap();

    let pointwise_svg = std::fs::read_to_string(pointwise_path).unwrap();
    let simultaneous_svg = std::fs::read_to_string(simultaneous_path).unwrap();
    assert_eq!(pointwise_svg.matches("fill-opacity=\"0.18\"").count(), 1);
    assert_eq!(pointwise_svg.matches("stroke-width=\"1.8\"").count(), 1);
    assert!(!pointwise_svg.contains("ordinary Gaussian model fit"));
    assert!(!pointwise_svg.contains("age grid with adjusted nuisance values"));
    assert!(!simultaneous_svg.contains("Scheffe projection"));
    assert!(!simultaneous_svg.contains("all declared x coordinates"));
    assert!(!simultaneous_svg.contains("external Gaussian model fit"));
    assert!(!simultaneous_svg.contains("external adjusted grid"));
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn invalid_effect_band_configurations_fail_before_backend_begin() {
    let data = data();
    let cases = [
        {
            let mut chart = Chart::new(&data)
                .interval_bounds("lower", "upper")
                .effect_band_semantics(pointwise());
            chart.x("x").y("estimate").scatter();
            chart
        },
        {
            let mut chart = Chart::new(&data).effect_band_semantics(pointwise());
            chart.x("x").y("estimate").line();
            chart
        },
        {
            let mut chart = Chart::new(&data)
                .group("estimate")
                .interval_bounds("lower", "upper")
                .effect_band_semantics(pointwise())
                .line_mode(LineMode::Trajectory);
            chart.x("x").y("estimate").line();
            chart
        },
    ];

    for chart in cases {
        let mut backend = BeginCountingBackend::default();
        assert!(chart.render_with_backend(&mut backend).is_err());
        assert_eq!(backend.begins, 0);
    }
}

#[test]
fn generic_line_with_ribbon_keeps_generic_semantics() {
    let data = data();
    let directory = std::env::temp_dir().join(format!(
        "lichtung-generic-line-render-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("line.svg");
    let mut chart = Chart::new(&data).interval_bounds("lower", "upper");
    chart.x("x").y("estimate").line();

    chart.render_to_svg(&path).unwrap();

    let svg = std::fs::read_to_string(path).unwrap();
    assert!(!svg.contains("model-adjusted effect"));
    assert!(!svg.contains("pointwise"));
    std::fs::remove_dir_all(directory).unwrap();
}
