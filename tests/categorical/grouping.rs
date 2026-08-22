use lichtung_plot::{
    BarAggregation, ComparisonAnnotation, ComparisonEndpoint, DumbbellEndpoints, EstimateSource,
    InferentialScale, IntervalKind, IntervalSemantics, Plot, PlotKind,
};

use super::{data, df, parse_polygon_bounds, unique_dir, DataFrame};

fn confidence() -> IntervalSemantics {
    IntervalSemantics::new(
        IntervalKind::Confidence,
        0.95,
        InferentialScale::Response,
        "group aesthetics fixture",
    )
    .unwrap()
}

fn shared_hue_data() -> DataFrame {
    df!(
        "group" => ["g1", "g1", "g1", "g2", "g2", "g2", "g3", "g3", "g3"],
        "hue" => ["shared", "shared", "shared", "other", "other", "other", "shared", "shared", "shared"],
        "x" => [0.0_f64, 1.0, 2.0, 0.0, 1.0, 2.0, 0.0, 1.0, 2.0],
        "y" => [1.0_f64, 2.0, 4.0, 2.0, 4.0, 7.0, 3.0, 5.0, 9.0],
    )
    .unwrap()
}

fn shared_hue_interaction_data() -> DataFrame {
    df!(
        "condition" => ["before", "after", "before", "after", "before", "after"],
        "group" => ["g1", "g1", "g2", "g2", "g3", "g3"],
        "hue" => ["shared", "shared", "other", "other", "shared", "shared"],
        "estimate" => [1.0_f64, 2.0, 2.0, 4.0, 3.0, 5.0],
        "lower" => [0.5_f64, 1.5, 1.5, 3.5, 2.5, 4.5],
        "upper" => [1.5_f64, 2.5, 2.5, 4.5, 3.5, 5.5],
    )
    .unwrap()
}

fn collective_data() -> DataFrame {
    df!(
        "group" => ["g1", "g1", "g1", "g1", "g1", "g1", "g2", "g2", "g2", "g2", "g2", "g2"],
        "hue" => ["h1", "h1", "h1", "h1", "h1", "h1", "h2", "h2", "h2", "h2", "h2", "h2"],
        "category" => ["a", "a", "a", "b", "b", "b", "a", "a", "a", "b", "b", "b"],
        "x" => [0.0_f64, 1.0, 2.0, 3.0, 4.0, 5.0, 0.0, 1.0, 2.0, 3.0, 4.0, 5.0],
        "y" => [1.0_f64, 2.0, 4.0, 2.0, 4.0, 7.0, 3.0, 5.0, 9.0, 4.0, 7.0, 11.0],
    )
    .unwrap()
}

fn interaction_data() -> DataFrame {
    df!(
        "condition" => ["before", "after", "before", "after"],
        "group" => ["g1", "g1", "g2", "g2"],
        "hue" => ["h1", "h1", "h2", "h2"],
        "estimate" => [1.0_f64, 2.0, 2.0, 4.0],
        "lower" => [0.5_f64, 1.5, 1.5, 3.5],
        "upper" => [1.5_f64, 2.5, 2.5, 4.5],
    )
    .unwrap()
}

fn configured_group_plot(data: &DataFrame, kind: PlotKind) -> Plot {
    let group = || data::category(data, "group");
    match kind {
        PlotKind::Line => {
            Plot::line(data::numeric(data, "x"), data::numeric(data, "y")).group(group())
        }
        PlotKind::Interaction => Plot::interaction(
            data::category(data, "condition"),
            data::numeric(data, "estimate"),
            group(),
            data::numeric(data, "lower"),
            data::numeric(data, "upper"),
        )
        .group(group())
        .interval_semantics(confidence())
        .interaction_source(EstimateSource::EstimatedMarginalMean),
        PlotKind::Density => Plot::density(data::numeric(data, "y")).group(group()),
        PlotKind::Marginal => {
            Plot::marginal(data::numeric(data, "x"), data::numeric(data, "y")).group(group())
        }
        PlotKind::Matrix => {
            Plot::matrix(vec![data::numeric(data, "x"), data::numeric(data, "y")]).group(group())
        }
        PlotKind::Bar => {
            Plot::bar(data::category(data, "category"), data::numeric(data, "y")).group(group())
        }
        PlotKind::HorizontalBar => {
            Plot::horizontal_bar(data::category(data, "category"), data::numeric(data, "y"))
                .group(group())
        }
        PlotKind::StackedBar => Plot::stacked_bar(
            data::category(data, "category"),
            data::numeric(data, "y"),
            group(),
        ),
        PlotKind::StackedHorizontalBar => Plot::stacked_horizontal_bar(
            data::category(data, "category"),
            data::numeric(data, "y"),
            group(),
        ),
        PlotKind::Boxplot => {
            Plot::boxplot(data::category(data, "category"), data::numeric(data, "y")).group(group())
        }
        PlotKind::Violin => {
            Plot::violin(data::category(data, "category"), data::numeric(data, "y")).group(group())
        }
        PlotKind::Dumbbell => Plot::dumbbell(
            data::category(data, "category"),
            data::numeric(data, "left"),
            data::numeric(data, "right"),
        )
        .group(group())
        .dumbbell_endpoints(
            DumbbellEndpoints::new("Left", (20, 80, 140), "Right", (180, 60, 40)).unwrap(),
        ),
        _ => panic!("unsupported grouped fixture kind {kind:?}"),
    }
}

fn render(plot: Plot, name: &str) -> Result<String, lichtung_plot::Error> {
    let directory = unique_dir(name);
    let output = directory.join("plot.svg");
    let result = plot
        .svg(&output)
        .and_then(|_| std::fs::read_to_string(&output).map_err(lichtung_plot::Error::Io));
    std::fs::remove_dir_all(directory).unwrap();
    result
}

fn rgb(color: (u8, u8, u8)) -> String {
    format!("#{:02X}{:02X}{:02X}", color.0, color.1, color.2)
}

#[test]
fn exact_supported_families_render_group_only_without_group_legends() {
    let data = collective_data();
    let interaction = interaction_data();
    let dumbbell = df!(
        "category" => ["a", "a", "b", "b"],
        "group" => ["g1", "g2", "g1", "g2"],
        "left" => [1.0_f64, 1.5, 2.0, 2.5],
        "right" => [2.0_f64, 2.5, 3.0, 3.5],
    )
    .unwrap();
    for kind in [
        PlotKind::Line,
        PlotKind::Interaction,
        PlotKind::Density,
        PlotKind::Marginal,
        PlotKind::Matrix,
        PlotKind::Bar,
        PlotKind::HorizontalBar,
        PlotKind::StackedBar,
        PlotKind::StackedHorizontalBar,
        PlotKind::Boxplot,
        PlotKind::Violin,
        PlotKind::Dumbbell,
    ] {
        let source = if kind == PlotKind::Interaction {
            &interaction
        } else if kind == PlotKind::Dumbbell {
            &dumbbell
        } else {
            &data
        };
        let svg = render(configured_group_plot(source, kind), "group-only-family")
            .unwrap_or_else(|error| panic!("{kind:?} should support group: {error}"));
        assert!(!svg.contains(">G 1</text>") && !svg.contains(">G 2</text>"));
    }
}

#[test]
fn every_other_plot_kind_rejects_group_before_backend_begin() {
    let data = collective_data();
    let group = || data::category(&data, "group");
    let unsupported = [
        Plot::scatter(data::numeric(&data, "x"), data::numeric(&data, "y")).group(group()),
        Plot::histogram(data::numeric(&data, "x")).group(group()),
        Plot::qq(data::numeric(&data, "x")).group(group()),
        Plot::regression(data::numeric(&data, "x"), data::numeric(&data, "y")).group(group()),
    ];
    for plot in unsupported {
        assert!(plot
            .preflight()
            .unwrap_err()
            .to_string()
            .contains("graphical group column"));
    }
}

#[test]
fn explicit_group_controls_paths_while_hue_controls_color_and_legend() {
    let data = collective_data();
    let group_only = configured_group_plot(&data, PlotKind::Line);
    let plain = render(group_only.clone(), "line-group-only").unwrap();
    assert!(!plain.contains(">H 1</text>") && !plain.contains(">H 2</text>"));

    let hued = render(
        group_only.hue(data::category(&data, "hue")),
        "line-group-hue",
    )
    .unwrap();
    assert!(hued.contains(">H 1</text>") && hued.contains(">H 2</text>"));
    assert!(!hued.contains(">G 1</text>") && !hued.contains(">G 2</text>"));
}

#[test]
fn explicit_group_rejects_multiple_hues_per_collective_mark() {
    let data = df!(
        "group" => ["g1", "g1", "g2", "g2"],
        "hue" => ["h1", "h2", "h2", "h2"],
        "x" => [0.0_f64, 1.0, 0.0, 1.0],
        "y" => [1.0_f64, 2.0, 3.0, 4.0],
    )
    .unwrap();
    let error = Plot::line(data::numeric(&data, "x"), data::numeric(&data, "y"))
        .group(data::category(&data, "group"))
        .hue(data::category(&data, "hue"))
        .preflight()
        .unwrap_err();
    assert!(error
        .to_string()
        .contains("each collective mark must have exactly one hue"));
    assert!(error.to_string().contains("source row 1"));
}

#[test]
fn categorical_marks_allow_one_group_to_change_hue_between_x_cells() {
    let data = df!(
        "category" => ["a", "a", "a", "a", "b", "b", "b", "b"],
        "group" => ["g1", "g1", "g2", "g2", "g1", "g1", "g2", "g2"],
        "hue" => ["h1", "h1", "h2", "h2", "h2", "h2", "h1", "h1"],
        "y" => [1.0_f64, 2.0, 3.0, 4.0, 2.0, 3.0, 4.0, 5.0],
    )
    .unwrap();
    Plot::bar(data::category(&data, "category"), data::numeric(&data, "y"))
        .group(data::category(&data, "group"))
        .hue(data::category(&data, "hue"))
        .preflight()
        .unwrap();
}

#[test]
fn significance_endpoints_select_explicit_group_and_validate_hue() {
    let data = collective_data();
    let comparison = ComparisonAnnotation {
        endpoint_a: ComparisonEndpoint::new("a").with_group("g1").with_hue("h1"),
        endpoint_b: ComparisonEndpoint::new("a").with_group("g2").with_hue("h2"),
        ..ComparisonAnnotation::new("a", "a", 0.01)
    };
    let svg = render(
        Plot::boxplot(data::category(&data, "category"), data::numeric(&data, "y"))
            .group(data::category(&data, "group"))
            .hue(data::category(&data, "hue"))
            .add_comparisons([comparison]),
        "group-significance",
    )
    .unwrap();
    assert_eq!(svg.matches("class=\"significance-bracket\"").count(), 1);
}

#[test]
fn faceted_group_hue_error_reports_original_source_row_before_begin() {
    let data = df!(
        "facet" => ["first", "first", "second", "second"],
        "group" => ["g1", "g1", "g1", "g1"],
        "hue" => ["h1", "h1", "h1", "h2"],
        "x" => [0.0_f64, 1.0, 0.0, 1.0],
        "y" => [1.0_f64, 2.0, 3.0, 4.0],
    )
    .unwrap();
    let error = Plot::line(data::numeric(&data, "x"), data::numeric(&data, "y"))
        .group(data::category(&data, "group"))
        .hue(data::category(&data, "hue"))
        .facet_col(data::category(&data, "facet"))
        .preflight()
        .unwrap_err();
    assert!(error.to_string().contains("source row 3"), "{error}");
}

#[test]
fn shared_hue_uses_one_chart_wide_color_across_collective_families() {
    let data = shared_hue_data();
    let interaction = shared_hue_interaction_data();
    let expected = lichtung_plot::Theme::Publication.palette().colors;
    let plots = [
        configured_group_plot(&data, PlotKind::Line).hue(data::category(&data, "hue")),
        configured_group_plot(&data, PlotKind::Density).hue(data::category(&data, "hue")),
        configured_group_plot(&data, PlotKind::Marginal).hue(data::category(&data, "hue")),
        configured_group_plot(&data, PlotKind::Matrix).hue(data::category(&data, "hue")),
        configured_group_plot(&interaction, PlotKind::Interaction)
            .hue(data::category(&interaction, "hue")),
    ];
    for (index, plot) in plots.into_iter().enumerate() {
        let svg = render(plot, &format!("shared-hue-{index}")).unwrap();
        assert!(svg.matches(&rgb(expected[0])).count() >= 2, "{index}");
        assert!(svg.contains(&rgb(expected[1])), "{index}");
    }
}

#[test]
fn group_only_density_uses_default_color_for_every_curve_and_fill() {
    let data = collective_data();
    let expected = rgb(lichtung_plot::Theme::Publication.palette().colors[0]);
    let svg = render(
        configured_group_plot(&data, PlotKind::Density),
        "group-only-density",
    )
    .unwrap();
    assert!(svg.matches(&expected).count() >= 4);
}

#[test]
fn faceted_explicit_groups_keep_global_dodge_slots_when_a_panel_omits_a_group() {
    let data = df!(
        "facet" => ["both", "both", "only-g2"],
        "category" => ["a", "a", "a"],
        "group" => ["g1", "g2", "g2"],
        "value" => [1.0_f64, 2.0, 3.0],
    )
    .unwrap();
    let svg = render(
        Plot::bar(
            data::category(&data, "category"),
            data::numeric(&data, "value"),
        )
        .group(data::category(&data, "group"))
        .facet_col(data::category(&data, "facet"))
        .bar_aggregation(BarAggregation::Identity),
        "faceted-dodge",
    )
    .unwrap();
    let mut bars = parse_polygon_bounds(&svg);
    bars.sort_by(|left, right| left.x.total_cmp(&right.x));
    assert_eq!(bars.len(), 3);
    assert!(bars[1].x - bars[0].x < bars[2].x - bars[1].x);
}

#[test]
fn every_categorical_group_family_rejects_conflicting_hue_within_one_cell() {
    let data = df!(
        "category" => ["a", "a"],
        "group" => ["g1", "g1"],
        "hue" => ["h1", "h2"],
        "value" => [1.0_f64, 2.0],
    )
    .unwrap();
    let group = || data::category(&data, "group");
    let hue = || data::category(&data, "hue");
    let plots = [
        Plot::bar(
            data::category(&data, "category"),
            data::numeric(&data, "value"),
        )
        .group(group())
        .hue(hue()),
        Plot::horizontal_bar(
            data::category(&data, "category"),
            data::numeric(&data, "value"),
        )
        .group(group())
        .hue(hue()),
        Plot::boxplot(
            data::category(&data, "category"),
            data::numeric(&data, "value"),
        )
        .group(group())
        .hue(hue()),
        Plot::violin(
            data::category(&data, "category"),
            data::numeric(&data, "value"),
        )
        .group(group())
        .hue(hue()),
    ];
    for plot in plots {
        assert!(plot
            .preflight()
            .unwrap_err()
            .to_string()
            .contains("each collective mark must have exactly one hue"));
    }
}

#[test]
fn faceted_collectives_keep_global_first_observed_hue_color_order() {
    let data = df!(
        "facet" => ["b", "b", "a", "a"],
        "hue" => ["h2", "h2", "h1", "h1"],
        "x" => [0.0_f64, 1.0, 0.0, 1.0],
        "y" => [1.0_f64, 2.0, 3.0, 4.0],
    )
    .unwrap();
    let expected = lichtung_plot::Theme::Publication.palette().colors;
    let svg = render(
        Plot::line(data::numeric(&data, "x"), data::numeric(&data, "y"))
            .hue(data::category(&data, "hue"))
            .facet_col(data::category(&data, "facet")),
        "faceted-hue-order",
    )
    .unwrap();
    let first = svg.find(&rgb(expected[0])).unwrap();
    let second = svg.find(&rgb(expected[1])).unwrap();
    assert!(first < second);
}

#[test]
fn horizontal_significance_uses_group_anchor_and_mismatched_hue_rejects_before_begin() {
    let data = collective_data();
    let valid = ComparisonAnnotation {
        endpoint_a: ComparisonEndpoint::new("a").with_group("g1").with_hue("h1"),
        endpoint_b: ComparisonEndpoint::new("a").with_group("g2").with_hue("h2"),
        ..ComparisonAnnotation::new("a", "a", 0.01)
    };
    let base = || {
        Plot::horizontal_bar(data::category(&data, "category"), data::numeric(&data, "y"))
            .group(data::category(&data, "group"))
            .hue(data::category(&data, "hue"))
    };
    let svg = render(
        base().add_comparisons([valid]),
        "horizontal-group-significance",
    )
    .unwrap();
    assert!(svg.contains("class=\"significance-bracket horizontal\""));

    let invalid = ComparisonAnnotation {
        endpoint_a: ComparisonEndpoint::new("a").with_group("g1").with_hue("h2"),
        endpoint_b: ComparisonEndpoint::new("a").with_group("g2").with_hue("h2"),
        ..ComparisonAnnotation::new("a", "a", 0.01)
    };
    let error = base().add_comparisons([invalid]).preflight().unwrap_err();
    assert!(error.to_string().contains("does not have hue \"h2\""));
}
