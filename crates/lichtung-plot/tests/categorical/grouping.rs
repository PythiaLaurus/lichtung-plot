use super::{parse_polygon_bounds, remove_legend_layers, unique_dir};
use lichtung_plot::{
    BarInput, CategoryValues, FacetSpec, FontFamily, FontStack, Labels, LegendComposition,
    LegendPolicy, NumericValues, Plot, Theme,
};

const COLORS: [&str; 2] = ["#0072B2", "#D55E00"];

fn render<K: lichtung_plot::kind::Marker>(plot: Plot<K>, name: &str) -> String {
    let directory = unique_dir(name);
    let path = directory.join("plot.svg");
    plot.svg_preserve_text(&path).unwrap();
    let svg = std::fs::read_to_string(path).unwrap();
    std::fs::remove_dir_all(directory).unwrap();
    svg
}

fn pair_line() -> Plot<lichtung_plot::kind::Line> {
    Plot::line(
        NumericValues::new("x", [0.0, 1.0, 0.0, 1.0, 0.0, 1.0]),
        NumericValues::new("y", [1.0, 2.0, 3.0, 4.0, 5.0, 6.0]),
    )
    .group(CategoryValues::new("group", ["g1", "g1", "g1", "g1", "g2", "g2"]).unwrap())
    .hue(CategoryValues::new("hue", ["h1", "h1", "h2", "h2", "h2", "h2"]).unwrap())
}

#[test]
fn explicit_group_and_hue_render_only_observed_pair_paths() {
    let svg = render(pair_line(), "line-observed-pairs");
    let marks = remove_legend_layers(&svg);
    let paths = marks
        .lines()
        .filter(|line| {
            line.starts_with("<polyline ") && line.contains("stroke-width=\"1.7998046875\"")
        })
        .collect::<Vec<_>>();
    assert_eq!(paths.len(), 3);
    assert_eq!(
        paths.iter().filter(|line| line.contains(COLORS[0])).count(),
        1
    );
    assert_eq!(
        paths.iter().filter(|line| line.contains(COLORS[1])).count(),
        2
    );
}

#[test]
fn distinct_equal_pair_values_keep_independently_resolved_legend_text() {
    let plot = Plot::line(
        NumericValues::new("x", [0.0, 1.0]),
        NumericValues::new("y", [1.0, 2.0]),
    )
    .group(CategoryValues::new("graphical_group", ["same", "same"]).unwrap())
    .hue(CategoryValues::new("hue_field", ["same", "same"]).unwrap())
    .labels(
        Labels::humanized()
            .alias("graphical_group", "Cohort")
            .alias("hue_field", "Shade")
            .alias("same", "Shared"),
    );
    let error = plot
        .clone()
        .legend_composition(LegendComposition::Separate)
        .preflight()
        .unwrap_err();
    assert!(error
        .to_string()
        .contains("unsupported discrete guide factorization"));
    let svg = render(plot, "distinct-equal-pair-legend");
    assert!(svg.contains(">Cohort × Shade</text>") && svg.contains(">Shared, Shared</text>"));
}

#[test]
fn equal_same_field_channels_collapse_legend_text() {
    let identity = || CategoryValues::new("identity", ["g1", "g1", "g2", "g2"]).unwrap();
    let plot = Plot::line(
        NumericValues::new("x", [0.0, 1.0, 0.0, 1.0]),
        NumericValues::new("y", [1.0, 2.0, 3.0, 4.0]),
    )
    .group(identity())
    .hue(identity());
    for composition in [LegendComposition::Combined, LegendComposition::Separate] {
        let svg = render(
            plot.clone().legend_composition(composition),
            "same-field-collapsed-legend",
        );
        assert!(
            svg.contains(">Identity</text>")
                && svg.contains(">G 1</text>")
                && svg.contains(">G 2</text>")
        );
        assert!(!svg.contains("Identity × Identity") && !svg.contains("G 1, G 1"));
    }
}

#[test]
fn conflicting_same_name_group_and_hue_channels_reject() {
    let plot = Plot::line(
        NumericValues::new("x", [0.0, 1.0]),
        NumericValues::new("y", [1.0, 2.0]),
    )
    .group(CategoryValues::new("identity", ["g1", "g1"]).unwrap())
    .hue(CategoryValues::new("identity", ["g1", "different"]).unwrap());
    assert!(plot
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("share semantic name \"identity\" but differ at source row 1"));
}

#[test]
fn separate_pair_guides_refuse_before_fonts_and_hidden_keeps_color_validation() {
    use lichtung_plot::{CategoricalColorMap, CategoricalColorScale, OpaqueColor};
    let absent = Theme::publication()
        .customize()
        .font_stack(
            FontStack::new([FontFamily::named("Absent factorization preflight font").unwrap()])
                .unwrap(),
        )
        .unwrap()
        .build()
        .unwrap();
    let plot = pair_line().legend_composition(LegendComposition::Separate);
    assert!(plot
        .clone()
        .theme(absent.clone())
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("unsupported discrete guide factorization"));
    plot.clone()
        .legend(LegendPolicy::Hidden)
        .labels(
            Labels::raw()
                .alias("group", "\u{10FFFF}")
                .alias("hue", "\u{10FFFF}"),
        )
        .preflight()
        .unwrap();
    let incomplete =
        CategoricalColorMap::new([("unobserved".into(), OpaqueColor::rgb(42, 42, 42))]).unwrap();
    assert!(plot
        .legend(LegendPolicy::Hidden)
        .categorical_color_scale(CategoricalColorScale::from_map(incomplete))
        .theme(absent)
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("no caller-owned color"));
}

#[test]
fn faceted_series_keep_global_dodge_slots_when_a_panel_omits_a_series() {
    let svg = render(
        Plot::bar(
            CategoryValues::new("category", ["a", "a", "a"]).unwrap(),
            BarInput::estimates(NumericValues::new("value", [1.0, 2.0, 3.0])),
        )
        .series(CategoryValues::new("series", ["g1", "g2", "g2"]).unwrap())
        .facet(FacetSpec::column(
            CategoryValues::new("facet", ["both", "both", "only-g2"]).unwrap(),
        )),
        "faceted-dodge",
    );
    let mut bars = parse_polygon_bounds(&svg);
    bars.sort_by(|left, right| left.x.total_cmp(&right.x));
    assert_eq!(bars.len(), 3);
    assert!(bars[1].x - bars[0].x < bars[2].x - bars[1].x);
    let marks = remove_legend_layers(&svg);
    let polygons = marks
        .lines()
        .filter(|line| line.starts_with("<polygon points="))
        .collect::<Vec<_>>();
    assert_eq!(
        polygons
            .iter()
            .filter(|line| line.contains(COLORS[0]))
            .count(),
        1
    );
    assert_eq!(
        polygons
            .iter()
            .filter(|line| line.contains(COLORS[1]))
            .count(),
        2
    );
    assert_eq!(svg.matches(">G 1</text>").count(), 1);
    assert_eq!(svg.matches(">G 2</text>").count(), 1);
}
