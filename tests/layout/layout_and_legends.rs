use super::*;

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

    let x_range = layout.projection.x_range();
    assert!(
        x_range.0 < 0.0 && x_range.1 > 10.0,
        "auto x range should broaden past the natural endpoint ticks: {x_range:?}"
    );
    assert!(
        layout
            .x_axis
            .ticks
            .iter()
            .any(|tick| tick.position.abs() < f64::EPSILON)
            && layout
                .x_axis
                .ticks
                .iter()
                .any(|tick| (tick.position - 10.0).abs() < f64::EPSILON),
        "auto x ticks should keep the natural endpoint ticks inside the broadened range: {:?}",
        layout.x_axis.ticks
    );
}

#[test]
fn figure_layout_reserves_caption_space() {
    let df = fixture_df();
    let plain = Chart::new(&df).x("x").y("y").size(320, 240).figure_layout();
    let mut captioned_chart = Chart::new(&df).caption("Caption text");
    let captioned = captioned_chart.x("x").y("y").size(320, 240).figure_layout();

    assert!(
        captioned.panel_box.y0 > plain.panel_box.y0,
        "caption should reserve bottom space outside the plot panel"
    );
}

#[test]
fn x_tick_labels_clear_tick_marks_by_their_measured_edge_gap() {
    let df = df!(
        "x" => [0.0_f64, 1.0],
        "y" => [0.0_f64, 1.0]
    )
    .unwrap();
    let mut chart = Chart::new(&df)
        .x_label("Horizontal axis")
        .x_limits(0.0, 1.0)
        .x_ticks(vec![AxisTick::new(0.0, "zero"), AxisTick::new(1.0, "one")]);
    let _ = chart.x("x").y("y").size(360, 260).scatter();
    let svg = String::from_utf8(chart.render_to_svg_bytes().unwrap()).unwrap();
    let text = parse_text_elements(&svg);
    let tick = text_element(&text, "zero");
    let clip = parse_first_clip_rect(&svg);
    let palette = Theme::Publication.palette();
    let tick_endpoint = clip.y + clip.height + f64::from(palette.tick_length);
    let tick_top = tick.y - tick.font_size * 0.6;
    let gap = tick_top - tick_endpoint;

    assert!(
        (7.0..=9.0).contains(&gap),
        "x tick label should clear the tick mark by the measured 8px gap: gap={gap}"
    );
}

#[test]
fn default_single_panel_left_inset_is_compact() {
    let df = fixture_df();
    let mut chart = Chart::new(&df).y_label("response");
    let layout = chart.x("x").y("y").size(320, 240).figure_layout();

    assert!(
        (60.0..=72.0).contains(&layout.panel_box.x0),
        "left inset should be close to matrix padding without clipping labels: {:?}",
        layout.panel_box
    );
}

#[test]
fn caption_uses_theme_font_size() {
    let df = fixture_df();
    let dir = unique_dir("caption-font-size");
    let out = dir.join("scatter.svg");
    let mut chart = Chart::new(&df).caption("Caption text");
    let _ = chart.x("x").y("y").size(320, 240).scatter();
    chart
        .render_to_svg(&out)
        .expect("scatter render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");

    assert!(
        svg.contains("font-size=\"15\"") && svg.contains(">Caption text</text>"),
        "publication captions should render at the theme caption font size"
    );
}

#[test]
fn caption_is_drawn_after_axes() {
    let df = fixture_df();
    let dir = unique_dir("caption-after-axes");
    let out = dir.join("scatter.svg");
    let mut chart = Chart::new(&df).caption("Caption text").x_label("x axis");
    let _ = chart.x("x").y("y").size(320, 240).scatter();
    chart
        .render_to_svg(&out)
        .expect("scatter render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");

    let axis_pos = svg.find(">x axis</text>").expect("x-axis label");
    let caption_pos = svg.find(">Caption text</text>").expect("caption text");
    assert!(
        caption_pos > axis_pos,
        "caption should be emitted after axes so it cannot be overpainted"
    );
}

#[test]
fn scatter_hue_legend_preserves_observed_order() {
    let df = df!(
        "x" => [1.0_f64, 2.0, 3.0, 4.0, 5.0],
        "y" => [1.0_f64, 2.0, 1.5, 2.5, 3.0],
        "group" => ["high", "high", "low", "mid", "low"]
    )
    .unwrap();

    let dir = unique_dir("scatter-hue-observed-order");
    let out = dir.join("scatter.svg");
    let mut chart = Chart::new(&df).hue("group");
    let _ = chart.x("x").y("y").size(360, 260).scatter();
    chart
        .render_to_svg(&out)
        .expect("scatter render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");

    let high_pos = svg.find(">High</text>").expect("high legend label");
    let low_pos = svg.find(">Low</text>").expect("low legend label");
    let mid_pos = svg.find(">Mid</text>").expect("mid legend label");
    assert!(
        high_pos < low_pos && low_pos < mid_pos && !svg.contains("&quot;high&quot;"),
        "scatter legend should follow observed order without quoted labels"
    );
}

#[test]
fn scatter_legend_avoids_upper_right_point_cluster() {
    let df = df!(
        "x" => [8.0_f64, 8.4, 8.8, 9.2, 9.6, 1.0, 2.0, 3.0],
        "y" => [8.0_f64, 8.5, 8.2, 9.0, 9.4, 1.0, 2.0, 1.5],
        "group" => ["high", "high", "high", "high", "high", "low", "low", "mid"]
    )
    .unwrap();

    let dir = unique_dir("scatter-legend-avoids-upper-right");
    let out = dir.join("scatter.svg");
    let mut chart = Chart::new(&df).hue("group");
    let _ = chart.x("x").y("y").size(360, 260).scatter();
    chart
        .render_to_svg(&out)
        .expect("scatter render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");

    let legend_rect = parse_legend_rect(&svg);
    let (_, legend_y, _, _) = legend_rect;
    assert!(
        legend_y < 100.0,
        "scatter legend should reserve top space before falling back; SVG y={legend_y}"
    );
    assert_inside_legend_clear(
        &svg,
        panel_to_svg_rect(chart.figure_layout().panel_box, f64::from(chart.height())),
        "scatter",
    );
}

#[test]
fn regression_inside_legend_does_not_cover_graphics_or_axes() {
    let df = df!(
        "x" => [8.0_f64, 8.4, 8.8, 9.2, 9.6, 1.0, 2.0, 3.0],
        "y" => [8.0_f64, 8.5, 8.2, 9.0, 9.4, 1.0, 2.0, 1.5]
    )
    .unwrap();

    let dir = unique_dir("regression-inside-legend-clear");
    let out = dir.join("regression.svg");
    let mut chart = Chart::new(&df);
    let _ = chart
        .x("x")
        .y("y")
        .show_legend()
        .size(360, 260)
        .regression();
    chart
        .render_to_svg(&out)
        .expect("regression render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");

    assert_inside_legend_clear(
        &svg,
        panel_to_svg_rect(chart.figure_layout().panel_box, f64::from(chart.height())),
        "regression",
    );
}

#[test]
fn regression_does_not_draw_legend_by_default() {
    let df = df!(
        "x" => [1.0_f64, 2.0, 3.0, 4.0],
        "y" => [1.0_f64, 2.1, 2.9, 4.0]
    )
    .unwrap();

    let dir = unique_dir("regression-no-default-legend");
    let out = dir.join("regression.svg");
    let mut chart = Chart::new(&df);
    let _ = chart.x("x").y("y").size(360, 260).regression();
    chart
        .render_to_svg(&out)
        .expect("regression render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");

    assert!(
        !has_legend(&svg),
        "regression should not draw a legend by default"
    );
}

#[test]
fn mass_spectrum_inside_legend_does_not_cover_graphics_or_axes() {
    let df = df!(
        "mz" => [100.0_f64, 150.0, 200.0, 250.0, 300.0],
        "intensity" => [1.0_f64, 2.0, 10.0, 2.0, 1.0]
    )
    .unwrap();

    let dir = unique_dir("mass-spectrum-inside-legend-clear");
    let out = dir.join("mass_spectrum.svg");
    let mut chart = Chart::new(&df);
    let _ = chart
        .x("mz")
        .y("intensity")
        .add_peaks(vec![(200.0, 10.0)])
        .size(360, 260)
        .mass_spectrum();
    chart
        .render_to_svg(&out)
        .expect("mass spectrum render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");

    assert_inside_legend_clear(
        &svg,
        panel_to_svg_rect(chart.figure_layout().panel_box, f64::from(chart.height())),
        "mass spectrum",
    );
}

#[test]
fn horizontal_bar_inside_right_legend_does_not_cover_graphics_or_axes() {
    let df = df!(
        "group" => ["AA", "AA", "AB", "AB", "BB", "BB"],
        "series" => ["Observed", "Expected", "Observed", "Expected", "Observed", "Expected"],
        "count" => [10.0_f64, 8.0, 6.0, 7.0, 3.0, 4.0]
    )
    .unwrap();

    let dir = unique_dir("horizontal-bar-inside-legend-clear");
    let out = dir.join("horizontal_bar.svg");
    let mut chart = Chart::new(&df)
        .group("series")
        .hue("series")
        .bar_aggregation(BarAggregation::Identity);
    let _ = chart.x("group").y("count").size(420, 300).horizontal_bar();
    chart
        .render_to_svg(&out)
        .expect("horizontal bar render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");

    assert_inside_legend_clear(
        &svg,
        panel_to_svg_rect(chart.figure_layout().panel_box, f64::from(chart.height())),
        "horizontal bar",
    );
}

#[test]
fn horizontal_bar_long_y_tick_labels_do_not_overlap_y_axis_label() {
    let df = df!(
        "group" => ["locus_0 aa", "locus_0 Aa", "locus_0 AA"],
        "count" => [25.0_f64, 50.0, 25.0]
    )
    .unwrap();

    let dir = unique_dir("horizontal-bar-long-y-labels");
    let out = dir.join("horizontal_bar.svg");
    let mut chart = Chart::new(&df)
        .labels(Labels::raw())
        .bar_aggregation(BarAggregation::Identity)
        .y_label("Locus and genotype");
    let _ = chart.x("group").y("count").size(420, 300).horizontal_bar();
    chart
        .render_to_svg(&out)
        .expect("horizontal bar render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");
    let text = parse_text_elements(&svg);
    let axis_label = text_element(&text, "Locus and genotype");
    let tick_label = text_element(&text, "locus_0 aa");
    let tick_left =
        tick_label.x - deterministic_text_width(&tick_label.content, tick_label.font_size);
    let axis_right = axis_label.x + axis_label.font_size / 2.0;
    let axis_left = axis_label.x - axis_label.font_size / 2.0;
    let gap = tick_left - axis_right;

    assert!(
        axis_left >= 12.0,
        "rotated y-axis label should not touch canvas edge: axis_left={axis_left}"
    );
    assert!(
        (12.0..=22.0).contains(&gap),
        "rotated y-axis label should keep a compact gap from long y tick labels: gap={gap}"
    );
}

#[test]
fn bar_inside_legend_does_not_cover_graphics_or_axes() {
    let df = df!(
        "group" => ["AA", "AA", "AB", "AB", "BB", "BB"],
        "series" => ["Observed", "Expected", "Observed", "Expected", "Observed", "Expected"],
        "count" => [10.0_f64, 8.0, 6.0, 7.0, 3.0, 4.0]
    )
    .unwrap();

    let dir = unique_dir("bar-inside-legend-clear");
    let out = dir.join("bar.svg");
    let mut chart = Chart::new(&df)
        .group("series")
        .hue("series")
        .bar_aggregation(BarAggregation::Identity);
    let _ = chart.x("group").y("count").size(420, 300).bar();
    chart.render_to_svg(&out).expect("bar render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");

    assert_inside_legend_clear(
        &svg,
        panel_to_svg_rect(chart.figure_layout().panel_box, f64::from(chart.height())),
        "bar",
    );
}

#[test]
fn bar_without_hue_does_not_draw_legend() {
    let df = df!(
        "group" => ["A", "A", "B", "B"],
        "value" => [1.0_f64, 2.0, 4.0, 6.0]
    )
    .unwrap();

    let dir = unique_dir("bar-no-hue-no-legend");
    let out = dir.join("bar.svg");
    let mut chart = Chart::new(&df);
    let _ = chart.x("group").y("value").size(360, 260).bar();
    chart.render_to_svg(&out).expect("bar render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");

    assert!(
        !has_legend(&svg),
        "single-series bar should not draw a legend"
    );
}

#[test]
fn boxplot_inside_legend_does_not_cover_graphics_or_axes() {
    let df = df!(
        "dose" => ["low", "low", "low", "low", "high", "high", "high", "high"],
        "batch" => ["a", "a", "b", "b", "a", "a", "b", "b"],
        "value" => [1.0_f64, 1.2, 1.6, 1.8, 4.0, 4.2, 4.6, 4.8]
    )
    .unwrap();

    let dir = unique_dir("boxplot-inside-legend-clear");
    let out = dir.join("boxplot.svg");
    let mut chart = Chart::new(&df).group("batch").hue("batch");
    let _ = chart.x("dose").y("value").size(420, 300).boxplot();
    chart
        .render_to_svg(&out)
        .expect("boxplot render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");

    assert_inside_legend_clear(
        &svg,
        panel_to_svg_rect(chart.figure_layout().panel_box, f64::from(chart.height())),
        "boxplot",
    );
}

#[test]
fn violin_inside_legend_does_not_cover_graphics_or_axes() {
    let df = df!(
        "dose" => ["low", "low", "low", "low", "low", "low", "high", "high", "high", "high", "high", "high"],
        "batch" => ["a", "a", "a", "b", "b", "b", "a", "a", "a", "b", "b", "b"],
        "value" => [1.0_f64, 1.2, 1.4, 1.7, 1.9, 2.1, 4.0, 4.2, 4.4, 4.7, 4.9, 5.1]
    )
    .unwrap();

    let dir = unique_dir("violin-inside-legend-clear");
    let out = dir.join("violin.svg");
    let mut chart = Chart::new(&df).group("batch").hue("batch");
    let _ = chart.x("dose").y("value").size(420, 300).violin();
    chart
        .render_to_svg(&out)
        .expect("violin render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");

    assert_inside_legend_clear(
        &svg,
        panel_to_svg_rect(chart.figure_layout().panel_box, f64::from(chart.height())),
        "violin",
    );
}

#[test]
fn scatter_legend_prefers_top_when_clear() {
    let df = df!(
        "x" => [4.0_f64, 4.2, 4.4, 4.6, 4.8, 5.0],
        "y" => [4.0_f64, 4.2, 4.1, 4.3, 4.0, 4.2],
        "group" => ["high", "high", "low", "low", "mid", "mid"]
    )
    .unwrap();

    let dir = unique_dir("scatter-legend-prefers-top");
    let out = dir.join("scatter.svg");
    let mut chart = Chart::new(&df)
        .hue("group")
        .x_limits(0.0, 10.0)
        .y_limits(0.0, 10.0);
    let _ = chart.x("x").y("y").size(360, 260).scatter();
    chart
        .render_to_svg(&out)
        .expect("scatter render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");

    let (_, legend_y, _, _) = parse_legend_rect(&svg);
    assert!(
        legend_y < 100.0,
        "clear legends should prefer a top placement; SVG y={legend_y}"
    );
}

#[test]
fn inside_top_left_legend_position_is_honored() {
    let df = df!(
        "x" => [4.0_f64, 4.2, 4.4, 4.6, 4.8, 5.0],
        "y" => [4.0_f64, 4.2, 4.1, 4.3, 4.0, 4.2],
        "group" => ["high", "high", "low", "low", "mid", "mid"]
    )
    .unwrap();

    let dir = unique_dir("scatter-legend-top-left");
    let out = dir.join("scatter.svg");
    let mut chart = Chart::new(&df).hue("group");
    let _ = chart
        .legend_position(LegendPosition::InsideTopLeft)
        .x("x")
        .y("y")
        .size(360, 260)
        .scatter();
    chart
        .render_to_svg(&out)
        .expect("scatter render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");
    let (legend_x, _, _, _) = parse_legend_rect(&svg);
    let panel = chart.figure_layout().panel_box;

    assert!(
        legend_x < panel.x0 + 40.0,
        "inside top-left legend should align near the panel left edge: x={legend_x}, panel={panel:?}"
    );
    assert_inside_legend_clear(
        &svg,
        panel_to_svg_rect(panel, f64::from(chart.height())),
        "scatter top-left",
    );
}

#[test]
fn outside_right_bottom_legend_position_is_honored() {
    let df = df!(
        "x" => [4.0_f64, 4.2, 4.4, 4.6, 4.8, 5.0],
        "y" => [4.0_f64, 4.2, 4.1, 4.3, 4.0, 4.2],
        "group" => ["high", "high", "low", "low", "mid", "mid"]
    )
    .unwrap();

    let dir = unique_dir("scatter-legend-outside-right-bottom");
    let out = dir.join("scatter.svg");
    let mut chart = Chart::new(&df).hue("group");
    let _ = chart
        .legend_position(LegendPosition::OutsideRightBottom)
        .x("x")
        .y("y")
        .size(460, 300)
        .scatter();
    chart
        .render_to_svg(&out)
        .expect("scatter render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");
    let legend = parse_legend_svg_rect(&svg);
    let panel = panel_to_svg_rect(chart.figure_layout().panel_box, f64::from(chart.height()));

    assert!(
        legend.x > panel.x + panel.width,
        "outside-right legend should be to the right of the axes: legend={legend:?}, panel={panel:?}"
    );
    assert!(
        (legend.y + legend.height) > panel.y + panel.height - 30.0,
        "outside-right-bottom legend should align near panel bottom: legend={legend:?}, panel={panel:?}"
    );
}

#[test]
fn top_scatter_legend_does_not_expand_lower_y_range() {
    let df = df!(
        "x" => [8.0_f64, 8.4, 8.8, 9.2, 9.6, 1.0, 2.0, 3.0],
        "y" => [8.0_f64, 8.5, 8.2, 9.0, 9.4, 1.0, 2.0, 1.5],
        "group" => ["high", "high", "high", "high", "high", "low", "low", "mid"]
    )
    .unwrap();

    let dir = unique_dir("scatter-top-legend-no-lower-expand");
    let out = dir.join("scatter.svg");
    let mut chart = Chart::new(&df).hue("group");
    let _ = chart.x("x").y("y").size(360, 260).scatter();
    chart
        .render_to_svg(&out)
        .expect("scatter render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");

    assert!(
        !svg.contains(">-5</text>") && !svg.contains(">-10</text>"),
        "top legend should not reserve bottom data range"
    );
}

#[test]
fn scatter_legend_entry_font_is_readable() {
    let df = df!(
        "x" => [1.0_f64, 2.0, 3.0],
        "y" => [1.0_f64, 2.0, 3.0],
        "group" => ["high", "low", "mid"]
    )
    .unwrap();

    let dir = unique_dir("scatter-legend-readable-font");
    let out = dir.join("scatter.svg");
    let mut chart = Chart::new(&df).hue("group");
    let _ = chart.x("x").y("y").size(640, 480).scatter();
    chart
        .render_to_svg(&out)
        .expect("scatter render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");

    assert!(
        svg.contains("font-size=\"11\"") && svg.contains(">High</text>"),
        "legend entries should use at least an 11px font at 640x480"
    );
}

#[test]
fn publication_theme_uses_larger_default_text() {
    let palette = Theme::Publication.palette();

    assert!(
        palette.tick_font_size >= 16
            && palette.label_font_size >= 17
            && palette.title_font_size >= 20,
        "publication typography should be readable by default: {palette:?}"
    );
}

#[test]
fn scatter_auto_range_pads_data_from_frame() {
    let df = df!(
        "x" => [0.0_f64, 10.0],
        "y" => [0.0_f64, 10.0]
    )
    .unwrap();

    let dir = unique_dir("scatter-auto-range-padding");
    let out = dir.join("scatter.svg");
    let mut chart = Chart::new(&df);
    let _ = chart.x("x").y("y").size(360, 260).scatter();
    let panel = chart.figure_layout().panel_box;
    chart
        .render_to_svg(&out)
        .expect("scatter render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");
    let circles = parse_circles(&svg);
    let (min_x, max_x, min_y, max_y) = circle_bounds(&circles);
    let top = 260.0 - panel.y1;
    let bottom = 260.0 - panel.y0;

    assert!(
        min_x > panel.x0 + 2.0
            && max_x < panel.x1 - 2.0
            && min_y > top + 2.0
            && max_y < bottom - 2.0,
        "auto-scaled scatter points should be padded from frame: circles={circles:?}, panel={panel:?}"
    );
}

#[test]
fn scatter_explicit_limits_are_not_padded() {
    let df = df!(
        "x" => [0.0_f64, 10.0],
        "y" => [0.0_f64, 10.0]
    )
    .unwrap();

    let dir = unique_dir("scatter-explicit-limits-no-padding");
    let out = dir.join("scatter.svg");
    let mut chart = Chart::new(&df).x_limits(0.0, 10.0).y_limits(0.0, 10.0);
    let _ = chart.x("x").y("y").size(360, 260).scatter();
    let panel = chart.figure_layout().panel_box;
    chart
        .render_to_svg(&out)
        .expect("scatter render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");
    let circles = parse_circles(&svg);
    let (min_x, max_x, min_y, max_y) = circle_bounds(&circles);
    let top = 260.0 - panel.y1;
    let bottom = 260.0 - panel.y0;

    assert!(
        (min_x - panel.x0).abs() < 1.0
            && (max_x - panel.x1).abs() < 1.0
            && (min_y - top).abs() < 1.0
            && (max_y - bottom).abs() < 1.0,
        "explicit scatter limits should remain exact: circles={circles:?}, panel={panel:?}"
    );
}

#[test]
fn regression_auto_range_pads_data_from_frame() {
    let df = df!(
        "x" => [0.0_f64, 5.0, 10.0],
        "y" => [0.0_f64, 5.0, 10.0]
    )
    .unwrap();

    let dir = unique_dir("regression-auto-range-padding");
    let out = dir.join("regression.svg");
    let mut chart = Chart::new(&df);
    let _ = chart.x("x").y("y").size(360, 260).regression();
    let panel = chart.figure_layout().panel_box;
    chart
        .render_to_svg(&out)
        .expect("regression render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");
    let circles = parse_circles(&svg);
    let (min_x, max_x, min_y, max_y) = circle_bounds(&circles);
    let top = 260.0 - panel.y1;
    let bottom = 260.0 - panel.y0;

    assert!(
        min_x > panel.x0 + 2.0
            && max_x < panel.x1 - 2.0
            && min_y > top + 2.0
            && max_y < bottom - 2.0,
        "auto-scaled regression points should be padded from frame: circles={circles:?}, panel={panel:?}"
    );
}

#[test]
fn regression_explicit_limits_are_not_padded() {
    let df = df!(
        "x" => [0.0_f64, 5.0, 10.0],
        "y" => [0.0_f64, 5.0, 10.0]
    )
    .unwrap();

    let dir = unique_dir("regression-explicit-limits-no-padding");
    let out = dir.join("regression.svg");
    let mut chart = Chart::new(&df).x_limits(0.0, 10.0).y_limits(0.0, 10.0);
    let _ = chart.x("x").y("y").size(360, 260).regression();
    let panel = chart.figure_layout().panel_box;
    chart
        .render_to_svg(&out)
        .expect("regression render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");
    let circles = parse_circles(&svg);
    let (min_x, max_x, min_y, max_y) = circle_bounds(&circles);
    let clip = parse_first_clip_rect(&svg);

    assert!(
        (min_x - clip.x).abs() < 1.0
            && (max_x - (clip.x + clip.width)).abs() < 1.0
            && (min_y - clip.y).abs() < 1.0
            && (max_y - (clip.y + clip.height)).abs() < 1.0,
        "explicit regression limits should remain exact: circles={circles:?}, clip={clip:?}, panel={panel:?}"
    );
}
