use lichtung_plot::layout::PanelBox;
use lichtung_plot::render::backend::{
    Backend, BracketStyle, FillStyle, GradientStop, LineStyle, Point2D, PointStyle,
    SignificanceBracket, TextStyle,
};
use lichtung_plot::Error;
use lichtung_plot::{
    CategoricalColors, Chart, Figure, FigureLegendPolicy, HistogramBinInput, HistogramBins,
    HistogramComposite, HistogramCompositeGeometryInput, HistogramDimensionsInput, HistogramKde,
    HistogramKdePoint, HistogramRugInput, HistogramStat, Labels, PlotKind, RaincloudDensityPoint,
    RaincloudDimensionsInput, RaincloudGeometryInput, RaincloudGroupInput, RaincloudPoint,
    RawOverlay, ViolinLayout,
};
use polars::prelude::{DataFrame, NamedFrom, Series};

#[derive(Debug, Clone)]
struct PolygonCall {
    points: Vec<Point2D>,
    style: FillStyle,
    clipped: bool,
}

#[derive(Debug, Clone, PartialEq)]
struct LineCall {
    points: Vec<Point2D>,
    style: LineStyle,
    clipped: bool,
}

#[derive(Debug, Clone, PartialEq)]
struct ScatterCall {
    points: Vec<Point2D>,
    style: PointStyle,
    clipped: bool,
}

#[derive(Debug, Default)]
struct RecordingBackend {
    begins: usize,
    clip_depth: usize,
    polygons: Vec<PolygonCall>,
    lines: Vec<LineCall>,
    scatters: Vec<ScatterCall>,
    texts: Vec<String>,
}

impl Backend for RecordingBackend {
    fn begin(&mut self, _width: u32, _height: u32) -> Result<(), Error> {
        self.begins += 1;
        Ok(())
    }

    fn begin_clip(&mut self, _panel: PanelBox) -> Result<(), Error> {
        self.clip_depth += 1;
        Ok(())
    }

    fn end_clip(&mut self) -> Result<(), Error> {
        self.clip_depth -= 1;
        Ok(())
    }

    fn draw_scatter(&mut self, points: &[Point2D], style: &PointStyle) -> Result<(), Error> {
        self.scatters.push(ScatterCall {
            points: points.to_vec(),
            style: style.clone(),
            clipped: self.clip_depth > 0,
        });
        Ok(())
    }

    fn draw_line(&mut self, points: &[Point2D], style: &LineStyle) -> Result<(), Error> {
        self.lines.push(LineCall {
            points: points.to_vec(),
            style: style.clone(),
            clipped: self.clip_depth > 0,
        });
        Ok(())
    }

    fn draw_polygon(&mut self, points: &[Point2D], style: &FillStyle) -> Result<(), Error> {
        self.polygons.push(PolygonCall {
            points: points.to_vec(),
            style: style.clone(),
            clipped: self.clip_depth > 0,
        });
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
        _bracket: &SignificanceBracket,
        _style: &BracketStyle,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn end(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

fn violin_data(values: &[f64]) -> DataFrame {
    DataFrame::new(
        values.len(),
        vec![
            Series::new("category".into(), vec!["a"; values.len()]).into(),
            Series::new("value".into(), values).into(),
        ],
    )
    .unwrap()
}

fn raincloud_chart(data: &DataFrame) -> Chart<'_> {
    let mut chart = Chart::new(data).violin_layout(ViolinLayout::Raincloud);
    chart.x("category").y("value").violin();
    chart
}

fn histogram_chart(data: &DataFrame, kde: HistogramKde) -> Chart<'_> {
    let mut chart = Chart::new(data).histogram_composite(HistogramComposite::KdeAndRug(kde));
    chart
        .x("value")
        .histogram_bins(HistogramBins::Count(2))
        .histogram_stat(HistogramStat::Density)
        .histogram();
    chart
}

fn explicit_raincloud() -> RaincloudGeometryInput {
    RaincloudGeometryInput {
        dimensions: RaincloudDimensionsInput {
            category: "category".into(),
            value: "value".into(),
            hue: Some("condition".into()),
        },
        groups: vec![RaincloudGroupInput {
            category: "Control".into(),
            hue: Some("Control".into()),
            density: vec![
                RaincloudDensityPoint {
                    y: 0.0,
                    normalized_width: 0.0,
                },
                RaincloudDensityPoint {
                    y: 1.0,
                    normalized_width: 1.0,
                },
                RaincloudDensityPoint {
                    y: 2.0,
                    normalized_width: 0.0,
                },
            ],
            q1: 0.5,
            median: 1.0,
            q3: 1.5,
            points: vec![
                RaincloudPoint {
                    y: 0.5,
                    normalized_strip: 0.0,
                },
                RaincloudPoint {
                    y: 1.5,
                    normalized_strip: 1.0,
                },
            ],
        }],
    }
}

fn explicit_histogram() -> HistogramCompositeGeometryInput {
    HistogramCompositeGeometryInput {
        dimensions: HistogramDimensionsInput {
            value: "value".into(),
            density: "density".into(),
        },
        bins: vec![
            HistogramBinInput {
                lower: 0.0,
                upper: 1.0,
                density: 0.4,
            },
            HistogramBinInput {
                lower: 1.0,
                upper: 2.0,
                density: 0.4,
            },
        ],
        kde: vec![
            HistogramKdePoint { x: 0.0, y: 0.1 },
            HistogramKdePoint { x: 1.0, y: 0.2 },
            HistogramKdePoint { x: 2.0, y: 0.1 },
        ],
        rugs: vec![HistogramRugInput {
            x: 1.0,
            baseline: 0.1,
            top: 0.2,
        }],
    }
}

fn parity_raincloud_data() -> DataFrame {
    DataFrame::new(
        10,
        vec![
            Series::new(
                "condition".into(),
                [
                    "control", "control", "control", "control", "control", "active", "active",
                    "active", "active", "active",
                ],
            )
            .into(),
            Series::new(
                "treatment".into(),
                [
                    "control", "control", "control", "control", "control", "active", "active",
                    "active", "active", "active",
                ],
            )
            .into(),
            Series::new(
                "score".into(),
                [1.0_f64, 2.0, 3.0, 4.0, 5.0, 2.0, 3.0, 4.0, 5.0, 6.0],
            )
            .into(),
        ],
    )
    .unwrap()
}

#[expect(
    clippy::cast_precision_loss,
    reason = "small focused test samples map their counts and hash mantissas to f64"
)]
fn parity_raincloud_geometry() -> RaincloudGeometryInput {
    let groups = [
        ("control", "control", vec![1.0, 2.0, 3.0, 4.0, 5.0]),
        ("active", "active", vec![2.0, 3.0, 4.0, 5.0, 6.0]),
    ];
    let bandwidth = |values: &[f64]| {
        let count = values.len() as f64;
        let mean = values.iter().sum::<f64>() / count;
        let variance = values
            .iter()
            .map(|value| (value - mean).powi(2))
            .sum::<f64>()
            / (count - 1.0);
        (count.powf(-0.2) * variance.sqrt()).max(1e-6)
    };
    let support_low = groups
        .iter()
        .map(|(_, _, values)| {
            values.iter().copied().fold(f64::INFINITY, f64::min) - 2.0 * bandwidth(values)
        })
        .fold(f64::INFINITY, f64::min);
    let support_high = groups
        .iter()
        .map(|(_, _, values)| {
            values.iter().copied().fold(f64::NEG_INFINITY, f64::max) + 2.0 * bandwidth(values)
        })
        .fold(f64::NEG_INFINITY, f64::max);
    let grid = (0..64)
        .map(|index| support_low + (support_high - support_low) * f64::from(index) / 63.0)
        .collect::<Vec<_>>();
    let mut source_row = 0_usize;
    let groups = groups
        .into_iter()
        .map(|(category, hue, values)| {
            let mut densities = lichtung_plot::stats::distribution::kde(&values, &grid).unwrap();
            densities[0] = 0.0;
            densities[63] = 0.0;
            let maximum = densities.iter().copied().fold(0.0_f64, f64::max);
            let density = grid
                .iter()
                .copied()
                .zip(densities)
                .map(|(y, density)| RaincloudDensityPoint {
                    y,
                    normalized_width: density / maximum,
                })
                .collect();
            let points = values
                .iter()
                .copied()
                .map(|y| {
                    let point = RaincloudPoint {
                        y,
                        normalized_strip: source_row_unit(source_row),
                    };
                    source_row += 1;
                    point
                })
                .collect();
            RaincloudGroupInput {
                category: category.into(),
                hue: Some(hue.into()),
                density,
                q1: lichtung_plot::stats::distribution::quantile(&values, 0.25).unwrap(),
                median: lichtung_plot::stats::distribution::quantile(&values, 0.5).unwrap(),
                q3: lichtung_plot::stats::distribution::quantile(&values, 0.75).unwrap(),
                points,
            }
        })
        .collect();
    RaincloudGeometryInput {
        dimensions: RaincloudDimensionsInput {
            category: "condition".into(),
            value: "score".into(),
            hue: Some("treatment".into()),
        },
        groups,
    }
}

#[expect(
    clippy::cast_precision_loss,
    reason = "the upper 53 hash bits intentionally map onto an f64 mantissa"
)]
fn source_row_unit(source_row: usize) -> f64 {
    let mut value = u64::try_from(source_row)
        .unwrap()
        .wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^= value >> 31;
    ((value >> 11) as f64) * (1.0 / ((1_u64 << 53) as f64))
}

fn assert_recorded_marks_equal(actual: &RecordingBackend, expected: &RecordingBackend) {
    assert_eq!(actual.lines, expected.lines);
    assert_eq!(actual.scatters, expected.scatters);
    assert_eq!(actual.texts, expected.texts);
    assert_eq!(actual.polygons.len(), expected.polygons.len());
    for (actual, expected) in actual.polygons.iter().zip(&expected.polygons) {
        assert_eq!(actual.points, expected.points);
        assert_eq!(actual.clipped, expected.clipped);
        assert_eq!(actual.style.color, expected.style.color);
        assert_eq!(actual.style.stroke, expected.style.stroke);
        assert_eq!(
            actual.style.stroke_width.to_bits(),
            expected.style.stroke_width.to_bits()
        );
        assert_eq!(
            actual.style.opacity.to_bits(),
            expected.style.opacity.to_bits()
        );
    }
}

#[test]
fn supplied_raincloud_geometry_matches_sample_labels_colors_legend_and_recorded_marks() {
    let data = parity_raincloud_data();
    let mut sample = Chart::new(&data)
        .hue("treatment")
        .violin_layout(ViolinLayout::Raincloud);
    sample.x("condition").y("score").violin();
    let empty = DataFrame::empty();
    let mut supplied = Chart::new(&empty).raincloud_geometry(parity_raincloud_geometry());
    supplied.violin();
    let mut sample_backend = RecordingBackend::default();
    let mut supplied_backend = RecordingBackend::default();

    sample.render_with_backend(&mut sample_backend).unwrap();
    supplied.render_with_backend(&mut supplied_backend).unwrap();

    assert_recorded_marks_equal(&supplied_backend, &sample_backend);
    assert!(sample_backend.texts.iter().any(|text| text == "Condition"));
    assert!(sample_backend.texts.iter().any(|text| text == "Score"));
    assert!(sample_backend.texts.iter().any(|text| text == "Treatment"));
    assert!(sample_backend.texts.iter().any(|text| text == "Control"));
    assert!(sample_backend.texts.iter().any(|text| text == "Active"));
    assert_eq!(
        supplied.render_to_svg_bytes().unwrap(),
        sample.render_to_svg_bytes().unwrap()
    );
}

#[test]
fn supplied_histogram_geometry_matches_sample_labels_and_recorded_marks() {
    let values = [0.0_f64, 0.5, 1.5, 2.0];
    let data = DataFrame::new(
        values.len(),
        vec![Series::new("value".into(), values).into()],
    )
    .unwrap();
    let kde = vec![(0.0, 0.5), (2.0, 0.5)];
    let sample = histogram_chart(&data, HistogramKde::Precomputed(kde));
    let rug_top = 0.5 * 0.035;
    let input = HistogramCompositeGeometryInput {
        dimensions: HistogramDimensionsInput {
            value: "value".into(),
            density: "density".into(),
        },
        bins: vec![
            HistogramBinInput {
                lower: 0.0,
                upper: 1.0,
                density: 0.5,
            },
            HistogramBinInput {
                lower: 1.0,
                upper: 2.0,
                density: 0.5,
            },
        ],
        kde: vec![
            HistogramKdePoint { x: 0.0, y: 0.5 },
            HistogramKdePoint { x: 2.0, y: 0.5 },
        ],
        rugs: values
            .into_iter()
            .map(|x| HistogramRugInput {
                x,
                baseline: 0.0,
                top: rug_top,
            })
            .collect(),
    };
    let empty = DataFrame::empty();
    let mut supplied = Chart::new(&empty).histogram_composite_geometry(input);
    supplied.histogram();
    let mut sample_backend = RecordingBackend::default();
    let mut supplied_backend = RecordingBackend::default();

    sample.render_with_backend(&mut sample_backend).unwrap();
    supplied.render_with_backend(&mut supplied_backend).unwrap();

    assert_recorded_marks_equal(&supplied_backend, &sample_backend);
    assert!(sample_backend.texts.iter().any(|text| text == "Value"));
    assert!(sample_backend.texts.iter().any(|text| text == "Density"));
    assert_eq!(
        supplied.render_to_svg_bytes().unwrap(),
        sample.render_to_svg_bytes().unwrap()
    );
}

#[test]
fn embedded_raincloud_hues_coordinate_with_sample_hues_in_shared_figure_legend() {
    let data = DataFrame::new(
        5,
        vec![
            Series::new("condition".into(), ["control"; 5]).into(),
            Series::new("treatment".into(), ["control"; 5]).into(),
            Series::new("score".into(), [1.0_f64, 2.0, 3.0, 4.0, 5.0]).into(),
        ],
    )
    .unwrap();
    let colors = CategoricalColors::new([
        ("control".to_owned(), (51, 51, 51)),
        ("active".to_owned(), (0, 114, 178)),
    ])
    .unwrap();
    let mut sample = Chart::new(&data)
        .hue("treatment")
        .categorical_colors(colors.clone())
        .violin_layout(ViolinLayout::Raincloud);
    sample.x("condition").y("score").violin();
    let mut input = parity_raincloud_geometry();
    input.groups.remove(0);
    let empty = DataFrame::empty();
    let mut supplied = Chart::new(&empty)
        .categorical_colors(colors)
        .raincloud_geometry(input);
    supplied.violin();
    let figure = Figure::grid(1, 2)
        .unwrap()
        .panel(0, 0, sample)
        .unwrap()
        .panel(0, 1, supplied)
        .unwrap()
        .labels(Labels::humanized().alias("treatment", "Study arm"))
        .legend_policy(FigureLegendPolicy::Shared);

    let mut backend = RecordingBackend::default();
    figure.render_with_backend(&mut backend).unwrap();
    let svg = String::from_utf8(figure.render_to_svg_bytes().unwrap()).unwrap();

    assert_eq!(svg.matches("id=\"figure-shared-legend\"").count(), 1);
    assert!(svg.contains(">Study arm</text>"));
    assert!(svg.contains(">Control</text>") && svg.contains(">Active</text>"));
    assert!(backend
        .scatters
        .iter()
        .any(|call| call.style.color == (51, 51, 51)));
    assert!(backend
        .scatters
        .iter()
        .any(|call| call.style.color == (0, 114, 178)));
}

#[test]
fn precomputed_raincloud_accepts_repeated_categories_with_distinct_hues() {
    let data = DataFrame::empty();
    let mut input = parity_raincloud_geometry();
    input.groups[1].category = input.groups[0].category.clone();
    let mut chart = Chart::new(&data).raincloud_geometry(input);
    chart.violin();
    let mut backend = RecordingBackend::default();

    chart.render_with_backend(&mut backend).unwrap();

    assert_eq!(backend.begins, 1);
}

#[test]
fn precomputed_raincloud_rejects_duplicate_category_hue_identities_before_backend_begin() {
    let data = DataFrame::empty();
    let mut input = parity_raincloud_geometry();
    input.groups[1].category = input.groups[0].category.clone();
    input.groups[1].hue = input.groups[0].hue.clone();
    let mut chart = Chart::new(&data).raincloud_geometry(input);
    chart.violin();
    let mut backend = RecordingBackend::default();

    let error = chart.render_with_backend(&mut backend).unwrap_err();

    assert!(
        error
            .to_string()
            .contains("duplicate category-hue identity")
            && backend.begins == 0
    );
}

#[test]
fn precomputed_histogram_accepts_signed_zero_contiguous_bin_edges() {
    let data = DataFrame::empty();
    let mut input = explicit_histogram();
    input.bins[0].lower = -1.0;
    input.bins[0].upper = -0.0;
    input.bins[1].lower = 0.0;
    input.bins[1].upper = 1.0;
    input.kde = vec![
        HistogramKdePoint { x: -1.0, y: 0.1 },
        HistogramKdePoint { x: 1.0, y: 0.1 },
    ];
    input.rugs[0].x = 0.0;
    let mut chart = Chart::new(&data).histogram_composite_geometry(input);
    chart.histogram();

    chart
        .render_with_backend(&mut RecordingBackend::default())
        .unwrap();
}

#[test]
fn precomputed_histogram_rejects_kde_over_safety_budget_before_backend_begin() {
    let data = DataFrame::empty();
    let mut input = explicit_histogram();
    input.kde = vec![HistogramKdePoint { x: 0.5, y: 0.1 }; 2_049];
    let mut chart = Chart::new(&data).histogram_composite_geometry(input);
    chart.histogram();
    let mut backend = RecordingBackend::default();

    let error = chart.render_with_backend(&mut backend).unwrap_err();

    assert!(error.to_string().contains("safety limit of 2048") && backend.begins == 0);
}

#[test]
fn precomputed_raincloud_renders_without_sample_columns_and_maps_strip_coordinates() {
    let data = DataFrame::empty();
    let mut chart = Chart::new(&data).raincloud_geometry(explicit_raincloud());
    chart.violin();
    let mut backend = RecordingBackend::default();

    chart.render_with_backend(&mut backend).unwrap();

    let strip = backend
        .scatters
        .iter()
        .find(|call| call.clipped && call.style.radius == 2)
        .unwrap();
    assert!(strip.points.len() == 2 && strip.points[0].x < strip.points[1].x);
}

#[test]
fn precomputed_histogram_renders_without_sample_columns_and_maps_explicit_rug_endpoints() {
    let data = DataFrame::empty();
    let mut chart = Chart::new(&data).histogram_composite_geometry(explicit_histogram());
    chart.histogram();
    let mut backend = RecordingBackend::default();

    chart.render_with_backend(&mut backend).unwrap();

    let curve = backend
        .lines
        .iter()
        .find(|call| call.clipped && call.style.width.to_bits() == 1.0_f32.to_bits())
        .unwrap();
    let rug = backend
        .lines
        .iter()
        .find(|call| call.clipped && call.style.width.to_bits() == 0.7_f32.to_bits())
        .unwrap();
    assert_eq!(
        rug.points
            .iter()
            .map(|point| point.y.to_bits())
            .collect::<Vec<_>>(),
        [curve.points[0].y.to_bits(), curve.points[1].y.to_bits()]
    );
}

#[test]
fn precomputed_raincloud_rejects_out_of_range_strip_coordinate_before_backend_begin() {
    let data = DataFrame::empty();
    let mut input = explicit_raincloud();
    input.groups[0].points[0].normalized_strip = 1.1;
    let mut chart = Chart::new(&data).raincloud_geometry(input);
    chart.violin();
    let mut backend = RecordingBackend::default();

    let error = chart.render_with_backend(&mut backend).unwrap_err();

    assert!(error.to_string().contains("normalized strip coordinate") && backend.begins == 0);
}

#[test]
fn precomputed_raincloud_rejects_nonincreasing_density_before_backend_begin() {
    let data = DataFrame::empty();
    let mut input = explicit_raincloud();
    input.groups[0].density[1].y = 0.0;
    let mut chart = Chart::new(&data).raincloud_geometry(input);
    chart.violin();
    let mut backend = RecordingBackend::default();

    let error = chart.render_with_backend(&mut backend).unwrap_err();

    assert!(error.to_string().contains("strictly increasing") && backend.begins == 0);
}

#[test]
fn precomputed_histogram_rejects_nonfinite_geometry_before_backend_begin() {
    let data = DataFrame::empty();
    let mut input = explicit_histogram();
    input.kde[0].y = f64::NAN;
    let mut chart = Chart::new(&data).histogram_composite_geometry(input);
    chart.histogram();
    let mut backend = RecordingBackend::default();

    let error = chart.render_with_backend(&mut backend).unwrap_err();

    assert!(error.to_string().contains("nonnegative finite y") && backend.begins == 0);
}

#[test]
fn precomputed_histogram_rejects_discontinuous_bins_before_backend_begin() {
    let data = DataFrame::empty();
    let mut input = explicit_histogram();
    input.bins[1].lower = 1.1;
    let mut chart = Chart::new(&data).histogram_composite_geometry(input);
    chart.histogram();
    let mut backend = RecordingBackend::default();

    let error = chart.render_with_backend(&mut backend).unwrap_err();

    assert!(error.to_string().contains("ordered and contiguous") && backend.begins == 0);
}

#[test]
fn raincloud_cloud_box_and_strip_have_separated_geometry() {
    let data = violin_data(&[1.0, 2.0, 3.0, 4.0, 5.0]);
    let mut backend = RecordingBackend::default();
    raincloud_chart(&data)
        .render_with_backend(&mut backend)
        .unwrap();
    let marks = backend
        .polygons
        .iter()
        .filter(|call| call.clipped)
        .collect::<Vec<_>>();
    let cloud_right = marks[0]
        .points
        .iter()
        .map(|point| point.x)
        .fold(f64::NEG_INFINITY, f64::max);
    let box_left = marks[1]
        .points
        .iter()
        .map(|point| point.x)
        .fold(f64::INFINITY, f64::min);
    let box_right = marks[1]
        .points
        .iter()
        .map(|point| point.x)
        .fold(f64::NEG_INFINITY, f64::max);
    let strip = backend
        .scatters
        .iter()
        .find(|call| call.clipped && call.style.radius == 2)
        .unwrap();

    assert!(
        cloud_right < box_left
            && strip
                .points
                .iter()
                .all(|point| point.x - f64::from(strip.style.radius) > box_right)
    );
}

#[test]
fn raincloud_strip_retains_every_source_observation_without_sampling() {
    let data = violin_data(&[1.0, 1.0, 2.0, 2.0, 3.0, 3.0, 4.0]);
    let mut backend = RecordingBackend::default();
    raincloud_chart(&data)
        .render_with_backend(&mut backend)
        .unwrap();

    assert_eq!(backend.scatters[0].points.len(), data.height());
}

#[test]
fn raincloud_uses_exact_categorical_colors_for_bodies_points_and_legend() {
    let data = DataFrame::new(
        8,
        vec![
            Series::new("category".into(), ["a", "a", "a", "a", "b", "b", "b", "b"]).into(),
            Series::new(
                "condition".into(),
                [
                    "Control",
                    "Control",
                    "Control",
                    "Control",
                    "Semantic Agent",
                    "Semantic Agent",
                    "Semantic Agent",
                    "Semantic Agent",
                ],
            )
            .into(),
            Series::new("value".into(), [1.0, 2.0, 3.0, 4.0, 2.0, 3.0, 4.0, 5.0]).into(),
        ],
    )
    .unwrap();
    let colors = CategoricalColors::new([
        ("Control".to_owned(), (51, 51, 51)),
        ("Semantic Agent".to_owned(), (0, 114, 178)),
    ])
    .unwrap();
    let mut chart = Chart::new(&data)
        .hue("condition")
        .categorical_colors(colors)
        .violin_layout(ViolinLayout::Raincloud);
    chart.x("category").y("value").violin();
    let mut backend = RecordingBackend::default();

    chart.render_with_backend(&mut backend).unwrap();

    assert_eq!(
        backend
            .polygons
            .iter()
            .filter(|call| matches!(call.style.color, (51, 51, 51) | (0, 114, 178)))
            .map(|call| call.style.color)
            .collect::<Vec<_>>(),
        [(51, 51, 51), (0, 114, 178)]
    );
    assert_eq!(
        backend
            .scatters
            .iter()
            .filter(|call| call.clipped)
            .map(|call| call.style.color)
            .collect::<Vec<_>>(),
        [(51, 51, 51), (0, 114, 178)]
    );
}

#[test]
fn raincloud_strip_placement_follows_explicit_source_rows_after_reordering() {
    let first = violin_data(&[1.0, 2.0, 3.0]);
    let reordered = violin_data(&[3.0, 1.0, 2.0]);
    let first_rows = [10, 20, 30];
    let reordered_rows = [30, 10, 20];
    let mut first_backend = RecordingBackend::default();
    let mut reordered_backend = RecordingBackend::default();
    let mut first_chart = Chart::new(&first)
        .source_rows(&first_rows)
        .violin_layout(ViolinLayout::Raincloud);
    first_chart.x("category").y("value").violin();
    first_chart.render_with_backend(&mut first_backend).unwrap();
    let mut reordered_chart = Chart::new(&reordered)
        .source_rows(&reordered_rows)
        .violin_layout(ViolinLayout::Raincloud);
    reordered_chart.x("category").y("value").violin();
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
fn histogram_rug_retains_every_source_observation_including_duplicates() {
    let data = DataFrame::new(
        5,
        vec![Series::new("value".into(), [0.0_f64, 0.0, 1.0, 1.0, 2.0]).into()],
    )
    .unwrap();
    let mut backend = RecordingBackend::default();
    histogram_chart(&data, HistogramKde::Internal)
        .render_with_backend(&mut backend)
        .unwrap();
    let rugs = backend
        .lines
        .iter()
        .filter(|call| call.clipped && call.style.width.to_bits() == 0.7_f32.to_bits())
        .count();

    assert_eq!(rugs, data.height());
}

#[test]
fn precomputed_kde_and_histogram_bars_share_density_projection() {
    let data = DataFrame::new(
        4,
        vec![Series::new("value".into(), [0.0_f64, 0.5, 1.5, 2.0]).into()],
    )
    .unwrap();
    let mut backend = RecordingBackend::default();
    histogram_chart(
        &data,
        HistogramKde::Precomputed(vec![(0.0, 0.5), (2.0, 0.5)]),
    )
    .render_with_backend(&mut backend)
    .unwrap();
    let bar_tops = backend
        .polygons
        .iter()
        .filter(|call| call.clipped && call.style.opacity.to_bits() == 0.82_f64.to_bits())
        .map(|call| call.points[2].y.to_bits())
        .collect::<Vec<_>>();
    let curve = backend
        .lines
        .iter()
        .find(|call| call.clipped && call.style.width.to_bits() == 1.0_f32.to_bits())
        .unwrap();

    assert_eq!(
        (
            bar_tops,
            curve
                .points
                .iter()
                .map(|point| point.y.to_bits())
                .collect::<Vec<_>>()
        ),
        (
            vec![curve.points[0].y.to_bits(); 2],
            vec![curve.points[0].y.to_bits(); 2]
        )
    );
}

#[test]
fn histogram_composite_requires_density_before_backend_begin() {
    let data = DataFrame::new(
        3,
        vec![Series::new("value".into(), [0.0_f64, 1.0, 2.0]).into()],
    )
    .unwrap();
    let mut chart = Chart::new(&data)
        .histogram_composite(HistogramComposite::KdeAndRug(HistogramKde::Internal));
    chart.x("value").histogram();
    let mut backend = RecordingBackend::default();

    let error = chart.render_with_backend(&mut backend).unwrap_err();

    assert!(
        error
            .to_string()
            .contains("requires HistogramStat::Density")
            && backend.begins == 0
    );
}

#[test]
fn histogram_composite_is_rejected_by_other_plot_kinds() {
    let data = DataFrame::new(
        3,
        vec![
            Series::new("value".into(), [0.0_f64, 1.0, 2.0]).into(),
            Series::new("other".into(), [2.0_f64, 1.0, 0.0]).into(),
        ],
    )
    .unwrap();
    let mut chart = Chart::new(&data)
        .histogram_composite(HistogramComposite::KdeAndRug(HistogramKde::Internal));
    chart.x("value").y("other").scatter();
    let mut backend = RecordingBackend::default();

    let error = chart.render_with_backend(&mut backend).unwrap_err();

    assert!(error.to_string().contains("supported only by Histogram") && backend.begins == 0);
}

#[test]
fn precomputed_kde_rejects_nonincreasing_points_before_backend_begin() {
    let data = DataFrame::new(
        3,
        vec![Series::new("value".into(), [0.0_f64, 1.0, 2.0]).into()],
    )
    .unwrap();
    let mut backend = RecordingBackend::default();

    let error = histogram_chart(
        &data,
        HistogramKde::Precomputed(vec![(1.0, 0.2), (0.5, 0.3)]),
    )
    .render_with_backend(&mut backend)
    .unwrap_err();

    assert!(error.to_string().contains("strictly increasing") && backend.begins == 0);
}

#[test]
fn raincloud_rejects_duplicate_raw_overlay_before_backend_begin() {
    let data = violin_data(&[1.0, 2.0, 3.0]);
    let mut chart = Chart::new(&data)
        .violin_layout(ViolinLayout::Raincloud)
        .raw_overlay(RawOverlay::Jittered);
    chart.x("category").y("value").violin();
    let mut backend = RecordingBackend::default();

    let error = chart.render_with_backend(&mut backend).unwrap_err();

    assert!(
        error
            .to_string()
            .contains("already includes every raw observation")
            && backend.begins == 0
    );
}

#[test]
fn raincloud_rejects_duplicate_source_row_identities_before_backend_begin() {
    let data = violin_data(&[1.0, 2.0, 3.0]);
    let source_rows = [4, 4, 8];
    let mut chart = Chart::new(&data)
        .source_rows(&source_rows)
        .violin_layout(ViolinLayout::Raincloud);
    chart.x("category").y("value").violin();
    let mut backend = RecordingBackend::default();

    let error = chart.render_with_backend(&mut backend).unwrap_err();

    assert!(
        error
            .to_string()
            .contains("duplicate source-row identity 4")
            && backend.begins == 0
    );
}

#[test]
fn raincloud_accepts_source_row_column_without_duplicate_overlay() {
    let data = DataFrame::new(
        3,
        vec![
            Series::new("category".into(), ["a", "a", "a"]).into(),
            Series::new("value".into(), [1.0_f64, 2.0, 3.0]).into(),
            Series::new("source_row".into(), [9_u64, 3, 7]).into(),
        ],
    )
    .unwrap();
    let mut chart = Chart::new(&data)
        .raw_overlay_source_row("source_row")
        .violin_layout(ViolinLayout::Raincloud);
    chart.x("category").y("value").violin();
    let mut backend = RecordingBackend::default();

    chart.render_with_backend(&mut backend).unwrap();

    assert_eq!(backend.scatters[0].points.len(), 3);
}

#[test]
fn explicit_inert_composite_options_match_default_svg_bytes() {
    let histogram_data = DataFrame::new(
        4,
        vec![Series::new("value".into(), [0.0_f64, 1.0, 2.0, 3.0]).into()],
    )
    .unwrap();
    let mut histogram_default = Chart::new(&histogram_data);
    histogram_default.x("value").histogram();
    let mut histogram_explicit =
        Chart::new(&histogram_data).histogram_composite(HistogramComposite::HistogramOnly);
    histogram_explicit.x("value").histogram();

    let violin_data = violin_data(&[1.0, 2.0, 3.0, 4.0]);
    let mut violin_default = Chart::new(&violin_data);
    violin_default.x("category").y("value").violin();
    let mut violin_explicit = Chart::new(&violin_data).violin_layout(ViolinLayout::Mirrored);
    violin_explicit.x("category").y("value").violin();

    assert_eq!(
        (
            histogram_default.render_to_svg_bytes().unwrap(),
            violin_default.render_to_svg_bytes().unwrap(),
        ),
        (
            histogram_explicit.render_to_svg_bytes().unwrap(),
            violin_explicit.render_to_svg_bytes().unwrap(),
        )
    );
}

#[test]
fn raincloud_layout_is_rejected_by_other_plot_kinds() {
    let data = violin_data(&[1.0, 2.0, 3.0]);
    let mut chart = Chart::new(&data).violin_layout(ViolinLayout::Raincloud);
    chart.x("category").y("value").select(PlotKind::Boxplot);
    let mut backend = RecordingBackend::default();

    let error = chart.render_with_backend(&mut backend).unwrap_err();

    assert!(error.to_string().contains("supported only by Violin") && backend.begins == 0);
}
