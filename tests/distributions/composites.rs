use lichtung_plot::{
    Canvas, CanvasLegendPolicy, CategoricalColorMap, CategoricalColorScale, FacetSpec,
    HistogramBin, HistogramBins, HistogramComposite, HistogramDimensions, HistogramGeometry,
    HistogramInput, HistogramKde, HistogramKdePoint, HistogramRug, HistogramStat, Labels,
    OpaqueColor, Plot, RaincloudDensityPoint, RaincloudDimensions, RaincloudGeometry,
    RaincloudGroup, RaincloudInput, RaincloudPoint, Significance, SignificanceMark, SourceRows,
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
    plot: Plot<K>,
    name: &str,
) -> Result<String, lichtung_plot::Error> {
    let directory = unique_dir(name);
    let output = directory.join("plot.svg");
    let result = plot
        .svg_preserve_text(&output)
        .and_then(|_| std::fs::read_to_string(&output).map_err(lichtung_plot::Error::Io));
    std::fs::remove_dir_all(directory).unwrap();
    result
}

fn explicit_raincloud() -> RaincloudGeometry {
    RaincloudGeometry {
        dimensions: RaincloudDimensions {
            category: "category".into(),
            value: "value".into(),
            hue: Some("condition".into()),
        },
        groups: vec![RaincloudGroup {
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

fn explicit_histogram() -> HistogramGeometry {
    HistogramGeometry {
        dimensions: HistogramDimensions {
            value: "value".into(),
            density: "density".into(),
        },
        bins: vec![
            HistogramBin {
                lower: 0.0,
                upper: 1.0,
                density: 0.4,
            },
            HistogramBin {
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
        rugs: vec![HistogramRug {
            x: 1.0,
            baseline: 0.1,
            top: 0.2,
        }],
    }
}

#[expect(
    clippy::cast_precision_loss,
    reason = "small focused test samples map their counts and hash mantissas to f64"
)]
fn parity_raincloud_geometry() -> RaincloudGeometry {
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
            RaincloudGroup {
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
    RaincloudGeometry {
        dimensions: RaincloudDimensions {
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

#[test]
fn supplied_and_sample_rainclouds_preserve_labels_colors_legend_and_recorded_marks() {
    let categories = lichtung_plot::CategoryValues::new(
        "condition",
        [
            "control", "control", "control", "control", "control", "active", "active", "active",
            "active", "active",
        ],
    )
    .unwrap();
    let series = lichtung_plot::CategoryValues::new(
        "treatment",
        [
            "control", "control", "control", "control", "control", "active", "active", "active",
            "active", "active",
        ],
    )
    .unwrap();
    let sample = Plot::raincloud(RaincloudInput::samples(
        categories,
        lichtung_plot::NumericValues::new(
            "score",
            [1.0, 2.0, 3.0, 4.0, 5.0, 2.0, 3.0, 4.0, 5.0, 6.0],
        ),
    ))
    .series(series);
    let format = lichtung_plot::NumericFormat::new(
        lichtung_plot::NumericNotation::Engineering,
        lichtung_plot::NumericPrecision::Significant(3),
    )
    .unwrap()
    .with_prefix("raw=")
    .unwrap();
    let supplied = Plot::raincloud(RaincloudInput::geometry(parity_raincloud_geometry()))
        .y_tick_format(format.clone());
    let sample = sample.y_tick_format(format);

    let supplied = render(supplied, "supplied-raincloud").unwrap();
    let sample = render(sample, "sample-raincloud").unwrap();

    for svg in [&supplied, &sample] {
        assert!(svg.contains(">Condition</text>") && svg.contains(">Score</text>"));
        assert!(svg.contains(">Control</text>") && svg.contains(">Active</text>"));
        assert!(svg.contains("#0072B2") && svg.contains("#D55E00"));
        assert_eq!(svg.matches("<circle ").count(), 12);
        assert!(svg.contains(">raw="));
    }
}

#[test]
fn sample_raincloud_accepts_and_renders_pairwise_significance() {
    let plot = Plot::raincloud(RaincloudInput::samples(
        lichtung_plot::CategoryValues::new("category", ["a", "a", "a", "a", "b", "b", "b", "b"])
            .unwrap(),
        lichtung_plot::NumericValues::new("value", [1.0, 1.5, 2.0, 2.5, 2.0, 2.5, 3.0, 3.5]),
    ))
    .significance([Significance::between("a", "b", SignificanceMark::TwoStars)]);

    let svg = render(plot, "sample-raincloud-significance").unwrap();

    assert_eq!(svg.matches("class=\"significance-bracket\"").count(), 1);
    assert!(svg.contains(">**</text>"));
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
        .with_decimal_scale(2)
        .unwrap()
        .with_suffix("%")
        .unwrap();
    let values = [0.0_f64, 0.5, 1.5, 2.0];
    let sample = histogram_plot(
        &values,
        HistogramKde::Precomputed(vec![
            HistogramKdePoint { x: 0.0, y: 0.5 },
            HistogramKdePoint { x: 2.0, y: 0.5 },
        ]),
    );
    let rug_top = 0.5 * 0.035;
    let mut supplied = explicit_histogram();
    supplied.bins.iter_mut().for_each(|bin| bin.density = 0.5);
    supplied.kde = vec![
        HistogramKdePoint { x: 0.0, y: 0.5 },
        HistogramKdePoint { x: 2.0, y: 0.5 },
    ];
    supplied.rugs = values
        .into_iter()
        .map(|x| HistogramRug {
            x,
            baseline: 0.0,
            top: rug_top,
        })
        .collect();
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
    let mut input = parity_raincloud_geometry();
    input.groups.remove(0);
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
fn precomputed_raincloud_accepts_repeated_categories_with_distinct_hues() {
    let mut input = parity_raincloud_geometry();
    input.groups[1].category = input.groups[0].category.clone();
    Plot::raincloud(RaincloudInput::geometry(input))
        .preflight()
        .unwrap();
}

#[test]
fn precomputed_raincloud_rejects_duplicate_category_hue_identities_before_backend_begin() {
    let mut input = parity_raincloud_geometry();
    input.groups[1].category = input.groups[0].category.clone();
    input.groups[1].hue = input.groups[0].hue.clone();
    assert!(Plot::raincloud(RaincloudInput::geometry(input))
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("duplicate category-hue identity"));
}

#[test]
fn precomputed_histogram_accepts_signed_zero_contiguous_bin_edges() {
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
    Plot::histogram(HistogramInput::geometry(input))
        .preflight()
        .unwrap();
}

#[test]
fn precomputed_histogram_rejects_kde_over_safety_budget_before_backend_begin() {
    let mut input = explicit_histogram();
    input.kde = vec![HistogramKdePoint { x: 0.5, y: 0.1 }; 2_049];
    assert!(Plot::histogram(HistogramInput::geometry(input))
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("safety limit of 2048"));
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
fn precomputed_histogram_renders_without_sample_columns_and_maps_explicit_rug_endpoints() {
    let svg = render(
        Plot::histogram(HistogramInput::geometry(explicit_histogram())),
        "explicit-histogram",
    )
    .unwrap();
    assert!(svg.contains("stroke-width=\"1\"") && svg.contains("stroke-width=\"0.7001953125\""));
}

#[test]
fn precomputed_raincloud_rejects_out_of_range_strip_coordinate_before_backend_begin() {
    let mut input = explicit_raincloud();
    input.groups[0].points[0].normalized_strip = 1.1;
    assert!(Plot::raincloud(RaincloudInput::geometry(input))
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("normalized strip coordinate"));
}

#[test]
fn precomputed_raincloud_rejects_nonincreasing_density_before_backend_begin() {
    let mut input = explicit_raincloud();
    input.groups[0].density[1].y = 0.0;
    assert!(Plot::raincloud(RaincloudInput::geometry(input))
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("strictly increasing"));
}

#[test]
fn precomputed_histogram_rejects_nonfinite_geometry_before_backend_begin() {
    let mut input = explicit_histogram();
    input.kde[0].y = f64::NAN;
    assert!(Plot::histogram(HistogramInput::geometry(input))
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("nonnegative finite y"));
}

#[test]
fn precomputed_histogram_rejects_discontinuous_bins_before_backend_begin() {
    let mut input = explicit_histogram();
    input.bins[1].lower = 1.1;
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
fn raincloud_uses_exact_categorical_colors_for_bodies_points_and_legend() {
    let colors = CategoricalColorMap::new([
        ("Control".to_owned(), OpaqueColor::rgb(51, 51, 51)),
        ("Semantic Agent".to_owned(), OpaqueColor::rgb(0, 114, 178)),
    ])
    .unwrap();
    let plot = Plot::raincloud(RaincloudInput::samples(
        lichtung_plot::CategoryValues::new("category", ["a", "a", "a", "a", "b", "b", "b", "b"])
            .unwrap(),
        lichtung_plot::NumericValues::new("value", [1.0, 2.0, 3.0, 4.0, 2.0, 3.0, 4.0, 5.0]),
    ))
    .series(
        lichtung_plot::CategoryValues::new(
            "condition",
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
        .unwrap(),
    )
    .categorical_color_scale(CategoricalColorScale::from_map(colors));
    let svg = render(plot, "raincloud-colors").unwrap();
    assert!(svg.contains("#333333") && svg.contains("#0072B2"));
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
fn precomputed_kde_and_histogram_bars_share_density_projection() {
    let svg = render(
        histogram_plot(
            &[0.0, 0.5, 1.5, 2.0],
            HistogramKde::Precomputed(vec![
                HistogramKdePoint { x: 0.0, y: 0.5 },
                HistogramKdePoint { x: 2.0, y: 0.5 },
            ]),
        ),
        "histogram-density-projection",
    )
    .unwrap();
    let bars = parse_polygon_bounds(&svg);
    assert_eq!(bars.len(), 2);
    assert!((bars[0].y - bars[1].y).abs() < f64::EPSILON);
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
        HistogramKde::Precomputed(vec![
            HistogramKdePoint { x: 1.0, y: 0.2 },
            HistogramKdePoint { x: 0.5, y: 0.3 },
        ]),
    )
    .preflight()
    .unwrap_err();
    assert!(error.to_string().contains("strictly increasing"));
}

#[test]
fn precomputed_kde_rejects_nonfinite_coordinates_before_backend_begin() {
    let error = histogram_plot(
        &[0.0, 1.0, 2.0],
        HistogramKde::Precomputed(vec![
            HistogramKdePoint {
                x: f64::NAN,
                y: 0.2,
            },
            HistogramKdePoint { x: 2.0, y: 0.3 },
        ]),
    )
    .preflight()
    .unwrap_err();

    assert!(error.to_string().contains("finite x"));
}

#[test]
fn precomputed_kde_rejects_negative_density_before_backend_begin() {
    let error = histogram_plot(
        &[0.0, 1.0, 2.0],
        HistogramKde::Precomputed(vec![
            HistogramKdePoint { x: 0.0, y: -0.1 },
            HistogramKdePoint { x: 2.0, y: 0.3 },
        ]),
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

#[test]
fn raincloud_accepts_source_row_column_without_duplicate_overlay() {
    let svg = render(
        raincloud_plot(&[1.0, 2.0, 3.0]).source_rows(SourceRows::new([9, 3, 7]).unwrap()),
        "raincloud-source-rows",
    )
    .unwrap();
    assert_eq!(parse_circle_bounds(&svg).len(), 3);
}

#[test]
fn explicit_inert_composite_options_match_default_svg_bytes() {
    let histogram_default = Plot::histogram(HistogramInput::samples(
        lichtung_plot::NumericValues::new("value", [0.0, 1.0, 2.0, 3.0]),
    ));
    let histogram_explicit = Plot::histogram(HistogramInput::samples(
        lichtung_plot::NumericValues::new("value", [0.0, 1.0, 2.0, 3.0]),
    ))
    .composite(HistogramComposite::HistogramOnly);
    assert_eq!(
        render(histogram_default, "histogram-default").unwrap(),
        render(histogram_explicit, "histogram-explicit").unwrap(),
    );
}
