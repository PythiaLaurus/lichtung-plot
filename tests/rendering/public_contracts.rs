use super::*;

fn render<K: lichtung_plot::kind::Marker>(plot: Plot<K>, name: &str) -> String {
    let directory = unique_dir(name);
    let output = directory.join("plot.svg");
    plot.svg_preserve_text(&output).unwrap();
    let svg = std::fs::read_to_string(output).unwrap();
    std::fs::remove_dir_all(directory).unwrap();
    svg
}

#[test]
fn erased_plot_preserves_selected_theme_text_dimensions_and_output() {
    use lichtung_plot::{PlotKind, Theme};

    let (x, y) = fixture_values();
    let exact = Plot::scatter(x, y)
        .theme(Theme::Nature)
        .size(321, 234)
        .title("Erasure parity");

    assert_eq!(exact.kind(), PlotKind::Scatter);
    assert_eq!((exact.width(), exact.height()), (321, 234));
    let exact_svg = render(exact.clone(), "exact-before-erasure");
    assert!(exact_svg.contains(">Erasure parity</text>"));
    assert!(exact_svg.contains("width=\"321\" height=\"234\""));

    let erased: Plot = exact.erase();
    assert_eq!(erased.kind(), PlotKind::Scatter);
    assert_eq!((erased.width(), erased.height()), (321, 234));
    erased.preflight().unwrap();

    assert_eq!(render(erased, "erased-terminal-output"), exact_svg);
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one public inventory checks every constructor with its contextual and placement capabilities"
)]
fn constructors_and_contextual_builders_preserve_exact_plot_markers() {
    use lichtung_plot::{
        kind, AxisScale, BarPointLayer, BarSpacing, Bounds, CategoryValues, CoefficientInput,
        Color, DumbbellEndpoints, HeatmapAnnotations, HeatmapLayout, HeatmapScale, HistogramBins,
        HistogramComposite, HistogramDimensions, HistogramGeometry, HistogramInput, HistogramStat,
        InteractionStyle, LinePathStyle, NumericValues, PlotKind, PointLayer, QqLine,
        RaincloudDimensions, RaincloudGeometry, RaincloudInput, RegressionBand, WaterfallLabels,
    };

    fn exact<K: kind::ExactFamily>(plot: Plot<K>, expected: PlotKind) {
        assert_eq!(plot.kind(), expected);
        assert_eq!(plot.erase().kind(), expected);
    }

    fn numeric(name: &'static str) -> NumericValues {
        NumericValues::new(name, [1.0, 2.0])
    }

    fn category(name: &'static str) -> CategoryValues {
        CategoryValues::new(name, ["a", "b"]).unwrap()
    }

    fn bounds() -> Bounds {
        Bounds::new(numeric("lower"), numeric("upper"))
    }

    let attached = || Significance::at("a", SignificanceMark::OneStar);
    let pairwise = || Significance::between("a", "b", SignificanceMark::TwoStars);
    let bar_points = BarPointLayer::new(
        category("point category"),
        None,
        numeric("point value"),
        Color::rgba(1, 2, 3, 128),
        2,
    )
    .unwrap();

    exact::<kind::Scatter>(Plot::scatter(numeric("x"), numeric("y")), PlotKind::Scatter);
    exact::<kind::Histogram>(
        Plot::histogram(HistogramInput::samples(numeric("value")))
            .bins(HistogramBins::Count(4))
            .stat(HistogramStat::Density)
            .composite(HistogramComposite::HistogramOnly),
        PlotKind::Histogram,
    );
    exact::<kind::Histogram>(
        Plot::histogram(HistogramInput::geometry(HistogramGeometry {
            dimensions: HistogramDimensions {
                value: "value".into(),
                density: "density".into(),
            },
            bins: Vec::new(),
            kde: Vec::new(),
            rugs: Vec::new(),
        })),
        PlotKind::Histogram,
    );
    exact::<kind::Density>(Plot::density(numeric("value")), PlotKind::Density);
    exact::<kind::Qq>(
        Plot::qq(numeric("value")).reference_line(QqLine::Identity),
        PlotKind::Qq,
    );
    exact::<kind::PointInterval>(
        Plot::point_interval(category("category"), numeric("estimate"), bounds()),
        PlotKind::PointInterval,
    );
    exact::<kind::Line>(
        Plot::line(numeric("x"), numeric("y"))
            .point_layer(
                PointLayer::new(numeric("point x"), numeric("point y"))
                    .with_color(Color::rgb(1, 2, 3))
                    .with_radius(2)
                    .unwrap()
                    .with_alpha(128),
            )
            .band(bounds()),
        PlotKind::Line,
    );
    exact::<kind::Trajectory>(
        Plot::trajectory(numeric("x"), numeric("y"), category("group")),
        PlotKind::Trajectory,
    );
    exact::<kind::LinePath>(
        Plot::line_path(numeric("x"), numeric("y"), category("group")).style(LinePathStyle::Hue),
        PlotKind::LinePath,
    );
    exact::<kind::Interaction>(
        Plot::interaction(
            category("condition"),
            numeric("estimate"),
            category("group"),
            bounds(),
        )
        .connectors(false)
        .styles([
            InteractionStyle::new("a").unwrap(),
            InteractionStyle::new("b").unwrap(),
        ]),
        PlotKind::Interaction,
    );
    exact::<kind::Coefficient>(
        Plot::coefficient(
            category("term"),
            CoefficientInput::estimates(numeric("estimate")),
        )
        .reference_line(0.0)
        .significance([attached()]),
        PlotKind::Coefficient,
    );
    exact::<kind::Forest>(
        Plot::forest(category("term"), numeric("estimate"), bounds(), 1.0).scale(AxisScale::Log10),
        PlotKind::Forest,
    );
    exact::<kind::Dumbbell>(
        Plot::dumbbell(
            category("category"),
            numeric("left"),
            numeric("right"),
            DumbbellEndpoints::new(
                "Left",
                Color::rgb(20, 80, 140),
                "Right",
                Color::rgb(180, 60, 40),
            )
            .unwrap(),
        ),
        PlotKind::Dumbbell,
    );
    exact::<kind::Heatmap>(
        Plot::heatmap(category("x"), category("y"), numeric("fill"))
            .scale(HeatmapScale::Sequential)
            .layout(HeatmapLayout::Cartesian)
            .annotations(HeatmapAnnotations::Hidden)
            .color_limits(0.0, 3.0),
        PlotKind::Heatmap,
    );
    exact::<kind::Bar>(
        Plot::bar(category("category"), BarInput::estimates(numeric("value")))
            .spacing(BarSpacing::Compact)
            .point_layer(bar_points.clone())
            .significance([attached()])
            .significance([pairwise()]),
        PlotKind::Bar,
    );
    exact::<kind::HorizontalBar>(
        Plot::horizontal_bar(category("category"), BarInput::estimates(numeric("value")))
            .spacing(BarSpacing::Comfort)
            .point_layer(bar_points)
            .significance([attached()])
            .significance([pairwise()]),
        PlotKind::HorizontalBar,
    );
    exact::<kind::StackedBar>(
        Plot::stacked_bar(category("category"), numeric("value"), category("group"))
            .significance([pairwise()]),
        PlotKind::StackedBar,
    );
    exact::<kind::StackedHorizontalBar>(
        Plot::stacked_horizontal_bar(category("category"), numeric("value"), category("group"))
            .significance([pairwise()]),
        PlotKind::StackedHorizontalBar,
    );
    exact::<kind::Boxplot>(
        Plot::boxplot(category("category"), numeric("value")).significance([pairwise()]),
        PlotKind::Boxplot,
    );
    exact::<kind::Violin>(
        Plot::violin(category("category"), numeric("value")).significance([pairwise()]),
        PlotKind::Violin,
    );
    exact::<kind::Raincloud>(
        Plot::raincloud(RaincloudInput::samples(
            category("category"),
            numeric("value"),
        ))
        .significance([pairwise()]),
        PlotKind::Raincloud,
    );
    exact::<kind::Raincloud>(
        Plot::raincloud(RaincloudInput::geometry(RaincloudGeometry {
            dimensions: RaincloudDimensions {
                category: "category".into(),
                value: "value".into(),
                hue: None,
            },
            groups: Vec::new(),
        })),
        PlotKind::Raincloud,
    );
    exact::<kind::Regression>(
        Plot::regression(numeric("x"), numeric("y")).band(RegressionBand::Hidden),
        PlotKind::Regression,
    );
    exact::<kind::Marginal>(
        Plot::marginal(numeric("x"), numeric("y")),
        PlotKind::Marginal,
    );
    exact::<kind::Matrix>(
        Plot::matrix(vec![numeric("x"), numeric("y")]),
        PlotKind::Matrix,
    );
    exact::<kind::Beeswarm>(
        Plot::beeswarm(category("category"), numeric("value"), numeric("color"))
            .color_limits(0.0, 3.0),
        PlotKind::Beeswarm,
    );
    exact::<kind::Waterfall>(
        Plot::waterfall(category("label"), numeric("delta"), 0.0)
            .total_labels(WaterfallLabels::new("Base", "Total").unwrap()),
        PlotKind::Waterfall,
    );
}

#[test]
fn categorical_plots_render_stacked_pairwise_brackets() {
    let group = CategoryValues::new("group", ["a", "a", "b", "b", "c", "c"]).unwrap();
    let value = NumericValues::new("value", [1.0, 1.2, 2.0, 2.2, 3.0, 3.2]);
    let comparisons = vec![
        Significance::between("a", "b", SignificanceMark::PValue(0.04)),
        Significance::between("b", "c", SignificanceMark::PValue(0.005)),
        Significance::between("a", "c", SignificanceMark::PValue(0.0005)),
    ];

    for kind in ["bar", "boxplot", "violin"] {
        let plot = match kind {
            "bar" => Plot::bar(group.clone(), BarInput::samples(value.clone()))
                .significance(comparisons.clone())
                .size(360, 280)
                .erase(),
            "boxplot" => Plot::boxplot(group.clone(), value.clone())
                .significance(comparisons.clone())
                .size(360, 280)
                .erase(),
            _ => Plot::violin(group.clone(), value.clone())
                .significance(comparisons.clone())
                .size(360, 280)
                .erase(),
        };
        let svg = render(plot, &format!("significance-{kind}"));

        assert!(svg.contains(">*<"));
        assert!(svg.contains(">**<"));
        assert!(svg.contains(">***<"));
        assert_eq!(svg.matches("class=\"significance-bracket\"").count(), 3);
    }
}

#[test]
fn chinese_text_uses_system_font_fallback_stack_without_embedding() {
    let (x, y) = fixture_values();
    let plot = Plot::scatter(x, y)
        .title("中文分析")
        .x_label("时间")
        .y_label("测量值");
    if let Err(error) = plot.preflight() {
        assert!(error.to_string().contains("do not contain U+4E2D"));
        return;
    }
    let svg = render(plot, "chinese-text");

    assert!(svg.contains(&format!(
        "font-family=\"{}\"",
        lichtung_plot::DEFAULT_FONT_FAMILY_STACK
    )));
    assert!(svg.contains("中文分析"));
    assert!(!svg.contains("data:font/woff2;base64,"));
}

#[test]
fn scatter_points_use_white_outline_by_default() {
    let (x, y) = fixture_values();
    let svg = render(
        Plot::scatter(x, y).size(320, 240),
        "scatter-point-outline-default",
    );

    assert!(
        svg.contains("stroke=\"#FFFFFF\" stroke-width=\"0.2998046875\""),
        "scatter points should have a thin white outline by default"
    );
}

#[test]
fn scatter_point_outline_can_be_disabled() {
    let (x, y) = fixture_values();
    let svg = render(
        Plot::scatter(x, y).hide_point_outline().size(320, 240),
        "scatter-point-outline-disabled",
    );

    assert!(
        !svg.contains("stroke=\"#FFFFFF\""),
        "hide_point_outline should render fill-only points"
    );
}

#[test]
fn preflight_rejects_invalid_explicit_axis_limits_centrally() {
    let x = NumericValues::new("x", [1.0, 2.0, 3.0]);
    let y = NumericValues::new("y", [2.0, 3.0, 4.0]);
    for (x_limits, y_limits) in [
        ((1.0, 1.0), (0.0, 1.0)),
        ((2.0, 1.0), (0.0, 1.0)),
        ((f64::NAN, 1.0), (0.0, 1.0)),
        ((0.0, 1.0), (0.0, f64::INFINITY)),
    ] {
        let plot = Plot::scatter(x.clone(), y.clone())
            .x_limits(x_limits.0, x_limits.1)
            .y_limits(y_limits.0, y_limits.1);

        let error = plot.preflight().unwrap_err().to_string();

        assert!(error.contains("limits must be finite and strictly ascending"));
    }
}

#[test]
fn waterfall_rejects_terminal_geometry_below_coordinate_precision() {
    let plot = Plot::waterfall(
        CategoryValues::new("label", ["a", "b"]).unwrap(),
        NumericValues::new("delta", [-10_000_000_000_000_000.0, 1.0]),
        10_000_000_000_000_000.0,
    );

    let error = plot.preflight().unwrap_err();

    assert!(matches!(error, lichtung_plot::Error::Render(_)), "{error}");
    assert!(error.to_string().contains("polygon"), "{error}");
    assert!(error.to_string().contains("collapse"), "{error}");
}

#[test]
fn waterfall_accepts_finite_ordered_geometry_when_detached_delta_sum_overflows() {
    // Two successive deltas overflow as a detached sum; ordered endpoints
    // remain finite. The broad exact window also gives the required
    // 300-digit total label a containing shelf without numeric-edge clipping.
    let span = f64::MAX * 0.6;
    let plot = Plot::waterfall(
        CategoryValues::new("label", ["a", "b", "c"]).unwrap(),
        NumericValues::new("delta", [span, span, -span]),
        -span,
    )
    .size(16_000, 300)
    .x_limits(-f64::MAX * 0.9, f64::MAX * 0.9);

    plot.preflight().unwrap();
}

#[test]
fn waterfall_rejects_nonfinite_intermediate_endpoint_with_source_row() {
    let plot = Plot::waterfall(
        CategoryValues::new("label", ["a", "b"]).unwrap(),
        NumericValues::new("delta", [f64::MAX, f64::MAX]),
        0.0,
    );

    assert_eq!(
        plot.preflight().unwrap_err().to_string(),
        "data error: waterfall: cumulative endpoint is non-finite at source row 1"
    );
}

#[test]
fn signed_horizontal_bar_supports_precomputed_sort_and_zero_line() {
    let svg = render(
        Plot::horizontal_bar(
            CategoryValues::new("label", ["neutral", "positive", "negative"]).unwrap(),
            BarInput::estimates(NumericValues::new("value", [0.1, 2.0, -1.0])),
        )
        .category_order(ValueSort::Descending)
        .color_by_sign()
        .zero_line(),
        "signed-horizontal-bar",
    );

    assert!(svg.contains("Negative"));
}

#[test]
fn scatter_hue_jointly_drops_null_and_nan_pairs_without_panic() {
    let svg = render(
        Plot::scatter(
            NumericValues::nullable("x", [Some(1.0), None, Some(f64::NAN), Some(4.0)]),
            NumericValues::nullable("y", [Some(2.0), Some(3.0), Some(4.0), Some(5.0)]),
        )
        .hue(CategoryValues::new("group", ["a", "a", "b", "b"]).unwrap())
        .size(320, 240),
        "scatter-hue-invalid-pairs",
    );

    assert!(svg.contains("Group"), "legend title should be rendered");
}
