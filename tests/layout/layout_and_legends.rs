use super::*;

fn render<K: lichtung_plot::kind::Marker>(plot: Plot<K>, name: &str) -> String {
    let directory = unique_dir(name);
    let output = directory.join("plot.svg");
    plot.svg(&output).unwrap();
    let svg = std::fs::read_to_string(output).unwrap();
    std::fs::remove_dir_all(directory).unwrap();
    svg
}

fn fixture_scatter() -> Plot<lichtung_plot::kind::Scatter> {
    let (x, y) = fixture_values();
    Plot::scatter(x, y)
}

#[test]
fn layout_respects_explicit_limits_and_tick_count() {
    let x_spec = AxisSpec::default().limits(0.0, 10.0).tick_count(3);
    let y_spec = AxisSpec::default().limits(-1.0, 1.0).tick_count(5);
    let layout = Layout::new_with_axis_specs(
        (2.0, 3.0),
        (4.0, 5.0),
        320,
        240,
        "x".to_string(),
        "y".to_string(),
        &x_spec,
        &y_spec,
    );
    assert_eq!(layout.projection.x_range(), (0.0, 10.0));
    assert_eq!(layout.projection.y_range(), (-1.0, 1.0));
    assert_eq!(layout.x_axis.ticks.len(), 3);
    assert!((layout.x_axis.ticks[0].position - 0.0).abs() < f64::EPSILON);
    assert!((layout.x_axis.ticks[2].position - 10.0).abs() < f64::EPSILON);
}

#[test]
fn layout_auto_range_moves_endpoint_ticks_inside_panel() {
    let layout = Layout::new_with_axis_specs(
        (0.0, 10.0),
        (0.0, 10.0),
        320,
        240,
        "x".to_string(),
        "y".to_string(),
        &AxisSpec::default(),
        &AxisSpec::default(),
    );
    let range = layout.projection.x_range();
    assert!(range.0 < 0.0 && range.1 > 10.0);
    assert!(layout
        .x_axis
        .ticks
        .iter()
        .any(|tick| tick.position.abs() < f64::EPSILON));
    assert!(layout
        .x_axis
        .ticks
        .iter()
        .any(|tick| (tick.position - 10.0).abs() < f64::EPSILON));
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
    let palette = Theme::Publication.palette();
    let endpoint = clip.y + clip.height + f64::from(palette.tick_length);
    let gap = tick.y - tick.font_size * 0.6 - endpoint;
    assert!((7.0..=9.0).contains(&gap), "gap={gap}");
}

#[test]
fn default_single_panel_left_inset_is_compact() {
    let svg = render(
        fixture_scatter().y_label("response").size(320, 240),
        "compact-left-inset",
    );
    let left = parse_first_clip_rect(&svg).x;
    assert!((72.0..=84.0).contains(&left), "left inset={left}");
}

#[test]
fn scatter_hue_legend_preserves_observed_order() {
    let frame = df!(
        "x" => [1.0_f64, 2.0, 3.0, 4.0, 5.0],
        "y" => [1.0_f64, 2.0, 1.5, 2.5, 3.0],
        "group" => ["high", "high", "low", "mid", "low"]
    )
    .unwrap();
    let svg = render(
        Plot::scatter(data::numeric(&frame, "x"), data::numeric(&frame, "y"))
            .hue(data::category(&frame, "group"))
            .size(360, 260),
        "observed-order",
    );
    let high = svg.find(">High</text>").unwrap();
    let low = svg.find(">Low</text>").unwrap();
    let mid = svg.find(">Mid</text>").unwrap();
    assert!(high < low && low < mid && !svg.contains("&quot;high&quot;"));
}

#[test]
fn scatter_legend_avoids_upper_right_cluster_without_covering_graphics() {
    let frame = df!(
        "x" => [8.0_f64, 8.4, 8.8, 9.2, 9.6, 1.0, 2.0, 3.0],
        "y" => [8.0_f64, 8.5, 8.2, 9.0, 9.4, 1.0, 2.0, 1.5],
        "group" => ["high", "high", "high", "high", "high", "low", "low", "mid"]
    )
    .unwrap();
    let svg = render(
        Plot::scatter(data::numeric(&frame, "x"), data::numeric(&frame, "y"))
            .hue(data::category(&frame, "group"))
            .size(360, 260),
        "scatter-cluster-legend",
    );
    assert!(parse_legend_rect(&svg).1 < 100.0);
    assert_inside_legend_clear(&svg, parse_first_clip_rect(&svg), "scatter");
}

#[test]
fn regression_inside_legend_is_clear_and_hidden_by_default() {
    let frame = df!(
        "x" => [8.0_f64, 8.4, 8.8, 9.2, 9.6, 1.0, 2.0, 3.0],
        "y" => [8.0_f64, 8.5, 8.2, 9.0, 9.4, 1.0, 2.0, 1.5]
    )
    .unwrap();
    let regression =
        || Plot::regression(data::numeric(&frame, "x"), data::numeric(&frame, "y")).size(360, 260);
    let default = render(regression(), "regression-no-legend");
    assert!(!has_legend(&default));
    let visible = render(
        regression().legend(LegendPolicy::Visible(LegendPosition::Auto)),
        "regression-visible-legend",
    );
    assert_inside_legend_clear(&visible, parse_first_clip_rect(&visible), "regression");
}

#[test]
fn translated_spectrum_legend_does_not_cover_graphics_or_axes() {
    let layer = PointLayer::new(
        lichtung_plot::NumericValues::new("selected mz", [200.0]),
        lichtung_plot::NumericValues::new("selected intensity", [10.0]),
        (213, 94, 0),
        4,
        0.9,
    )
    .unwrap()
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
    assert_inside_legend_clear(&svg, parse_first_clip_rect(&svg), "spectrum line");
}

fn grouped_bar(frame: &data::DataFrame, horizontal: bool) -> Plot {
    if horizontal {
        Plot::horizontal_bar(
            data::category(frame, "group"),
            BarInput::estimates(data::numeric(frame, "count")),
        )
        .series(data::category(frame, "series"))
        .size(420, 300)
        .erase()
    } else {
        Plot::bar(
            data::category(frame, "group"),
            BarInput::estimates(data::numeric(frame, "count")),
        )
        .series(data::category(frame, "series"))
        .size(420, 300)
        .erase()
    }
}

#[test]
fn grouped_bar_legends_do_not_cover_graphics_or_axes() {
    let frame = df!(
        "group" => ["AA", "AA", "AB", "AB", "BB", "BB"],
        "series" => ["Observed", "Expected", "Observed", "Expected", "Observed", "Expected"],
        "count" => [10.0_f64, 8.0, 6.0, 7.0, 3.0, 4.0]
    )
    .unwrap();
    for (horizontal, name) in [(false, "bar"), (true, "horizontal-bar")] {
        let svg = render(grouped_bar(&frame, horizontal), name);
        assert_inside_legend_clear(&svg, parse_first_clip_rect(&svg), name);
    }
}

#[test]
fn horizontal_bar_long_ticks_clear_y_axis_label() {
    let frame = df!(
        "group" => ["locus_0 aa", "locus_0 Aa", "locus_0 AA"],
        "count" => [25.0_f64, 50.0, 25.0]
    )
    .unwrap();
    let svg = render(
        Plot::horizontal_bar(
            data::category(&frame, "group"),
            BarInput::estimates(data::numeric(&frame, "count")),
        )
        .labels(Labels::raw())
        .y_label("Locus and genotype")
        .size(420, 300),
        "horizontal-long-labels",
    );
    let text = parse_text_elements(&svg);
    let axis = text_element(&text, "Locus and genotype");
    let tick = text_element(&text, "locus_0 aa");
    let gap = tick.x
        - deterministic_text_width(&tick.content, tick.font_size)
        - (axis.x + axis.font_size / 2.0);
    assert!(axis.x - axis.font_size / 2.0 >= 12.0);
    assert!((12.0..=22.0).contains(&gap), "gap={gap}");
}

#[test]
fn single_series_bar_does_not_draw_legend() {
    let frame = df!("group" => ["A", "A", "B", "B"], "value" => [1.0_f64, 2.0, 4.0, 6.0]).unwrap();
    let svg = render(
        Plot::bar(
            data::category(&frame, "group"),
            BarInput::samples(data::numeric(&frame, "value")),
        )
        .size(360, 260),
        "bar-no-legend",
    );
    assert!(!has_legend(&svg));
}

#[test]
fn grouped_distribution_legends_do_not_cover_graphics_or_axes() {
    let box_frame = df!(
        "dose" => ["low", "low", "low", "low", "high", "high", "high", "high"],
        "batch" => ["a", "a", "b", "b", "a", "a", "b", "b"],
        "value" => [1.0_f64, 1.2, 1.6, 1.8, 4.0, 4.2, 4.6, 4.8]
    )
    .unwrap();
    let violin_frame = df!(
        "dose" => ["low", "low", "low", "low", "low", "low", "high", "high", "high", "high", "high", "high"],
        "batch" => ["a", "a", "a", "b", "b", "b", "a", "a", "a", "b", "b", "b"],
        "value" => [1.0_f64, 1.2, 1.4, 1.7, 1.9, 2.1, 4.0, 4.2, 4.4, 4.7, 4.9, 5.1]
    )
    .unwrap();
    let grouped = |frame: &data::DataFrame, violin| {
        if violin {
            Plot::violin(data::category(frame, "dose"), data::numeric(frame, "value"))
                .series(data::category(frame, "batch"))
                .size(420, 300)
                .erase()
        } else {
            Plot::boxplot(data::category(frame, "dose"), data::numeric(frame, "value"))
                .series(data::category(frame, "batch"))
                .size(420, 300)
                .erase()
        }
    };
    for (plot, name) in [
        (grouped(&box_frame, false), "boxplot"),
        (grouped(&violin_frame, true), "violin"),
    ] {
        let svg = render(plot, name);
        assert_inside_legend_clear(&svg, parse_first_clip_rect(&svg), name);
    }
}

#[test]
fn scatter_legend_prefers_top_when_clear() {
    let frame = df!(
        "x" => [4.0_f64, 4.2, 4.4, 4.6, 4.8, 5.0],
        "y" => [4.0_f64, 4.2, 4.1, 4.3, 4.0, 4.2],
        "group" => ["high", "high", "low", "low", "mid", "mid"]
    )
    .unwrap();
    let svg = render(
        Plot::scatter(data::numeric(&frame, "x"), data::numeric(&frame, "y"))
            .hue(data::category(&frame, "group"))
            .x_limits(0.0, 10.0)
            .y_limits(0.0, 10.0)
            .size(360, 260),
        "legend-prefers-top",
    );
    assert!(parse_legend_rect(&svg).1 < 100.0);
}

#[test]
fn explicit_inside_and_outside_legend_positions_are_honored() {
    let frame = df!(
        "x" => [4.0_f64, 4.2, 4.4, 4.6, 4.8, 5.0],
        "y" => [4.0_f64, 4.2, 4.1, 4.3, 4.0, 4.2],
        "group" => ["high", "high", "low", "low", "mid", "mid"]
    )
    .unwrap();
    let scatter = |position, width, height| {
        Plot::scatter(data::numeric(&frame, "x"), data::numeric(&frame, "y"))
            .hue(data::category(&frame, "group"))
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
    let frame = df!(
        "x" => [8.0_f64, 8.4, 8.8, 9.2, 9.6, 1.0, 2.0, 3.0],
        "y" => [8.0_f64, 8.5, 8.2, 9.0, 9.4, 1.0, 2.0, 1.5],
        "group" => ["high", "high", "high", "high", "high", "low", "low", "mid"]
    )
    .unwrap();
    let svg = render(
        Plot::scatter(data::numeric(&frame, "x"), data::numeric(&frame, "y"))
            .hue(data::category(&frame, "group"))
            .size(360, 260),
        "top-legend-range",
    );
    assert!(!svg.contains(">-5</text>") && !svg.contains(">-10</text>"));
}

#[test]
fn scatter_legend_entry_font_is_readable() {
    let frame = df!(
        "x" => [1.0_f64, 2.0, 3.0], "y" => [1.0_f64, 2.0, 3.0],
        "group" => ["high", "low", "mid"]
    )
    .unwrap();
    let svg = render(
        Plot::scatter(data::numeric(&frame, "x"), data::numeric(&frame, "y"))
            .hue(data::category(&frame, "group"))
            .size(640, 480),
        "legend-font",
    );
    assert!(svg.contains("font-size=\"11\"") && svg.contains(">High</text>"));
}

#[test]
fn publication_theme_uses_larger_default_text() {
    let palette = Theme::Publication.palette();
    assert!(palette.tick_font_size >= 16);
    assert!(palette.label_font_size >= 17);
    assert!(palette.title_font_size >= 20);
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
