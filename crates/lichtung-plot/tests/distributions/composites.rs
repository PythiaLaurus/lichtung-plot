use lichtung_plot::{
    Canvas, CanvasLegendPolicy, CategoricalColorMap, CategoricalColorScale, CategoryValues,
    FacetSpec, HistogramBins, HistogramComposite, HistogramGeometry, HistogramInput, HistogramKde,
    HistogramStat, Labels, NumericValues, OpaqueColor, Plot, RaincloudGeometry, RaincloudInput,
    Significance, SignificanceMark, SourceRows,
};

use super::{parse_circle_bounds, parse_polygon_bounds, unique_dir};

fn values(values: &[f64]) -> (lichtung_plot::CategoryValues, lichtung_plot::NumericValues) {
    (
        lichtung_plot::CategoryValues::new("category", vec!["a"; values.len()]).unwrap(),
        lichtung_plot::NumericValues::new("value", values.iter().copied()),
    )
}

fn raincloud_plot(samples: &[f64]) -> Plot<lichtung_plot::kind::Raincloud> {
    let (category, values) = values(samples);
    Plot::raincloud(RaincloudInput::samples(category, values))
}

fn histogram_plot(values: &[f64], kde: HistogramKde) -> Plot<lichtung_plot::kind::Histogram> {
    Plot::histogram(HistogramInput::samples(lichtung_plot::NumericValues::new(
        "value",
        values.iter().copied(),
    )))
    .composite(HistogramComposite::KdeAndRug(kde))
    .bins(HistogramBins::Count(2))
    .stat(HistogramStat::Density)
}

fn render<K: lichtung_plot::kind::Marker>(
    plot: impl std::borrow::Borrow<Plot<K>>,
    name: &str,
) -> Result<String, lichtung_plot::Error> {
    let directory = unique_dir(name);
    let output = directory.join("plot.svg");
    let result = plot
        .borrow()
        .svg_preserve_text(&output)
        .and_then(|_| std::fs::read_to_string(&output).map_err(lichtung_plot::Error::Io));
    std::fs::remove_dir_all(directory).unwrap();
    result
}

fn explicit_raincloud() -> RaincloudGeometry {
    raincloud_geometry(&["Control"], &["Control"])
}

fn raincloud_geometry(category: &[&str], hue: &[&str]) -> RaincloudGeometry {
    RaincloudGeometry {
        category_dimension: "category".into(),
        value_dimension: "value".into(),
        hue_dimension: Some("condition".into()),
        category: CategoryValues::new("category", category.iter().copied()).unwrap(),
        hue: Some(CategoryValues::new("condition", hue.iter().copied()).unwrap()),
        q1: NumericValues::new("q1", vec![0.5; category.len()]),
        median: NumericValues::new("median", vec![1.0; category.len()]),
        q3: NumericValues::new("q3", vec![1.5; category.len()]),
        density_category: CategoryValues::new(
            "category",
            category
                .iter()
                .flat_map(|value| std::iter::repeat_n(*value, 3)),
        )
        .unwrap(),
        density_hue: Some(
            CategoryValues::new(
                "condition",
                hue.iter().flat_map(|value| std::iter::repeat_n(*value, 3)),
            )
            .unwrap(),
        ),
        density_y: NumericValues::new("y", category.iter().flat_map(|_| [0.0, 1.0, 2.0])),
        normalized_width: NumericValues::new(
            "width",
            category.iter().flat_map(|_| [0.0, 1.0, 0.0]),
        ),
        point_category: CategoryValues::new(
            "category",
            category
                .iter()
                .flat_map(|value| std::iter::repeat_n(*value, 2)),
        )
        .unwrap(),
        point_hue: Some(
            CategoryValues::new(
                "condition",
                hue.iter().flat_map(|value| std::iter::repeat_n(*value, 2)),
            )
            .unwrap(),
        ),
        point_y: NumericValues::new("y", category.iter().flat_map(|_| [0.5, 1.5])),
        normalized_strip: NumericValues::new("strip", category.iter().flat_map(|_| [0.0, 1.0])),
    }
}

fn explicit_histogram() -> HistogramGeometry {
    HistogramGeometry {
        value_dimension: "value".into(),
        density_dimension: "density".into(),
        bin_lower: NumericValues::new("lower", [0.0, 1.0]),
        bin_upper: NumericValues::new("upper", [1.0, 2.0]),
        bin_density: NumericValues::new("density", [0.4, 0.4]),
        kde_x: NumericValues::new("x", [0.0, 1.0, 2.0]),
        kde_y: NumericValues::new("density", [0.1, 0.2, 0.1]),
        rug_x: NumericValues::new("x", [1.0]),
        rug_baseline: NumericValues::new("baseline", [0.1]),
        rug_top: NumericValues::new("top", [0.2]),
    }
}

fn hued_raincloud_geometry(categories: &[&str]) -> RaincloudGeometry {
    let mut geometry = raincloud_geometry(categories, categories);
    geometry.category_dimension = "condition".into();
    geometry.value_dimension = "score".into();
    geometry.hue_dimension = Some("treatment".into());
    geometry
}

#[test]
fn precomputed_raincloud_rejects_pairwise_significance_at_runtime() {
    let error = Plot::raincloud(RaincloudInput::geometry(explicit_raincloud()))
        .significance([Significance::between(
            "Control",
            "Other",
            SignificanceMark::OneStar,
        )])
        .preflight()
        .unwrap_err();

    assert!(error
        .to_string()
        .contains("precomputed geometry does not support significance annotations"));
}

#[test]
fn precomputed_raincloud_series_type_checks_but_preflight_rejects_it() {
    let error = Plot::raincloud(RaincloudInput::geometry(explicit_raincloud()))
        .series(lichtung_plot::CategoryValues::new("series", ["configured"; 2]).unwrap())
        .preflight()
        .unwrap_err();

    assert_eq!(
        error.to_string(),
        "data error: raincloud: Plot::series requires RaincloudInput::samples; remove .series(...) or construct the raincloud from samples"
    );
}

#[test]
fn supplied_histogram_geometry_matches_sample_labels_and_recorded_marks() {
    let format = lichtung_plot::NumericFormat::default()
        .decimal_scale(2)
        .unwrap()
        .suffix("%")
        .unwrap();
    let values = [0.0_f64, 0.5, 1.5, 2.0];
    let sample = histogram_plot(
        &values,
        HistogramKde::Precomputed {
            x: NumericValues::new("x", [0.0, 2.0]),
            y: NumericValues::new("density", [0.5, 0.5]),
        },
    );
    let rug_top = 0.5 * 0.035;
    let mut supplied = explicit_histogram();
    supplied.bin_density = NumericValues::new("density", [0.5, 0.5]);
    supplied.kde_x = NumericValues::new("x", [0.0, 2.0]);
    supplied.kde_y = NumericValues::new("density", [0.5, 0.5]);
    supplied.rug_x = NumericValues::new("x", values);
    supplied.rug_baseline = NumericValues::new("baseline", [0.0; 4]);
    supplied.rug_top = NumericValues::new("top", [rug_top; 4]);
    let rendered = render(
        Plot::histogram(HistogramInput::geometry(supplied))
            .x_tick_format(format.clone())
            .y_tick_format(format.clone()),
        "supplied-histogram",
    )
    .unwrap();
    assert!(rendered.contains("%</text>"));
    assert_eq!(
        rendered,
        render(
            sample.x_tick_format(format.clone()).y_tick_format(format),
            "sample-histogram"
        )
        .unwrap()
    );
}

#[test]
fn embedded_raincloud_hues_coordinate_with_sample_series_in_shared_canvas_legend() {
    let colors = CategoricalColorMap::new([
        ("control".to_owned(), OpaqueColor::rgb(51, 51, 51)),
        ("active".to_owned(), OpaqueColor::rgb(0, 114, 178)),
    ])
    .unwrap();
    let sample = Plot::raincloud(RaincloudInput::samples(
        lichtung_plot::CategoryValues::new("condition", ["control"; 5]).unwrap(),
        lichtung_plot::NumericValues::new("score", [1.0, 2.0, 3.0, 4.0, 5.0]),
    ))
    .series(lichtung_plot::CategoryValues::new("treatment", ["control"; 5]).unwrap())
    .categorical_color_scale(CategoricalColorScale::from_map(colors.clone()));
    let input = hued_raincloud_geometry(&["active"]);
    let supplied = Plot::raincloud(RaincloudInput::geometry(input))
        .categorical_color_scale(CategoricalColorScale::from_map(colors));
    let canvas = Canvas::grid(1, 2)
        .unwrap()
        .panel(0, 0, sample)
        .unwrap()
        .panel(0, 1, supplied)
        .unwrap()
        .labels(Labels::humanized().alias("treatment", "Study arm"))
        .legend_policy(CanvasLegendPolicy::Shared);
    let directory = unique_dir("raincloud-shared-legend");
    let output = directory.join("plot.svg");
    canvas.svg_preserve_text(&output).unwrap();
    let svg = std::fs::read_to_string(output).unwrap();
    assert_eq!(svg.matches("id=\"canvas-shared-legend\"").count(), 1);
    assert!(svg.contains(">Study arm</text>"));
    assert!(svg.contains("#333333") && svg.contains("#0072B2"));
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn precomputed_raincloud_rejects_duplicate_category_hue_identities_before_backend_begin() {
    let mut input = hued_raincloud_geometry(&["control", "active"]);
    input.category = CategoryValues::new("condition", ["control", "control"]).unwrap();
    input.hue = Some(CategoryValues::new("treatment", ["control", "control"]).unwrap());
    assert!(Plot::raincloud(RaincloudInput::geometry(input))
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("duplicate category-hue identity"));
}

#[test]
fn precomputed_histogram_rejects_faceting_before_backend_begin() {
    let error = Plot::histogram(HistogramInput::geometry(explicit_histogram()))
        .facet(FacetSpec::column(
            lichtung_plot::CategoryValues::new("facet", std::iter::empty::<&str>()).unwrap(),
        ))
        .preflight()
        .unwrap_err();

    assert!(error.to_string().contains("not supported with faceting"));
}

#[test]
fn precomputed_histogram_rejects_source_rows_before_backend_begin() {
    let error = Plot::histogram(HistogramInput::geometry(explicit_histogram()))
        .source_rows(SourceRows::new([]).unwrap())
        .preflight()
        .unwrap_err();

    assert!(error
        .to_string()
        .contains("does not accept source-row inputs"));
}

#[test]
fn precomputed_raincloud_renders_without_sample_columns_and_maps_strip_coordinates() {
    let svg = render(
        Plot::raincloud(RaincloudInput::geometry(explicit_raincloud())),
        "explicit-raincloud",
    )
    .unwrap();
    let marks = svg
        .split_once("<g clip-path=")
        .unwrap()
        .1
        .split_once("</g>")
        .unwrap()
        .0;
    let circles = parse_circle_bounds(marks);
    assert_eq!(circles.len(), 2);
    assert!(circles[0].x < circles[1].x);
}

#[test]
fn precomputed_raincloud_rejects_out_of_range_strip_coordinate_before_backend_begin() {
    let mut input = explicit_raincloud();
    input.normalized_strip = NumericValues::new("strip", [1.1, 1.0]);
    assert!(Plot::raincloud(RaincloudInput::geometry(input))
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("normalized strip coordinate"));
}

#[test]
fn precomputed_raincloud_rejects_nonincreasing_density_before_backend_begin() {
    let mut input = explicit_raincloud();
    input.density_y = NumericValues::new("y", [0.0, 0.0, 2.0]);
    assert!(Plot::raincloud(RaincloudInput::geometry(input))
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("strictly increasing"));
}

#[test]
fn precomputed_histogram_rejects_discontinuous_bins_before_backend_begin() {
    let mut input = explicit_histogram();
    input.bin_lower = NumericValues::new("lower", [0.0, 1.1]);
    assert!(Plot::histogram(HistogramInput::geometry(input))
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("ordered and contiguous"));
}

#[test]
fn raincloud_cloud_box_and_strip_have_separated_geometry() {
    let svg = render(
        raincloud_plot(&[1.0, 2.0, 3.0, 4.0, 5.0]),
        "raincloud-separated",
    )
    .unwrap();
    let polygons = parse_polygon_bounds(&svg);
    let points = parse_circle_bounds(&svg);
    assert!(polygons.len() >= 2 && !points.is_empty());
    assert!(polygons[0].x + polygons[0].width < polygons[1].x);
    assert!(points
        .iter()
        .all(|point| point.x > polygons[1].x + polygons[1].width));
}

#[test]
fn raincloud_strip_retains_every_source_observation_without_sampling() {
    let svg = render(
        raincloud_plot(&[1.0, 1.0, 2.0, 2.0, 3.0, 3.0, 4.0]),
        "raincloud-all-points",
    )
    .unwrap();
    assert_eq!(parse_circle_bounds(&svg).len(), 7);
}

#[test]
fn raincloud_strip_placement_follows_explicit_source_rows_after_reordering() {
    let first = Plot::raincloud(RaincloudInput::samples(
        lichtung_plot::CategoryValues::new("category", ["a"; 3]).unwrap(),
        lichtung_plot::NumericValues::new("value", [1.0, 2.0, 3.0]),
    ))
    .source_rows(SourceRows::new([10, 20, 30]).unwrap());
    let reordered = Plot::raincloud(RaincloudInput::samples(
        lichtung_plot::CategoryValues::new("category", ["a"; 3]).unwrap(),
        lichtung_plot::NumericValues::new("value", [3.0, 1.0, 2.0]),
    ))
    .source_rows(SourceRows::new([30, 10, 20]).unwrap());
    let first = parse_circle_bounds(&render(first, "raincloud-source-first").unwrap());
    let reordered = parse_circle_bounds(&render(reordered, "raincloud-source-second").unwrap());
    assert_eq!(reordered[0].x.to_bits(), first[2].x.to_bits());
    assert_eq!(reordered[1].x.to_bits(), first[0].x.to_bits());
    assert_eq!(reordered[2].x.to_bits(), first[1].x.to_bits());
}

#[test]
fn histogram_rug_retains_every_source_observation_including_duplicates() {
    let svg = render(
        histogram_plot(&[0.0, 0.0, 1.0, 1.0, 2.0], HistogramKde::Internal),
        "histogram-rugs",
    )
    .unwrap();
    assert_eq!(svg.matches("stroke-width=\"0.7001953125\"").count(), 5);
}

#[test]
fn histogram_composite_requires_density_before_backend_begin() {
    let error = Plot::histogram(HistogramInput::samples(lichtung_plot::NumericValues::new(
        "value",
        [0.0, 1.0, 2.0],
    )))
    .composite(HistogramComposite::KdeAndRug(HistogramKde::Internal))
    .preflight()
    .unwrap_err();
    assert!(error
        .to_string()
        .contains("requires HistogramStat::Density"));
}

#[test]
fn precomputed_kde_rejects_nonincreasing_points_before_backend_begin() {
    let error = histogram_plot(
        &[0.0, 1.0, 2.0],
        HistogramKde::Precomputed {
            x: NumericValues::new("x", [1.0, 0.5]),
            y: NumericValues::new("density", [0.2, 0.3]),
        },
    )
    .preflight()
    .unwrap_err();
    assert!(error.to_string().contains("strictly increasing"));
}

#[test]
fn precomputed_kde_rejects_nonfinite_coordinates_before_backend_begin() {
    let error = histogram_plot(
        &[0.0, 1.0, 2.0],
        HistogramKde::Precomputed {
            x: NumericValues::new("x", [f64::NAN, 2.0]),
            y: NumericValues::new("density", [0.2, 0.3]),
        },
    )
    .preflight()
    .unwrap_err();

    assert!(error.to_string().contains("finite x"));
}

#[test]
fn precomputed_kde_rejects_negative_density_before_backend_begin() {
    let error = histogram_plot(
        &[0.0, 1.0, 2.0],
        HistogramKde::Precomputed {
            x: NumericValues::new("x", [0.0, 2.0]),
            y: NumericValues::new("density", [-0.1, 0.3]),
        },
    )
    .preflight()
    .unwrap_err();

    assert!(error.to_string().contains("nonnegative finite density"));
}

#[test]
fn raincloud_rejects_duplicate_source_row_identities_before_backend_begin() {
    assert!(SourceRows::new([4, 4, 8])
        .unwrap_err()
        .to_string()
        .contains("duplicate 4"));
}
