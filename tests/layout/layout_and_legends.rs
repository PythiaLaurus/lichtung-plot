use super::*;

fn render<K: lichtung_plot::kind::Marker>(plot: Plot<K>, name: &str) -> String {
    let directory = unique_dir(name);
    let output = directory.join("plot.svg");
    plot.svg_preserve_text(&output).unwrap();
    let svg = std::fs::read_to_string(output).unwrap();
    std::fs::remove_dir_all(directory).unwrap();
    svg
}

fn fixture_scatter() -> Plot<lichtung_plot::kind::Scatter> {
    let (x, y) = fixture_values();
    Plot::scatter(x, y)
}

#[test]
fn caption_reserves_space_uses_theme_font_and_is_drawn_after_axes() {
    let plain = render(fixture_scatter().size(320, 240), "caption-plain");
    let captioned = render(
        fixture_scatter()
            .caption("Caption text")
            .x_label("x axis")
            .size(320, 240),
        "captioned",
    );
    let plain_clip = parse_first_clip_rect(&plain);
    let captioned_clip = parse_first_clip_rect(&captioned);
    assert!(captioned_clip.height < plain_clip.height);
    assert!(captioned.contains("font-size=\"15\"") && captioned.contains(">Caption text</text>"));
    assert!(
        captioned.find(">Caption text</text>").unwrap() > captioned.find(">x axis</text>").unwrap()
    );
}

#[test]
fn x_tick_labels_clear_tick_marks_by_their_measured_edge_gap() {
    let svg = render(
        Plot::scatter(
            lichtung_plot::NumericValues::new("x", [0.0, 1.0]),
            lichtung_plot::NumericValues::new("y", [0.0, 1.0]),
        )
        .x_label("Horizontal axis")
        .x_limits(0.0, 1.0)
        .x_ticks(vec![AxisTick::new(0.0, "zero"), AxisTick::new(1.0, "one")])
        .size(360, 260),
        "tick-edge-gap",
    );
    let text = parse_text_elements(&svg);
    let tick = text_element(&text, "zero");
    let clip = parse_first_clip_rect(&svg);
    let endpoint = clip.y + clip.height + 5.0;
    let gap = tick.y - tick.font_size * 0.6 - endpoint;
    // This SVG helper estimates glyph height. Exact retained-ink clearance is
    // checked in the native axis tests; the owned label gap is 8px, not an
    // incidental whole-pixel anchor/line-height offset.
    assert!(gap >= 8.0, "gap={gap}");
}

#[test]
fn native_single_panel_left_inset_tracks_selected_labels_not_root_size() {
    let svg = render(
        fixture_scatter()
            .y_label("response")
            .y_ticks(vec![AxisTick::new(10.0, "10")])
            .size(320, 240),
        "compact-left-inset",
    );
    let left = parse_first_clip_rect(&svg).x;
    let larger = render(
        fixture_scatter()
            .y_label("response")
            .y_ticks(vec![AxisTick::new(10.0, "10")])
            .size(640, 480),
        "native-larger-root",
    );
    assert!((parse_first_clip_rect(&larger).x - left).abs() < 1.0 / 1024.0);
    let wider_label = render(
        fixture_scatter()
            .y_label("response")
            .y_ticks(vec![AxisTick::new(10.0, "Extended measurement")])
            .size(640, 480),
        "native-wider-tick",
    );
    assert!(parse_first_clip_rect(&wider_label).x > left);
}

#[test]
fn scatter_hue_legend_preserves_observed_order() {
    let x = NumericValues::new("x", [1.0, 2.0, 3.0, 4.0, 5.0]);
    let y = NumericValues::new("y", [1.0, 2.0, 1.5, 2.5, 3.0]);
    let group = CategoryValues::new("group", ["high", "high", "low", "mid", "low"]).unwrap();
    let svg = render(
        Plot::scatter(x, y).hue(group).size(360, 260),
        "observed-order",
    );
    let high = svg.find(">High</text>").unwrap();
    let low = svg.find(">Low</text>").unwrap();
    let mid = svg.find(">Mid</text>").unwrap();
    assert!(high < low && low < mid && !svg.contains("&quot;high&quot;"));
}

#[test]
fn scatter_legend_avoids_upper_right_cluster_without_covering_graphics() {
    let x = NumericValues::new("x", [8.0, 8.4, 8.8, 9.2, 9.6, 1.0, 2.0, 3.0]);
    let y = NumericValues::new("y", [8.0, 8.5, 8.2, 9.0, 9.4, 1.0, 2.0, 1.5]);
    let group = CategoryValues::new(
        "group",
        ["high", "high", "high", "high", "high", "low", "low", "mid"],
    )
    .unwrap();
    let svg = render(
        Plot::scatter(x, y).hue(group).size(360, 260),
        "scatter-cluster-legend",
    );
    assert!(parse_legend_rect(&svg).1 < 100.0);
    assert_outside_legend_clear(&svg, parse_first_clip_rect(&svg), "scatter");
}

#[test]
fn regression_automatic_legend_is_outside_and_hidden_by_default() {
    let x = NumericValues::new("x", [8.0, 8.4, 8.8, 9.2, 9.6, 1.0, 2.0, 3.0]);
    let y = NumericValues::new("y", [8.0, 8.5, 8.2, 9.0, 9.4, 1.0, 2.0, 1.5]);
    let regression = || Plot::regression(x.clone(), y.clone()).size(360, 260);
    let default = render(regression(), "regression-no-legend");
    assert!(!has_legend(&default));
    let visible = render(
        regression().legend(LegendPolicy::Visible(LegendPosition::Auto)),
        "regression-visible-legend",
    );
    assert_outside_legend_clear(&visible, parse_first_clip_rect(&visible), "regression");
}

#[test]
fn translated_spectrum_legend_does_not_cover_graphics_or_axes() {
    let layer = PointLayer::new(
        lichtung_plot::NumericValues::new("selected mz", [200.0]),
        lichtung_plot::NumericValues::new("selected intensity", [10.0]),
    )
    .with_color(lichtung_plot::Color::rgba(213, 94, 0, 230))
    .legend_label("Selected peaks");
    let svg = render(
        Plot::line(
            lichtung_plot::NumericValues::new("mz", [100.0, 150.0, 200.0, 250.0, 300.0]),
            lichtung_plot::NumericValues::new("intensity", [1.0, 2.0, 10.0, 2.0, 1.0]),
        )
        .markers(false)
        .point_layer(layer)
        .size(360, 260),
        "translated-spectrum-legend",
    );
    assert_outside_legend_clear(&svg, parse_first_clip_rect(&svg), "spectrum line");
}

fn grouped_bar(
    group: &CategoryValues,
    series: &CategoryValues,
    count: &NumericValues,
    horizontal: bool,
) -> Plot {
    if horizontal {
        Plot::horizontal_bar(group.clone(), BarInput::estimates(count.clone()))
            .series(series.clone())
            .size(420, 300)
            .erase()
    } else {
        Plot::bar(group.clone(), BarInput::estimates(count.clone()))
            .series(series.clone())
            .size(420, 300)
            .erase()
    }
}

#[test]
fn grouped_bar_legends_do_not_cover_graphics_or_axes() {
    let group = CategoryValues::new("group", ["AA", "AA", "AB", "AB", "BB", "BB"]).unwrap();
    let series = CategoryValues::new(
        "series",
        [
            "Observed", "Expected", "Observed", "Expected", "Observed", "Expected",
        ],
    )
    .unwrap();
    let count = NumericValues::new("count", [10.0, 8.0, 6.0, 7.0, 3.0, 4.0]);
    for (horizontal, name) in [(false, "bar"), (true, "horizontal-bar")] {
        let svg = render(grouped_bar(&group, &series, &count, horizontal), name);
        assert_outside_legend_clear(&svg, parse_first_clip_rect(&svg), name);
    }
}

#[test]
fn horizontal_bar_long_ticks_clear_y_axis_label() {
    let group = CategoryValues::new("group", ["locus_0 aa", "locus_0 Aa", "locus_0 AA"]).unwrap();
    let count = NumericValues::new("count", [25.0, 50.0, 25.0]);
    let svg = render(
        Plot::horizontal_bar(group, BarInput::estimates(count))
            .labels(Labels::raw())
            .y_label("Locus and genotype")
            .size(420, 300),
        "horizontal-long-labels",
    );
    let text = parse_text_elements(&svg);
    let axis = text_element(&text, "Locus and genotype");
    // Native categorical axes use the common intrinsic label column, wrapping
    // at spaces rather than retaining the old horizontal-bar-only allocation.
    let tick = text_element(&text, "locus_0 ");
    assert_eq!(
        text.iter()
            .filter(|entry| entry.content == "locus_0 ")
            .count(),
        3
    );
    for genotype in ["aa", "Aa", "AA"] {
        assert_eq!(
            text.iter()
                .filter(|entry| entry.content == genotype)
                .count(),
            1
        );
        assert!(text_element(&text, genotype).x >= tick.x);
    }
    let axis_line = svg
        .lines()
        .find(|line| line.contains(">Locus and genotype</text>"))
        .unwrap();
    let axis_x = axis_line
        .split_once("transform=\"rotate(-90.0,")
        .unwrap()
        .1
        .split_once(',')
        .unwrap()
        .0
        .parse::<f64>()
        .unwrap();
    let gap = tick.x - (axis_x + axis.font_size / 2.0);
    assert!(axis_x - axis.font_size / 2.0 >= 12.0);
    assert!(gap >= 12.0, "gap={gap}");
}

#[test]
fn single_series_bar_does_not_draw_legend() {
    let group = CategoryValues::new("group", ["A", "A", "B", "B"]).unwrap();
    let value = NumericValues::new("value", [1.0, 2.0, 4.0, 6.0]);
    let svg = render(
        Plot::bar(group, BarInput::samples(value)).size(360, 260),
        "bar-no-legend",
    );
    assert!(!has_legend(&svg));
}

#[test]
fn grouped_distribution_legends_do_not_cover_graphics_or_axes() {
    let box_dose = CategoryValues::new(
        "dose",
        ["low", "low", "low", "low", "high", "high", "high", "high"],
    )
    .unwrap();
    let box_batch = CategoryValues::new("batch", ["a", "a", "b", "b", "a", "a", "b", "b"]).unwrap();
    let box_value = NumericValues::new("value", [1.0, 1.2, 1.6, 1.8, 4.0, 4.2, 4.6, 4.8]);
    let violin_dose = CategoryValues::new(
        "dose",
        [
            "low", "low", "low", "low", "low", "low", "high", "high", "high", "high", "high",
            "high",
        ],
    )
    .unwrap();
    let violin_batch = CategoryValues::new(
        "batch",
        ["a", "a", "a", "b", "b", "b", "a", "a", "a", "b", "b", "b"],
    )
    .unwrap();
    let violin_value = NumericValues::new(
        "value",
        [1.0, 1.2, 1.4, 1.7, 1.9, 2.1, 4.0, 4.2, 4.4, 4.7, 4.9, 5.1],
    );
    for (plot, name) in [
        (
            Plot::boxplot(box_dose, box_value)
                .series(box_batch)
                .size(420, 300)
                .erase(),
            "boxplot",
        ),
        (
            Plot::violin(violin_dose, violin_value)
                .series(violin_batch)
                .size(420, 300)
                .erase(),
            "violin",
        ),
    ] {
        let svg = render(plot, name);
        assert_outside_legend_clear(&svg, parse_first_clip_rect(&svg), name);
    }
}

#[test]
fn scatter_legend_prefers_top_when_clear() {
    let x = NumericValues::new("x", [4.0, 4.2, 4.4, 4.6, 4.8, 5.0]);
    let y = NumericValues::new("y", [4.0, 4.2, 4.1, 4.3, 4.0, 4.2]);
    let group = CategoryValues::new("group", ["high", "high", "low", "low", "mid", "mid"]).unwrap();
    let svg = render(
        Plot::scatter(x, y)
            .hue(group)
            .x_limits(0.0, 10.0)
            .y_limits(0.0, 10.0)
            .size(360, 260),
        "legend-prefers-top",
    );
    assert!(parse_legend_rect(&svg).1 < 100.0);
}

#[test]
fn explicit_inside_and_outside_legend_positions_are_honored() {
    let x = NumericValues::new("x", [4.0, 4.2, 4.4, 4.6, 4.8, 5.0]);
    let y = NumericValues::new("y", [4.0, 4.2, 4.1, 4.3, 4.0, 4.2]);
    let group = CategoryValues::new("group", ["high", "high", "low", "low", "mid", "mid"]).unwrap();
    let scatter = |position, width, height| {
        Plot::scatter(x.clone(), y.clone())
            .hue(group.clone())
            .legend(LegendPolicy::Visible(position))
            .size(width, height)
    };
    let inside = render(
        scatter(LegendPosition::InsideTopLeft, 360, 260),
        "inside-top-left",
    );
    let inside_legend = parse_legend_svg_rect(&inside);
    let inside_panel = parse_first_clip_rect(&inside);
    assert!(inside_legend.x < inside_panel.x + 40.0);
    assert_inside_legend_clear(&inside, inside_panel, "inside top-left");

    let outside = render(
        scatter(LegendPosition::OutsideRightBottom, 460, 300),
        "outside-right-bottom",
    );
    let legend = parse_legend_svg_rect(&outside);
    let panel = parse_first_clip_rect(&outside);
    assert!(legend.x > panel.x + panel.width);
    assert!(legend.y + legend.height > panel.y + panel.height - 30.0);
}

#[test]
fn top_scatter_legend_does_not_expand_lower_y_range() {
    let x = NumericValues::new("x", [8.0, 8.4, 8.8, 9.2, 9.6, 1.0, 2.0, 3.0]);
    let y = NumericValues::new("y", [8.0, 8.5, 8.2, 9.0, 9.4, 1.0, 2.0, 1.5]);
    let group = CategoryValues::new(
        "group",
        ["high", "high", "high", "high", "high", "low", "low", "mid"],
    )
    .unwrap();
    let svg = render(
        Plot::scatter(x, y).hue(group).size(360, 260),
        "top-legend-range",
    );
    assert!(!svg.contains(">-5</text>") && !svg.contains(">-10</text>"));
}

#[test]
fn scatter_legend_entry_font_is_readable() {
    let x = NumericValues::new("x", [1.0, 2.0, 3.0]);
    let y = NumericValues::new("y", [1.0, 2.0, 3.0]);
    let group = CategoryValues::new("group", ["high", "low", "mid"]).unwrap();
    let svg = render(Plot::scatter(x, y).hue(group).size(640, 480), "legend-font");
    assert!(svg.contains("font-size=\"11\"") && svg.contains(">High</text>"));
}

#[test]
fn publication_theme_uses_larger_default_text() {
    let svg = render(
        fixture_scatter()
            .title("Publication title")
            .x_label("Publication axis")
            .size(360, 260),
        "publication-text-size",
    );
    assert!(svg.contains("font-size=\"16\""));
    assert!(svg.contains("font-size=\"17\"") && svg.contains(">Publication axis</text>"));
    assert!(svg.contains("font-size=\"20\"") && svg.contains(">Publication title</text>"));
}

fn assert_range_padding<K: lichtung_plot::kind::Marker>(plot: Plot<K>, name: &str, padded: bool) {
    let svg = render(plot.size(360, 260), name);
    let clip = parse_first_clip_rect(&svg);
    let circles = parse_circles(&svg);
    let (min_x, max_x, min_y, max_y) = circle_bounds(&circles);
    if padded {
        assert!(min_x > clip.x + 2.0 && max_x < clip.x + clip.width - 2.0);
        assert!(min_y > clip.y + 2.0 && max_y < clip.y + clip.height - 2.0);
    } else {
        assert!((min_x - clip.x).abs() < 1.0);
        assert!((max_x - (clip.x + clip.width)).abs() < 1.0);
        assert!((min_y - clip.y).abs() < 1.0);
        assert!((max_y - (clip.y + clip.height)).abs() < 1.0);
    }
}

#[test]
fn scatter_auto_range_pads_data_from_frame() {
    assert_range_padding(
        Plot::scatter(
            lichtung_plot::NumericValues::new("x", [0.0, 10.0]),
            lichtung_plot::NumericValues::new("y", [0.0, 10.0]),
        ),
        "scatter-auto-padding",
        true,
    );
}

#[test]
fn scatter_explicit_limits_are_not_padded() {
    assert_range_padding(
        Plot::scatter(
            lichtung_plot::NumericValues::new("x", [0.0, 10.0]),
            lichtung_plot::NumericValues::new("y", [0.0, 10.0]),
        )
        .x_limits(0.0, 10.0)
        .y_limits(0.0, 10.0),
        "scatter-exact-limits",
        false,
    );
}

#[test]
fn regression_auto_range_pads_data_from_frame() {
    assert_range_padding(
        Plot::regression(
            lichtung_plot::NumericValues::new("x", [0.0, 5.0, 10.0]),
            lichtung_plot::NumericValues::new("y", [0.0, 5.0, 10.0]),
        ),
        "regression-auto-padding",
        true,
    );
}

#[test]
fn regression_explicit_limits_are_not_padded() {
    assert_range_padding(
        Plot::regression(
            lichtung_plot::NumericValues::new("x", [0.0, 5.0, 10.0]),
            lichtung_plot::NumericValues::new("y", [0.0, 5.0, 10.0]),
        )
        .x_limits(0.0, 10.0)
        .y_limits(0.0, 10.0),
        "regression-exact-limits",
        false,
    );
}
