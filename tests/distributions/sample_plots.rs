use super::*;

#[test]
fn histogram_does_not_preflight_nonexistent_legend_text() {
    Plot::histogram(HistogramInput::samples(NumericValues::new(
        "value",
        [1.0, 2.0, 3.0],
    )))
    .labels(lichtung_plot::Labels::humanized().alias("series", "\u{10FFFF}"))
    .preflight()
    .unwrap();
}

/// Violin plot renders to a non-empty SVG without
/// panicking. We use a 3-species × 15-observation
/// synthetic sample (smaller than the iris
/// template's, but enough to exercise the KDE).
#[test]
fn violin_smoke() {
    let species: Vec<&str> = (0..15)
        .map(|i| match i % 3 {
            0 => "a",
            1 => "b",
            _ => "c",
        })
        .collect();
    let value: Vec<f64> = (0..15)
        .map(|i| match i % 3 {
            0 => 1.0 + f64::from(i) * 0.1,
            1 => 3.0 + f64::from(i) * 0.1,
            _ => 5.0 + f64::from(i) * 0.1,
        })
        .collect();
    let species = CategoryValues::new("species", species).unwrap();
    let value = NumericValues::new("value", value);

    let dir = unique_dir("violin-smoke");
    let out = dir.join("violin.svg");
    let chart = Plot::violin(species, value)
        .theme(Theme::Publication)
        .size(640, 480);
    chart
        .svg_preserve_text(&out)
        .expect("violin render must succeed");
    let svg_bytes = std::fs::read(&out).expect("svg must exist");
    assert!(!svg_bytes.is_empty(), "violin svg must be non-empty");
}

#[test]
fn mass_spectrum_high_intensity_renders_upward() {
    let mz = NumericValues::new("mz", [100.0, 200.0, 300.0]);
    let intensity = NumericValues::new("intensity", [1.0, 10.0, 1.0]);

    let dir = unique_dir("mass-spectrum-orientation");
    let out = dir.join("mass_spectrum.svg");
    let chart = Plot::line(mz, intensity)
        .markers(false)
        .point_layer(
            PointLayer::new(
                lichtung_plot::NumericValues::new("peak m/z", [200.0]),
                lichtung_plot::NumericValues::new("peak intensity", [10.0]),
            )
            .color((222, 45, 38))
            .opacity(0.9)
            .unwrap(),
        )
        .size(320, 240)
        .title("Mass spectrum");
    chart
        .svg_preserve_text(&out)
        .expect("mass spectrum render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");
    // Find the data polyline (3 points) — skip grid polylines (2 points each).
    let y_values: Vec<f64> = svg
        .split("<polyline points=\"")
        .skip(1) // skip the part before the first polyline tag
        .find_map(|segment| {
            let pts_str = segment.split_once('"').map(|(p, _)| p)?;
            let ys: Vec<f64> = pts_str
                .split_whitespace()
                .filter_map(|point| {
                    point
                        .split_once(',')
                        .and_then(|(_, y)| y.parse::<f64>().ok())
                })
                .collect();
            if ys.len() == 3 {
                Some(ys)
            } else {
                None
            }
        })
        .expect("mass spectrum data polyline with 3 points");

    assert_eq!(y_values.len(), 3);
    assert!(
        y_values[1] < y_values[0] && y_values[1] < y_values[2],
        "high-intensity middle point should render above low-intensity endpoints: {y_values:?}"
    );
}

#[test]
fn beeswarm_is_deterministic_and_renders_continuous_colorbar() {
    let feature = CategoryValues::new("feature", ["a", "a", "a", "b", "b"]).unwrap();
    let effect = NumericValues::new("effect", [0.1, 0.1, 0.2, -0.3, 0.4]);
    let value = NumericValues::new("value", [1.0, 2.0, 3.0, 4.0, 5.0]);
    let dir = unique_dir("beeswarm-deterministic");
    let first = dir.join("first.svg");
    let second = dir.join("second.svg");
    for path in [&first, &second] {
        Plot::beeswarm(feature.clone(), effect.clone(), value.clone())
            .zero_line()
            .size(420, 300)
            .svg_preserve_text(path)
            .unwrap();
    }
    let first_svg = std::fs::read_to_string(first).unwrap();
    let second_svg = std::fs::read_to_string(second).unwrap();
    assert_eq!(first_svg, second_svg);
    assert!(first_svg.contains("Value"));
    assert_eq!(first_svg.matches("<linearGradient ").count(), 1);
    assert_eq!(
        first_svg
            .matches("fill=\"url(#linear-gradient-0)\"")
            .count(),
        1
    );
    assert!(first_svg.contains("<stop offset=\"0.000000\" stop-color=\"#3182BD\"/>"));
    assert!(first_svg.contains("<stop offset=\"1.000000\" stop-color=\"#DE2D26\"/>"));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn histogram_svg_is_deterministic_and_labels_selected_statistic() {
    let value = NumericValues::new("value", [0.0, 0.5, 1.0, 1.5, 2.0, 2.5]);
    let dir = unique_dir("histogram-deterministic");
    let first = dir.join("first.svg");
    let second = dir.join("second.svg");
    for path in [&first, &second] {
        let plot = Plot::histogram(HistogramInput::samples(value.clone()))
            .title("Deterministic histogram")
            .size(420, 300);
        let plot = plot
            .bins(HistogramBins::Count(3))
            .stat(HistogramStat::Probability);
        plot.svg_preserve_text(path).unwrap();
    }

    let first_svg = std::fs::read_to_string(first).unwrap();
    assert_eq!(first_svg, std::fs::read_to_string(second).unwrap());
    assert_eq!(first_svg.matches("<polygon points=").count(), 3);
    assert!(first_svg.contains(">Probability</text>"));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn histogram_implicit_axis_label_coordinates_stay_inside_canvas() {
    let value = NumericValues::new("value", [1.0, 2.0, 3.0]);
    let dir = unique_dir("histogram-implicit-label-layout");
    let output = dir.join("histogram.svg");
    Plot::histogram(HistogramInput::samples(value))
        .svg_preserve_text(&output)
        .unwrap();
    let svg = std::fs::read_to_string(output).unwrap();
    let text = parse_text_elements(&svg);
    let value_label = text_element(&text, "Value");
    assert!(value_label.x.is_finite() && (0.0..=640.0).contains(&value_label.x));
    assert!(value_label.y.is_finite() && (0.0..=480.0).contains(&value_label.y));

    let count_line = svg
        .lines()
        .find(|line| line.contains(">Count</text>"))
        .unwrap();
    let transform = count_line
        .split_once("transform=\"")
        .unwrap()
        .1
        .split_once('"')
        .unwrap()
        .0
        .strip_prefix("rotate(-90.0,")
        .unwrap()
        .strip_suffix(')')
        .unwrap();
    let (pivot_x, pivot_y) = transform.split_once(',').unwrap();
    assert!((0.0..=640.0).contains(&pivot_x.parse::<f64>().unwrap()));
    assert!((0.0..=480.0).contains(&pivot_y.parse::<f64>().unwrap()));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn histogram_explicit_limits_project_marks_beyond_clipped_panel() {
    let value = NumericValues::new("value", [0.0, 2.0, 4.0, 6.0, 8.0, 10.0]);
    let dir = unique_dir("histogram-explicit-limits");
    let output = dir.join("histogram.svg");
    let plot = Plot::histogram(HistogramInput::samples(value)).x_limits(2.0, 8.0);
    let plot = plot.bins(HistogramBins::Count(5));
    plot.svg_preserve_text(&output).unwrap();
    let svg = std::fs::read_to_string(output).unwrap();

    assert_eq!(svg.matches("<polygon points=").count(), 5);
    assert!(svg.contains("<g clip-path=\"url(#clip-0)\">"));
    assert!(svg.contains("<polygon points=\"-"));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn histogram_rejects_null_nonfinite_and_empty_values() {
    let cases = [
        NumericValues::nullable("value", [Some(1.0), None]),
        NumericValues::new("value", [1.0, f64::INFINITY]),
        NumericValues::new("value", []),
    ];
    for (index, values) in cases.into_iter().enumerate() {
        let dir = unique_dir(&format!("histogram-invalid-{index}"));
        let output = dir.join("histogram.svg");
        let chart = Plot::histogram(HistogramInput::samples(values));
        assert!(chart.svg(&output).is_err(), "invalid case {index}");
        std::fs::remove_dir_all(dir).unwrap();
    }
}

#[test]
fn density_hue_is_deterministic_with_independent_curves_and_stable_legend() {
    let value = NumericValues::new("value", [1.0, 2.0, 1.5, 4.0, 5.0, 4.5]);
    let group = CategoryValues::new("group", ["b", "b", "b", "a", "a", "a"]).unwrap();
    let dir = unique_dir("density-hue-deterministic");
    let first = dir.join("first.svg");
    let second = dir.join("second.svg");
    for path in [&first, &second] {
        Plot::density(value.clone())
            .group(group.clone())
            .hue(group.clone())
            .svg_preserve_text(path)
            .unwrap();
    }

    let svg = std::fs::read_to_string(first).unwrap();
    assert_eq!(svg, std::fs::read_to_string(second).unwrap());
    let marks = remove_legend_layers(&svg);
    assert_eq!(
        marks.matches("fill-opacity=\"0.2196078431372549\"").count(),
        2
    );
    assert!(svg.find(">B</text>").unwrap() < svg.find(">A</text>").unwrap());
    assert!(svg.contains(">Value</text>") && svg.contains(">Density</text>"));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn density_best_legend_avoids_top_right_curve_peak() {
    let value = NumericValues::new("value", [0.0, 2.0, 4.0, 9.0, 9.05, 9.1]);
    let group = CategoryValues::new(
        "group",
        ["broad", "broad", "broad", "sharp", "sharp", "sharp"],
    )
    .unwrap();
    let dir = unique_dir("density-legend-avoidance");
    let output = dir.join("density.svg");
    Plot::density(value)
        .group(group.clone())
        .hue(group)
        .svg_preserve_text(&output)
        .unwrap();
    let svg = std::fs::read_to_string(output).unwrap();
    let (legend, _) = parse_legend_rect_and_start(&svg);
    let sharp_line = svg
        .split("<polyline points=\"")
        .skip(1)
        .filter(|segment| segment.contains("stroke-width=\"1.599609375\""))
        .nth(1)
        .unwrap();
    let points = sharp_line.split_once('"').unwrap().0;
    let peak = points
        .split_whitespace()
        .filter_map(|point| point.split_once(','))
        .map(|(x, y)| (x.parse::<f64>().unwrap(), y.parse::<f64>().unwrap()))
        .min_by(|left, right| left.1.total_cmp(&right.1))
        .unwrap();

    assert!(
        peak.0 < legend.x
            || peak.0 > legend.x + legend.width
            || peak.1 < legend.y
            || peak.1 > legend.y + legend.height
    );
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn density_explicit_limits_clip_full_sample_kde() {
    let value = NumericValues::new("value", [0.0, 1.0, 2.0, 8.0, 9.0, 10.0]);
    let dir = unique_dir("density-explicit-limits");
    let output = dir.join("density.svg");
    let chart = Plot::density(value).x_limits(1.0, 2.0).y_limits(0.0, 0.5);

    chart.svg_preserve_text(&output).unwrap();
    let svg = std::fs::read_to_string(output).unwrap();
    let panel = parse_first_clip_rect(&svg);
    let marks = svg
        .split("<g clip-path=\"url(#clip-0)\">")
        .nth(1)
        .unwrap()
        .split("</g>")
        .next()
        .unwrap();

    assert!(svg.contains("<g clip-path=\"url(#clip-0)\">"));
    assert!(parse_graphic_bounds(marks).into_iter().all(|bounds| {
        bounds.x >= panel.x - 1.0
            && bounds.y >= panel.y - 1.0
            && bounds.x + bounds.width <= panel.x + panel.width + 1.0
            && bounds.y + bounds.height <= panel.y + panel.height + 1.0
    }));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn density_clips_large_finite_support_before_projection() {
    let base = 1e307_f64;
    let values = [base, base + 1e293, base + 2e293, base + 3e293];
    let dir = unique_dir("density-large-support-explicit-limits");
    let output = dir.join("density.svg");
    let chart = Plot::density(NumericValues::new("value", values))
        .x_limits(values[1], values[2])
        .y_limits(0.0, 2.4e-294);
    chart.svg_preserve_text(&output).unwrap();
    let svg = std::fs::read_to_string(output).unwrap();
    let panel = parse_first_clip_rect(&svg);
    let marks = svg
        .split("<g clip-path=\"url(#clip-0)\">")
        .nth(1)
        .unwrap()
        .split("</g>")
        .next()
        .unwrap();

    assert!(!svg.contains("NaN") && !svg.contains("inf"));
    assert!(parse_graphic_bounds(marks).into_iter().all(|bounds| {
        bounds.x >= panel.x - 1.0
            && bounds.y >= panel.y - 1.0
            && bounds.x + bounds.width <= panel.x + panel.width + 1.0
            && bounds.y + bounds.height <= panel.y + panel.height + 1.0
    }));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn density_renders_broad_opposite_sign_support_with_tiny_auto_y_range() {
    let extreme = f64::MAX * 0.2;
    let dir = unique_dir("density-broad-opposite-sign-support");
    let output = dir.join("density.svg");
    Plot::density(NumericValues::new("value", [-extreme, extreme]))
        .svg_preserve_text(&output)
        .unwrap();
    let svg = std::fs::read_to_string(output).unwrap();

    assert!(svg.contains("fill-opacity=\"0.2196078431372549\""));
    assert!(!svg.contains("NaN") && !svg.contains("inf"));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn density_rejects_null_and_nonfinite_values() {
    for (index, values) in [
        NumericValues::nullable("value", [Some(1.0), None, Some(2.0)]),
        NumericValues::new("value", [1.0, f64::INFINITY, 2.0]),
    ]
    .into_iter()
    .enumerate()
    {
        let dir = unique_dir(&format!("density-invalid-{index}"));
        let output = dir.join("density.svg");
        assert!(Plot::density(values).svg(&output).is_err());
        std::fs::remove_dir_all(dir).unwrap();
    }
}

#[test]
fn qq_svg_is_deterministic_and_line_modes_are_explicit() {
    let value = NumericValues::new("value", [1.0, 2.0, 3.0, 4.0, 5.0]);
    let dir = unique_dir("qq-deterministic");
    let first = dir.join("first.svg");
    let second = dir.join("second.svg");
    let hidden = dir.join("hidden.svg");
    for path in [&first, &second] {
        Plot::qq(value.clone()).svg_preserve_text(path).unwrap();
    }
    let hidden_plot = Plot::qq(value);
    let hidden_plot = hidden_plot.reference_line(QqLine::Hidden);
    hidden_plot.svg_preserve_text(&hidden).unwrap();

    let svg = std::fs::read_to_string(first).unwrap();
    assert_eq!(svg, std::fs::read_to_string(second).unwrap());
    assert!(svg.contains("stroke-width=\"1.2001953125\""));
    assert!(!std::fs::read_to_string(hidden)
        .unwrap()
        .contains("stroke-width=\"1.2001953125\""));
    assert!(svg.contains(">Theoretical normal quantiles</text>"));
    assert!(svg.contains(">Value</text>"));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn qq_rejects_null_and_nonfinite_values() {
    for (index, values) in [
        NumericValues::nullable("value", [Some(1.0), None]),
        NumericValues::new("value", [1.0, f64::NAN]),
    ]
    .into_iter()
    .enumerate()
    {
        let dir = unique_dir(&format!("qq-invalid-{index}"));
        let output = dir.join("qq.svg");
        assert!(Plot::qq(values).svg(&output).is_err());
        std::fs::remove_dir_all(dir).unwrap();
    }
}

#[test]
fn qq_explicit_limits_clip_points_and_reference_line() {
    let value = NumericValues::new("value", [1.0, 2.0, 3.0, 4.0, 5.0]);
    let dir = unique_dir("qq-explicit-limits");
    let output = dir.join("qq.svg");
    let chart = Plot::qq(value).x_limits(-0.5, 0.5).y_limits(2.0, 4.0);

    chart.svg_preserve_text(&output).unwrap();
    let svg = std::fs::read_to_string(output).unwrap();

    assert!(svg.contains("<g clip-path=\"url(#clip-0)\">"));
    assert!(svg.matches("<circle ").count() < 5);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn qq_renders_constant_large_finite_sample_with_representable_auto_range() {
    let dir = unique_dir("qq-large-constant");
    let output = dir.join("qq.svg");
    Plot::qq(NumericValues::new("value", [1e300, 1e300, 1e300]))
        .svg_preserve_text(&output)
        .unwrap();
    let svg = std::fs::read_to_string(output).unwrap();

    assert_eq!(svg.matches("<circle ").count(), 3);
    assert!(!svg.contains("NaN") && !svg.contains("=\"inf"));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn qq_renders_zero_subnormal_and_f64_max_constants() {
    for (name, value) in [
        ("zero", 0.0),
        ("subnormal", f64::from_bits(1)),
        ("maximum", f64::MAX),
        ("minimum", -f64::MAX),
    ] {
        let dir = unique_dir(&format!("qq-constant-{name}"));
        let output = dir.join("qq.svg");
        Plot::qq(NumericValues::new("value", [value, value, value]))
            .svg_preserve_text(&output)
            .unwrap();
        let svg = std::fs::read_to_string(output).unwrap();

        assert_eq!(svg.matches("<circle ").count(), 3, "{name}");
        assert!(!svg.contains("NaN") && !svg.contains("inf"), "{name}");
        std::fs::remove_dir_all(dir).unwrap();
    }
}

#[test]
fn qq_renders_with_adjacent_finite_explicit_limits() {
    let upper = f64::from_bits(1);
    let dir = unique_dir("qq-adjacent-explicit-limits");
    let output = dir.join("qq.svg");
    let plot = Plot::qq(NumericValues::new("value", [0.0, upper])).y_limits(0.0, upper);
    let plot = plot.reference_line(QqLine::Hidden);
    plot.svg_preserve_text(&output).unwrap();
    let svg = std::fs::read_to_string(output).unwrap();

    assert_eq!(svg.matches("<circle ").count(), 2);
    assert!(!svg.contains("NaN") && !svg.contains("inf"));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn qq_default_quartiles_report_unrepresentable_extreme_geometry() {
    let dir = unique_dir("qq-opposite-extremes-auto");
    let output = dir.join("qq.svg");
    let error = Plot::qq(NumericValues::new(
        "value",
        [-f64::MAX, -f64::MAX, f64::MAX, f64::MAX],
    ))
    .svg(&output)
    .err()
    .unwrap()
    .to_string();

    assert!(error.contains("quartile reference line geometry is not representable"));
    assert!(error.contains("QqLine::Hidden or QqLine::Identity"));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn qq_explicit_limits_skip_opposite_sign_extreme_outliers_before_projection() {
    let dir = unique_dir("qq-opposite-extremes-explicit");
    let output = dir.join("qq.svg");
    let plot = Plot::qq(NumericValues::new("value", [-f64::MAX, 0.0, f64::MAX]))
        .x_limits(-2.0, 2.0)
        .y_limits(-1.0, 1.0);
    let plot = plot.reference_line(QqLine::Hidden);
    plot.svg_preserve_text(&output).unwrap();
    let svg = std::fs::read_to_string(output).unwrap();

    assert_eq!(svg.matches("<circle ").count(), 1);
    assert!(!svg.contains("NaN") && !svg.contains("=\"inf"));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn qq_hidden_line_renders_opposite_extremes_with_exact_auto_range() {
    let dir = unique_dir("qq-opposite-extremes-auto-hidden");
    let output = dir.join("qq.svg");
    let plot = Plot::qq(NumericValues::new("value", [-f64::MAX, f64::MAX]));
    let plot = plot.reference_line(QqLine::Hidden);
    plot.svg_preserve_text(&output).unwrap();
    let svg = std::fs::read_to_string(output).unwrap();

    assert_eq!(svg.matches("<circle ").count(), 2);
    assert!(!svg.contains("NaN") && !svg.contains("inf"));
    std::fs::remove_dir_all(dir).unwrap();
}
