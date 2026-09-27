use lichtung_plot::{
    BarInput, Bounds, CategoryValues, DumbbellEndpoints, FacetSpec, Labels, LinePathStyle,
    NumericValues, Plot, PlotKind, Significance, SignificanceMark, SignificanceTarget,
};

use super::{parse_polygon_bounds, remove_legend_layers, unique_dir};

const PUBLICATION_COLORS: [(u8, u8, u8); 2] = [(0, 114, 178), (213, 94, 0)];

#[derive(Clone)]
struct CollectiveData {
    group: CategoryValues,
    hue: CategoryValues,
    category: CategoryValues,
    x: NumericValues,
    y: NumericValues,
}

#[derive(Clone)]
struct InteractionData {
    condition: CategoryValues,
    group: CategoryValues,
    hue: CategoryValues,
    estimate: NumericValues,
    lower: NumericValues,
    upper: NumericValues,
}

#[derive(Clone)]
struct DumbbellGroupingData {
    category: CategoryValues,
    group: CategoryValues,
    left: NumericValues,
    right: NumericValues,
}

fn shared_hue_data() -> CollectiveData {
    let groups = ["g1", "g1", "g1", "g2", "g2", "g2", "g3", "g3", "g3"];
    CollectiveData {
        group: CategoryValues::new("group", groups).unwrap(),
        hue: CategoryValues::new(
            "hue",
            [
                "shared", "shared", "shared", "other", "other", "other", "shared", "shared",
                "shared",
            ],
        )
        .unwrap(),
        category: CategoryValues::new("category", ["a"; 9]).unwrap(),
        x: NumericValues::new("x", [0.0, 1.0, 2.0, 0.0, 1.0, 2.0, 0.0, 1.0, 2.0]),
        y: NumericValues::new("y", [1.0, 2.0, 4.0, 2.0, 4.0, 7.0, 3.0, 5.0, 9.0]),
    }
}

fn shared_hue_interaction_data() -> InteractionData {
    InteractionData {
        condition: CategoryValues::new(
            "condition",
            ["before", "after", "before", "after", "before", "after"],
        )
        .unwrap(),
        group: CategoryValues::new("group", ["g1", "g1", "g2", "g2", "g3", "g3"]).unwrap(),
        hue: CategoryValues::new(
            "hue",
            ["shared", "shared", "other", "other", "shared", "shared"],
        )
        .unwrap(),
        estimate: NumericValues::new("estimate", [1.0, 2.0, 2.0, 4.0, 3.0, 5.0]),
        lower: NumericValues::new("lower", [0.5, 1.5, 1.5, 3.5, 2.5, 4.5]),
        upper: NumericValues::new("upper", [1.5, 2.5, 2.5, 4.5, 3.5, 5.5]),
    }
}

fn collective_data() -> CollectiveData {
    CollectiveData {
        group: CategoryValues::new(
            "group",
            [
                "g1", "g1", "g1", "g1", "g1", "g1", "g2", "g2", "g2", "g2", "g2", "g2",
            ],
        )
        .unwrap(),
        hue: CategoryValues::new(
            "hue",
            [
                "h1", "h1", "h1", "h1", "h1", "h1", "h2", "h2", "h2", "h2", "h2", "h2",
            ],
        )
        .unwrap(),
        category: CategoryValues::new(
            "category",
            ["a", "a", "a", "b", "b", "b", "a", "a", "a", "b", "b", "b"],
        )
        .unwrap(),
        x: NumericValues::new(
            "x",
            [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 0.0, 1.0, 2.0, 3.0, 4.0, 5.0],
        ),
        y: NumericValues::new(
            "y",
            [1.0, 2.0, 4.0, 2.0, 4.0, 7.0, 3.0, 5.0, 9.0, 4.0, 7.0, 11.0],
        ),
    }
}

fn interaction_data() -> InteractionData {
    InteractionData {
        condition: CategoryValues::new("condition", ["before", "after", "before", "after"])
            .unwrap(),
        group: CategoryValues::new("group", ["g1", "g1", "g2", "g2"]).unwrap(),
        hue: CategoryValues::new("hue", ["h1", "h1", "h2", "h2"]).unwrap(),
        estimate: NumericValues::new("estimate", [1.0, 2.0, 2.0, 4.0]),
        lower: NumericValues::new("lower", [0.5, 1.5, 1.5, 3.5]),
        upper: NumericValues::new("upper", [1.5, 2.5, 2.5, 4.5]),
    }
}

fn same_name_collective_plots(hues: [&str; 6]) -> Vec<(PlotKind, Plot)> {
    let groups = ["g1", "g1", "g2", "g2", "g3", "g3"];
    let group = || lichtung_plot::CategoryValues::new("identity", groups).unwrap();
    let hue = || lichtung_plot::CategoryValues::new("identity", hues).unwrap();
    let x = || lichtung_plot::NumericValues::new("x", [0.0, 1.0, 0.0, 1.0, 0.0, 1.0]);
    let y = || lichtung_plot::NumericValues::new("y", [1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    let condition = || {
        lichtung_plot::CategoryValues::new(
            "condition",
            ["before", "after", "before", "after", "before", "after"],
        )
        .unwrap()
    };

    vec![
        (
            PlotKind::Line,
            Plot::line(x(), y()).group(group()).hue(hue()).erase(),
        ),
        (
            PlotKind::Density,
            Plot::density(y()).group(group()).hue(hue()).erase(),
        ),
        (
            PlotKind::Interaction,
            Plot::interaction(
                condition(),
                y(),
                group(),
                Bounds::new(
                    lichtung_plot::NumericValues::new("lower", [0.5, 1.5, 2.5, 3.5, 4.5, 5.5]),
                    lichtung_plot::NumericValues::new("upper", [1.5, 2.5, 3.5, 4.5, 5.5, 6.5]),
                ),
            )
            .hue(hue())
            .erase(),
        ),
        (
            PlotKind::Marginal,
            Plot::marginal(x(), y()).group(group()).hue(hue()).erase(),
        ),
        (
            PlotKind::Matrix,
            Plot::matrix(vec![x(), y()])
                .group(group())
                .hue(hue())
                .erase(),
        ),
        (
            PlotKind::LinePath,
            Plot::line_path(x(), y(), group())
                .hue(hue())
                .style(LinePathStyle::Hue)
                .erase(),
        ),
    ]
}

fn configured_hue<K: lichtung_plot::kind::ExactFamily + lichtung_plot::kind::Hue>(
    plot: Plot<K>,
    hue: &CategoryValues,
    with_hue: bool,
) -> Plot {
    if with_hue {
        plot.hue(hue.clone())
    } else {
        plot
    }
    .erase()
}

fn configured_collective_plot(data: &CollectiveData, kind: PlotKind, with_hue: bool) -> Plot {
    let group = || data.group.clone();
    match kind {
        PlotKind::Line => configured_hue(
            Plot::line(data.x.clone(), data.y.clone()).group(group()),
            &data.hue,
            with_hue,
        ),
        PlotKind::Density => configured_hue(
            Plot::density(data.y.clone()).group(group()),
            &data.hue,
            with_hue,
        ),
        PlotKind::Marginal => configured_hue(
            Plot::marginal(data.x.clone(), data.y.clone()).group(group()),
            &data.hue,
            with_hue,
        ),
        PlotKind::Matrix => configured_hue(
            Plot::matrix(vec![data.x.clone(), data.y.clone()]).group(group()),
            &data.hue,
            with_hue,
        ),
        _ => panic!("unsupported collective plot kind {kind:?}"),
    }
}

fn configured_interaction_plot(data: &InteractionData, with_hue: bool) -> Plot {
    configured_hue(
        Plot::interaction(
            data.condition.clone(),
            data.estimate.clone(),
            data.group.clone(),
            Bounds::new(data.lower.clone(), data.upper.clone()),
        ),
        &data.hue,
        with_hue,
    )
}

fn configured_dumbbell_plot(data: &DumbbellGroupingData) -> Plot {
    Plot::dumbbell(
        data.category.clone(),
        data.left.clone(),
        data.right.clone(),
        DumbbellEndpoints::new("Left", (20, 80, 140), "Right", (180, 60, 40)).unwrap(),
    )
    .group(data.group.clone())
    .erase()
}

fn configured_positioned_plot(data: &CollectiveData, kind: PlotKind) -> Plot {
    let series = || data.group.clone();
    match kind {
        PlotKind::Bar => Plot::bar(data.category.clone(), BarInput::samples(data.y.clone()))
            .series(series())
            .erase(),
        PlotKind::HorizontalBar => {
            Plot::horizontal_bar(data.category.clone(), BarInput::samples(data.y.clone()))
                .series(series())
                .erase()
        }
        PlotKind::Boxplot => Plot::boxplot(data.category.clone(), data.y.clone())
            .series(series())
            .erase(),
        PlotKind::Violin => Plot::violin(data.category.clone(), data.y.clone())
            .series(series())
            .erase(),
        _ => panic!("unsupported positioned plot kind {kind:?}"),
    }
}

fn constructor_owned_stacked_plot(data: &CollectiveData, kind: PlotKind) -> Plot {
    let series = || data.group.clone();
    match kind {
        PlotKind::StackedBar => {
            Plot::stacked_bar(data.category.clone(), data.y.clone(), series()).erase()
        }
        PlotKind::StackedHorizontalBar => {
            Plot::stacked_horizontal_bar(data.category.clone(), data.y.clone(), series()).erase()
        }
        _ => panic!("unsupported stacked plot kind {kind:?}"),
    }
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

fn rgb(color: (u8, u8, u8)) -> String {
    format!("#{:02X}{:02X}{:02X}", color.0, color.1, color.2)
}

#[test]
fn separate_group_hue_families_render_group_only_without_group_legends() {
    let data = collective_data();
    let interaction = interaction_data();
    let dumbbell = DumbbellGroupingData {
        category: CategoryValues::new("category", ["a", "a", "b", "b"]).unwrap(),
        group: CategoryValues::new("group", ["g1", "g2", "g1", "g2"]).unwrap(),
        left: NumericValues::new("left", [1.0, 1.5, 2.0, 2.5]),
        right: NumericValues::new("right", [2.0, 2.5, 3.0, 3.5]),
    };
    for kind in [
        PlotKind::Line,
        PlotKind::Interaction,
        PlotKind::Density,
        PlotKind::Marginal,
        PlotKind::Matrix,
        PlotKind::Dumbbell,
    ] {
        let plot = match kind {
            PlotKind::Interaction => configured_interaction_plot(&interaction, false),
            PlotKind::Dumbbell => configured_dumbbell_plot(&dumbbell),
            _ => configured_collective_plot(&data, kind, false),
        };
        let svg = render(plot, "group-only-family")
            .unwrap_or_else(|error| panic!("{kind:?} should support group: {error}"));
        assert!(!svg.contains(">G 1</text>") && !svg.contains(">G 2</text>"));
    }
}

#[test]
fn positioned_and_stacked_families_render_series_legends() {
    let data = collective_data();
    for kind in [
        PlotKind::Bar,
        PlotKind::HorizontalBar,
        PlotKind::StackedBar,
        PlotKind::StackedHorizontalBar,
        PlotKind::Boxplot,
        PlotKind::Violin,
    ] {
        let plot = match kind {
            PlotKind::Bar | PlotKind::HorizontalBar | PlotKind::Boxplot | PlotKind::Violin => {
                configured_positioned_plot(&data, kind)
            }
            PlotKind::StackedBar | PlotKind::StackedHorizontalBar => {
                constructor_owned_stacked_plot(&data, kind)
            }
            _ => unreachable!(),
        };
        let svg = render(plot, "categorical-series-family").unwrap_or_else(|error| {
            panic!("{kind:?} should render its configured series: {error}")
        });
        assert!(svg.contains(">G 1</text>") && svg.contains(">G 2</text>"));
    }
}

#[test]
fn explicit_group_controls_paths_while_hue_controls_color_and_legend() {
    let data = collective_data();
    let group_only = Plot::line(data.x.clone(), data.y.clone()).group(data.group.clone());
    let plain = render(group_only.clone(), "line-group-only").unwrap();
    assert!(!plain.contains(">H 1</text>") && !plain.contains(">H 2</text>"));

    let hued = render(group_only.hue(data.hue.clone()), "line-group-hue").unwrap();
    assert!(hued.contains(">Group × Hue</text>"));
    assert!(hued.contains(">G 1, H 1</text>") && hued.contains(">G 2, H 2</text>"));
}

#[test]
fn explicit_group_and_hue_render_only_observed_pair_paths() {
    let group = CategoryValues::new("group", ["g1", "g1", "g1", "g1", "g2", "g2"]).unwrap();
    let hue = CategoryValues::new("hue", ["h1", "h1", "h2", "h2", "h2", "h2"]).unwrap();
    let x = NumericValues::new("x", [0.0, 1.0, 0.0, 1.0, 0.0, 1.0]);
    let y = NumericValues::new("y", [1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    let svg = render(
        Plot::line(x, y).group(group).hue(hue),
        "line-observed-pairs",
    )
    .unwrap();
    let marks = remove_legend_layers(&svg);
    let paths = marks
        .lines()
        .filter(|line| {
            line.starts_with("<polyline ") && line.contains("stroke-width=\"1.7998046875\"")
        })
        .collect::<Vec<_>>();
    let colors = PUBLICATION_COLORS;

    assert_eq!(paths.len(), 3);
    assert_eq!(
        paths
            .iter()
            .filter(|line| line.contains(&rgb(colors[0])))
            .count(),
        1
    );
    assert_eq!(
        paths
            .iter()
            .filter(|line| line.contains(&rgb(colors[1])))
            .count(),
        2
    );
}

#[test]
fn hue_only_line_and_density_create_one_mark_per_hue() {
    let hue = CategoryValues::new("hue", ["h1", "h1", "h1", "h2", "h2", "h2"]).unwrap();
    let x = NumericValues::new("x", [0.0, 1.0, 2.0, 0.0, 1.0, 2.0]);
    let y = NumericValues::new("y", [1.0, 2.0, 4.0, 3.0, 5.0, 8.0]);
    let line = render(Plot::line(x, y.clone()).hue(hue.clone()), "line-hue-only").unwrap();
    let density = render(Plot::density(y).hue(hue), "density-hue-only").unwrap();

    let line_marks = remove_legend_layers(&line);
    let density_marks = remove_legend_layers(&density);
    assert_eq!(
        line_marks
            .lines()
            .filter(|tag| tag.starts_with("<polyline ")
                && tag.contains("stroke-width=\"1.7998046875\""))
            .count(),
        2
    );
    assert_eq!(
        density_marks
            .matches("fill-opacity=\"0.2196078431372549\"")
            .count(),
        2
    );
}

#[test]
fn density_renders_each_observed_crossed_pair() {
    let group = CategoryValues::new("group", ["g1", "g1", "g1", "g1", "g2", "g2"]).unwrap();
    let hue = CategoryValues::new("hue", ["h1", "h1", "h2", "h2", "h2", "h2"]).unwrap();
    let value = NumericValues::new("value", [1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    let svg = render(
        Plot::density(value).group(group).hue(hue),
        "density-crossed-pairs",
    )
    .unwrap();

    let marks = remove_legend_layers(&svg);
    assert_eq!(
        marks.matches("fill-opacity=\"0.2196078431372549\"").count(),
        3
    );
}

#[test]
fn distinct_equal_pair_values_keep_independently_resolved_legend_text() {
    let plot = Plot::line(
        lichtung_plot::NumericValues::new("x", [0.0, 1.0]),
        lichtung_plot::NumericValues::new("y", [1.0, 2.0]),
    )
    .group(lichtung_plot::CategoryValues::new("graphical_group", ["same", "same"]).unwrap())
    .hue(lichtung_plot::CategoryValues::new("hue_field", ["same", "same"]).unwrap())
    .labels(
        Labels::humanized()
            .alias("graphical_group", "Cohort")
            .alias("hue_field", "Shade")
            .alias("same", "Shared"),
    );
    let svg = render(plot, "distinct-equal-pair-legend").unwrap();

    assert!(svg.contains(">Cohort × Shade</text>"));
    assert!(svg.contains(">Shared, Shared</text>"));
}

#[test]
fn equal_same_field_channels_collapse_legend_text() {
    let identity =
        || lichtung_plot::CategoryValues::new("identity", ["g1", "g1", "g2", "g2"]).unwrap();
    let svg = render(
        Plot::line(
            lichtung_plot::NumericValues::new("x", [0.0, 1.0, 0.0, 1.0]),
            lichtung_plot::NumericValues::new("y", [1.0, 2.0, 3.0, 4.0]),
        )
        .group(identity())
        .hue(identity()),
        "same-field-collapsed-legend",
    )
    .unwrap();

    assert!(svg.contains(">Identity</text>"));
    assert!(svg.contains(">G 1</text>") && svg.contains(">G 2</text>"));
    assert!(!svg.contains("Identity × Identity") && !svg.contains("G 1, G 1"));
}

#[test]
fn density_legend_preserves_curve_and_translucent_fill_without_point() {
    let hue = CategoryValues::new("hue", ["h1", "h1", "h2", "h2"]).unwrap();
    let value = NumericValues::new("value", [1.0, 2.0, 3.0, 4.0]);
    let svg = render(Plot::density(value).hue(hue), "density-exact-legend").unwrap();
    let legend = svg
        .split_once("<g id=\"legend\">")
        .unwrap()
        .1
        .split_once("</g>")
        .unwrap()
        .0;

    assert_eq!(legend.matches("stroke-width=\"1.599609375\"").count(), 2);
    assert_eq!(
        legend
            .matches("fill-opacity=\"0.2196078431372549\"")
            .count(),
        2
    );
    assert!(!legend.contains("<circle "));
}

#[test]
fn density_pair_error_identifies_group_and_hue() {
    let group = CategoryValues::new("group", ["small", "valid", "valid"]).unwrap();
    let hue = CategoryValues::new("hue", ["red", "blue", "blue"]).unwrap();
    let value = NumericValues::new("value", [1.0, 2.0, 3.0]);
    let error = Plot::density(value)
        .group(group)
        .hue(hue)
        .preflight()
        .unwrap_err()
        .to_string();

    assert!(error
        .contains("graphical group \"small\" with hue \"red\" requires at least 2 observations"));
}

#[test]
fn equal_same_name_group_and_hue_channels_collapse_for_every_pair_family() {
    for (kind, plot) in same_name_collective_plots(["g1", "g1", "g2", "g2", "g3", "g3"]) {
        plot.preflight().unwrap_or_else(|error| {
            panic!("{kind:?} should accept equal same-name channels: {error}")
        });
    }
}

#[test]
fn conflicting_same_name_group_and_hue_channels_reject_for_every_pair_family() {
    for (kind, plot) in same_name_collective_plots(["g1", "different", "g2", "g2", "g3", "g3"]) {
        let error = plot.preflight().unwrap_err().to_string();
        assert!(
            error.contains("share semantic name \"identity\" but differ at source row 1"),
            "{kind:?}: {error}"
        );
    }
}

#[test]
fn significance_endpoints_select_configured_series() {
    let data = collective_data();
    let comparison = Significance::between(
        SignificanceTarget::category_series("a", "g1"),
        SignificanceTarget::category_series("a", "g2"),
        SignificanceMark::PValue(0.01),
    );
    let svg = render(
        Plot::boxplot(data.category.clone(), data.y.clone())
            .series(data.group.clone())
            .significance([comparison]),
        "boxplot-series-significance",
    )
    .unwrap();
    assert_eq!(svg.matches("class=\"significance-bracket\"").count(), 1);
}

#[test]
fn faceted_group_hue_pairs_keep_global_hue_colors() {
    let facet = CategoryValues::new(
        "facet",
        ["first", "first", "first", "first", "second", "second"],
    )
    .unwrap();
    let group = CategoryValues::new("group", ["g1"; 6]).unwrap();
    let hue = CategoryValues::new("hue", ["h2", "h2", "h1", "h1", "h1", "h1"]).unwrap();
    let x = NumericValues::new("x", [0.0, 1.0, 0.0, 1.0, 0.0, 1.0]);
    let y = NumericValues::new("y", [1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    let svg = render(
        Plot::line(x, y)
            .group(group)
            .hue(hue)
            .facet(FacetSpec::column(facet)),
        "faceted-line-observed-pairs",
    )
    .unwrap();
    let colors = PUBLICATION_COLORS;
    let marks = remove_legend_layers(&svg);
    let paths = marks
        .lines()
        .filter(|line| {
            line.starts_with("<polyline ") && line.contains("stroke-width=\"1.7998046875\"")
        })
        .collect::<Vec<_>>();

    assert_eq!(paths.len(), 3);
    assert_eq!(
        paths
            .iter()
            .filter(|line| line.contains(&rgb(colors[0])))
            .count(),
        1
    );
    assert_eq!(
        paths
            .iter()
            .filter(|line| line.contains(&rgb(colors[1])))
            .count(),
        2
    );
}

#[test]
fn shared_hue_uses_one_chart_wide_color_across_collective_families() {
    let data = shared_hue_data();
    let interaction = shared_hue_interaction_data();
    let expected = PUBLICATION_COLORS;
    let plots = vec![
        configured_collective_plot(&data, PlotKind::Line, true),
        configured_collective_plot(&data, PlotKind::Density, true),
        configured_collective_plot(&data, PlotKind::Marginal, true),
        configured_collective_plot(&data, PlotKind::Matrix, true),
        configured_interaction_plot(&interaction, true),
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
    let expected = rgb(PUBLICATION_COLORS[0]);
    let svg = render(
        configured_collective_plot(&data, PlotKind::Density, false),
        "group-only-density",
    )
    .unwrap();
    assert!(svg.matches(&expected).count() >= 4);
}

#[test]
fn faceted_series_keep_global_dodge_slots_when_a_panel_omits_a_series() {
    let facet = CategoryValues::new("facet", ["both", "both", "only-g2"]).unwrap();
    let category = CategoryValues::new("category", ["a", "a", "a"]).unwrap();
    let series = CategoryValues::new("series", ["g1", "g2", "g2"]).unwrap();
    let value = NumericValues::new("value", [1.0, 2.0, 3.0]);
    let svg = render(
        Plot::bar(category, BarInput::estimates(value))
            .series(series)
            .facet(FacetSpec::column(facet)),
        "faceted-dodge",
    )
    .unwrap();
    let mut bars = parse_polygon_bounds(&svg);
    bars.sort_by(|left, right| left.x.total_cmp(&right.x));
    assert_eq!(bars.len(), 3);
    assert!(bars[1].x - bars[0].x < bars[2].x - bars[1].x);
    let expected = PUBLICATION_COLORS;
    let marks = remove_legend_layers(&svg);
    let polygons = marks
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
fn faceted_collectives_keep_global_first_observed_hue_color_order() {
    let facet = CategoryValues::new("facet", ["b", "b", "a", "a"]).unwrap();
    let hue = CategoryValues::new("hue", ["h2", "h2", "h1", "h1"]).unwrap();
    let x = NumericValues::new("x", [0.0, 1.0, 0.0, 1.0]);
    let y = NumericValues::new("y", [1.0, 2.0, 3.0, 4.0]);
    let expected = PUBLICATION_COLORS;
    let svg = render(
        Plot::line(x, y).hue(hue).facet(FacetSpec::column(facet)),
        "faceted-hue-order",
    )
    .unwrap();
    let first = svg.find(&rgb(expected[0])).unwrap();
    let second = svg.find(&rgb(expected[1])).unwrap();
    assert!(first < second);
}

#[test]
fn horizontal_significance_uses_configured_series() {
    let data = collective_data();
    let valid = Significance::between(
        SignificanceTarget::category_series("a", "h1"),
        SignificanceTarget::category_series("a", "h2"),
        SignificanceMark::PValue(0.01),
    );
    let base = || {
        Plot::horizontal_bar(data.category.clone(), BarInput::samples(data.y.clone()))
            .series(data.hue.clone())
    };
    let svg = render(
        base().significance([valid]),
        "horizontal-series-significance",
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
