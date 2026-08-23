use lichtung_plot::{
    BarInput, Bounds, DumbbellEndpoints, FacetSpec, Plot, PlotKind, Significance, SignificanceMark,
    SignificanceTarget,
};

use super::{data, df, parse_polygon_bounds, unique_dir, DataFrame};

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

fn configured_hue<K: lichtung_plot::kind::Marker + lichtung_plot::kind::Hue>(
    plot: Plot<K>,
    data: &DataFrame,
    with_hue: bool,
) -> Plot {
    if with_hue {
        plot.hue(data::category(data, "hue"))
    } else {
        plot
    }
    .erase()
}

fn configured_group_plot(data: &DataFrame, kind: PlotKind, with_hue: bool) -> Plot {
    let group = || data::category(data, "group");
    match kind {
        PlotKind::Line => configured_hue(
            Plot::line(data::numeric(data, "x"), data::numeric(data, "y")).group(group()),
            data,
            with_hue,
        ),
        PlotKind::Interaction => configured_hue(
            Plot::interaction(
                data::category(data, "condition"),
                data::numeric(data, "estimate"),
                group(),
                Bounds::new(data::numeric(data, "lower"), data::numeric(data, "upper")),
            )
            .group(group()),
            data,
            with_hue,
        ),
        PlotKind::Density => configured_hue(
            Plot::density(data::numeric(data, "y")).group(group()),
            data,
            with_hue,
        ),
        PlotKind::Marginal => configured_hue(
            Plot::marginal(data::numeric(data, "x"), data::numeric(data, "y")).group(group()),
            data,
            with_hue,
        ),
        PlotKind::Matrix => configured_hue(
            Plot::matrix(vec![data::numeric(data, "x"), data::numeric(data, "y")]).group(group()),
            data,
            with_hue,
        ),
        PlotKind::Bar => configured_hue(
            Plot::bar(
                data::category(data, "category"),
                BarInput::samples(data::numeric(data, "y")),
            )
            .group(group()),
            data,
            with_hue,
        ),
        PlotKind::HorizontalBar => configured_hue(
            Plot::horizontal_bar(
                data::category(data, "category"),
                BarInput::samples(data::numeric(data, "y")),
            )
            .group(group()),
            data,
            with_hue,
        ),
        PlotKind::StackedBar => configured_hue(
            Plot::stacked_bar(
                data::category(data, "category"),
                data::numeric(data, "y"),
                group(),
            ),
            data,
            with_hue,
        ),
        PlotKind::StackedHorizontalBar => configured_hue(
            Plot::stacked_horizontal_bar(
                data::category(data, "category"),
                data::numeric(data, "y"),
                group(),
            ),
            data,
            with_hue,
        ),
        PlotKind::Boxplot => configured_hue(
            Plot::boxplot(data::category(data, "category"), data::numeric(data, "y"))
                .group(group()),
            data,
            with_hue,
        ),
        PlotKind::Violin => configured_hue(
            Plot::violin(data::category(data, "category"), data::numeric(data, "y")).group(group()),
            data,
            with_hue,
        ),
        PlotKind::Dumbbell => Plot::dumbbell(
            data::category(data, "category"),
            data::numeric(data, "left"),
            data::numeric(data, "right"),
        )
        .group(group())
        .dumbbell_endpoints(
            DumbbellEndpoints::new("Left", (20, 80, 140), "Right", (180, 60, 40)).unwrap(),
        )
        .erase(),
        _ => panic!("unsupported grouped fixture kind {kind:?}"),
    }
}

fn render<K: lichtung_plot::kind::Marker>(
    plot: Plot<K>,
    name: &str,
) -> Result<String, lichtung_plot::Error> {
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
fn separate_group_hue_families_render_group_only_without_group_legends() {
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
        PlotKind::Dumbbell,
    ] {
        let source = if kind == PlotKind::Interaction {
            &interaction
        } else if kind == PlotKind::Dumbbell {
            &dumbbell
        } else {
            &data
        };
        let svg = render(
            configured_group_plot(source, kind, false),
            "group-only-family",
        )
        .unwrap_or_else(|error| panic!("{kind:?} should support group: {error}"));
        assert!(!svg.contains(">G 1</text>") && !svg.contains(">G 2</text>"));
    }
}

#[test]
fn unified_categorical_families_treat_group_as_colored_series() {
    let data = collective_data();
    for kind in [
        PlotKind::Bar,
        PlotKind::HorizontalBar,
        PlotKind::StackedBar,
        PlotKind::StackedHorizontalBar,
        PlotKind::Boxplot,
        PlotKind::Violin,
    ] {
        let svg = render(
            configured_group_plot(&data, kind, false),
            "unified-group-family",
        )
        .unwrap_or_else(|error| panic!("{kind:?} should support group as its series: {error}"));
        assert!(svg.contains(">G 1</text>") && svg.contains(">G 2</text>"));
    }
}

#[test]
fn explicit_group_controls_paths_while_hue_controls_color_and_legend() {
    let data = collective_data();
    let group_only = Plot::line(data::numeric(&data, "x"), data::numeric(&data, "y"))
        .group(data::category(&data, "group"));
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
fn significance_endpoints_select_group_when_group_is_last_series_alias() {
    let data = collective_data();
    let comparison = Significance::between(
        SignificanceTarget::category_series("a", "g1"),
        SignificanceTarget::category_series("a", "g2"),
        SignificanceMark::PValue(0.01),
    );
    let svg = render(
        Plot::boxplot(data::category(&data, "category"), data::numeric(&data, "y"))
            .hue(data::category(&data, "hue"))
            .group(data::category(&data, "group"))
            .significance([comparison]),
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
        .facet(FacetSpec::column(data::category(&data, "facet")))
        .preflight()
        .unwrap_err();
    assert!(error.to_string().contains("source row 3"), "{error}");
}

#[test]
fn shared_hue_uses_one_chart_wide_color_across_collective_families() {
    let data = shared_hue_data();
    let interaction = shared_hue_interaction_data();
    let expected = lichtung_plot::Theme::Publication.palette().colors;
    let plots = vec![
        configured_group_plot(&data, PlotKind::Line, true),
        configured_group_plot(&data, PlotKind::Density, true),
        configured_group_plot(&data, PlotKind::Marginal, true),
        configured_group_plot(&data, PlotKind::Matrix, true),
        configured_group_plot(&interaction, PlotKind::Interaction, true),
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
        configured_group_plot(&data, PlotKind::Density, false),
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
            BarInput::estimates(data::numeric(&data, "value")),
        )
        .group(data::category(&data, "group"))
        .facet(FacetSpec::column(data::category(&data, "facet"))),
        "faceted-dodge",
    )
    .unwrap();
    let mut bars = parse_polygon_bounds(&svg);
    bars.sort_by(|left, right| left.x.total_cmp(&right.x));
    assert_eq!(bars.len(), 3);
    assert!(bars[1].x - bars[0].x < bars[2].x - bars[1].x);
    let expected = lichtung_plot::Theme::Publication.palette().colors;
    let polygons = svg
        .lines()
        .filter(|line| line.starts_with("<polygon points="))
        .collect::<Vec<_>>();
    assert_eq!(
        polygons
            .iter()
            .filter(|line| line.contains(&rgb(expected[0])))
            .count(),
        1
    );
    assert_eq!(
        polygons
            .iter()
            .filter(|line| line.contains(&rgb(expected[1])))
            .count(),
        2
    );
    assert_eq!(svg.matches(">G 1</text>").count(), 1);
    assert_eq!(svg.matches(">G 2</text>").count(), 1);
}

#[test]
fn hue_last_replaces_group_for_every_unified_categorical_family() {
    let data = df!(
        "category" => ["a", "a", "a", "a", "a", "a"],
        "group" => ["replaced", "replaced", "replaced", "replaced", "replaced", "replaced"],
        "hue" => ["h1", "h1", "h1", "h2", "h2", "h2"],
        "value" => [1.0_f64, 2.0, 3.0, 4.0, 5.0, 6.0],
    )
    .unwrap();
    let group = || data::category(&data, "group");
    let hue = || data::category(&data, "hue");
    let plots = vec![
        Plot::bar(
            data::category(&data, "category"),
            BarInput::samples(data::numeric(&data, "value")),
        )
        .group(group())
        .hue(hue())
        .erase(),
        Plot::horizontal_bar(
            data::category(&data, "category"),
            BarInput::samples(data::numeric(&data, "value")),
        )
        .group(group())
        .hue(hue())
        .erase(),
        Plot::stacked_bar(
            data::category(&data, "category"),
            data::numeric(&data, "value"),
            group(),
        )
        .hue(hue())
        .erase(),
        Plot::stacked_horizontal_bar(
            data::category(&data, "category"),
            data::numeric(&data, "value"),
            group(),
        )
        .hue(hue())
        .erase(),
        Plot::boxplot(
            data::category(&data, "category"),
            data::numeric(&data, "value"),
        )
        .group(group())
        .hue(hue())
        .erase(),
        Plot::violin(
            data::category(&data, "category"),
            data::numeric(&data, "value"),
        )
        .group(group())
        .hue(hue())
        .erase(),
    ];
    for plot in plots {
        plot.preflight().unwrap();
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
            .facet(FacetSpec::column(data::category(&data, "facet"))),
        "faceted-hue-order",
    )
    .unwrap();
    let first = svg.find(&rgb(expected[0])).unwrap();
    let second = svg.find(&rgb(expected[1])).unwrap();
    assert!(first < second);
}

#[test]
fn horizontal_significance_uses_hue_when_hue_is_last_series_alias() {
    let data = collective_data();
    let valid = Significance::between(
        SignificanceTarget::category_series("a", "h1"),
        SignificanceTarget::category_series("a", "h2"),
        SignificanceMark::PValue(0.01),
    );
    let base = || {
        Plot::horizontal_bar(
            data::category(&data, "category"),
            BarInput::samples(data::numeric(&data, "y")),
        )
        .group(data::category(&data, "group"))
        .hue(data::category(&data, "hue"))
    };
    let svg = render(
        base().significance([valid]),
        "horizontal-group-significance",
    )
    .unwrap();
    assert!(svg.contains("class=\"significance-bracket horizontal\""));

    let invalid = Significance::between(
        SignificanceTarget::category_series("a", "g1"),
        SignificanceTarget::category_series("a", "h2"),
        SignificanceMark::PValue(0.01),
    );
    let error = base().significance([invalid]).preflight().unwrap_err();
    assert!(error.to_string().contains("unknown series \"g1\""));
}
