use super::*;

fn estimate_line(frame: &DataFrame) -> Plot<lichtung_plot::kind::Line> {
    Plot::line(data::numeric(frame, "x"), data::numeric(frame, "estimate"))
        .group(data::category(frame, "trace"))
        .hue(data::category(frame, "trace"))
}

fn clip_rects(svg: &str) -> Vec<(f64, f64, f64, f64)> {
    svg.lines()
        .filter(|line| line.starts_with("<clipPath "))
        .map(|line| {
            (
                attribute(line, "x").parse().unwrap(),
                attribute(line, "y").parse().unwrap(),
                attribute(line, "width").parse().unwrap(),
                attribute(line, "height").parse().unwrap(),
            )
        })
        .collect()
}

#[test]
fn shared_legend_is_global_complete_and_semantically_validated() {
    let first = df!(
        "x" => [1.0_f64, 2.0], "y" => [1.0_f64, 2.0], "trace" => ["control", "control"]
    )
    .unwrap();
    let second = df!(
        "x" => [1.0_f64, 2.0], "y" => [2.0_f64, 3.0],
        "trace" => ["active", "active"], "other" => ["active", "active"]
    )
    .unwrap();
    let line = |frame: &DataFrame, hue| {
        Plot::line(data::numeric(frame, "x"), data::numeric(frame, "y"))
            .hue(data::category(frame, hue))
    };
    let svg = render_canvas(
        Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, line(&first, "trace"))
            .unwrap()
            .panel(0, 1, line(&second, "trace"))
            .unwrap()
            .legend_policy(CanvasLegendPolicy::Shared),
        "shared-global-legend",
    )
    .unwrap();
    assert_eq!(svg.matches("id=\"figure-shared-legend\"").count(), 1);
    assert!(svg.contains(">Control</text>") && svg.contains(">Active</text>"));
    assert_unique_svg_ids(&svg);
    assert_grid_layers_before_marks(&svg, &["panel-0-grid", "panel-1-grid"]);

    let error = Canvas::grid(1, 2)
        .unwrap()
        .panel(0, 0, line(&first, "trace"))
        .unwrap()
        .panel(0, 1, line(&second, "other"))
        .unwrap()
        .legend_policy(CanvasLegendPolicy::Shared)
        .preflight()
        .unwrap_err();
    assert!(error.to_string().contains("compatible semantic titles"));
}

#[test]
fn shared_legend_gap_reduces_canvas_legend_separation() {
    let frame = df!(
        "x" => [1.0_f64, 2.0], "y" => [1.0_f64, 2.0], "trace" => ["control", "control"]
    )
    .unwrap();
    let render = |gap: Option<f64>| {
        let canvas = Canvas::grid(1, 1)
            .unwrap()
            .panel(
                0,
                0,
                Plot::line(data::numeric(&frame, "x"), data::numeric(&frame, "y"))
                    .hue(data::category(&frame, "trace")),
            )
            .unwrap()
            .legend_policy(CanvasLegendPolicy::Shared);
        let canvas = match gap {
            Some(gap) => canvas.shared_legend_gap(gap).unwrap(),
            None => canvas,
        };
        render_canvas(canvas, "shared-legend-gap").unwrap()
    };
    let legend_x = |svg: &str| {
        let layer = svg.split("<g id=\"figure-shared-legend\">").nth(1).unwrap();
        let rect = layer
            .lines()
            .find(|line| line.starts_with("<rect "))
            .unwrap();
        attribute(rect, "x").parse::<f64>().unwrap()
    };
    let panel_right = |svg: &str| {
        let clip = svg
            .lines()
            .find(|line| line.starts_with("<clipPath "))
            .unwrap();
        attribute(clip, "x").parse::<f64>().unwrap()
            + attribute(clip, "width").parse::<f64>().unwrap()
    };
    let default = render(None);
    let compact = render(Some(18.0));
    assert!(
        legend_x(&compact) - panel_right(&compact) < legend_x(&default) - panel_right(&default)
    );
    assert!(Canvas::grid(1, 1).unwrap().shared_legend_gap(-1.0).is_err());
}

#[test]
fn per_plot_legends_keep_locally_observed_hue_levels() {
    let first = df!(
        "x" => [1.0_f64, 2.0], "y" => [1.0_f64, 2.0], "trace" => ["control", "control"]
    )
    .unwrap();
    let second = df!(
        "x" => [1.0_f64, 2.0], "y" => [2.0_f64, 3.0], "trace" => ["active", "active"]
    )
    .unwrap();
    let line = |frame: &DataFrame| {
        Plot::line(data::numeric(frame, "x"), data::numeric(frame, "y"))
            .hue(data::category(frame, "trace"))
    };
    let svg = render_canvas(
        Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, line(&first))
            .unwrap()
            .panel(0, 1, line(&second))
            .unwrap()
            .legend_policy(CanvasLegendPolicy::PerPlot),
        "per-plot-local-legends",
    )
    .unwrap();
    assert_eq!(svg.matches(">Control</text>").count(), 1);
    assert_eq!(svg.matches(">Active</text>").count(), 1);
    assert!(svg.contains("id=\"panel-0-legend\"") && svg.contains("id=\"panel-1-legend\""));
}

#[test]
fn composed_offset_panel_reserves_long_y_tick_labels_locally() {
    let line_frame = estimate_data();
    let bar_frame = df!(
        "category" => ["a very long category label in the second panel", "short"],
        "value" => [2.0_f64, 4.0]
    )
    .unwrap();
    let svg = render_canvas(
        Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, estimate_line(&line_frame))
            .unwrap()
            .panel(
                0,
                1,
                Plot::horizontal_bar(
                    data::category(&bar_frame, "category"),
                    BarInput::estimates(data::numeric(&bar_frame, "value")),
                ),
            )
            .unwrap(),
        "composed-local-gutter",
    )
    .unwrap();
    let clips = clip_rects(&svg);
    assert_eq!(clips.len(), 2);
    assert!(clips[1].0 > clips[0].0 + clips[0].2 + 40.0, "{clips:?}");
}

#[test]
fn composed_right_panel_keeps_y_axis_title_out_of_the_left_panel() {
    let line_frame = estimate_data();
    let bar_frame = df!(
        "category" => ["3D-SART Understanding", "short"], "value" => [2.0_f64, 4.0]
    )
    .unwrap();
    let svg = render_canvas(
        Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, estimate_line(&line_frame))
            .unwrap()
            .panel(
                0,
                1,
                Plot::horizontal_bar(
                    data::category(&bar_frame, "category"),
                    BarInput::estimates(data::numeric(&bar_frame, "value")),
                )
                .y_label("Outcome"),
            )
            .unwrap(),
        "composed-right-title",
    )
    .unwrap();
    let outcome = svg
        .lines()
        .find(|line| line.contains(">Outcome</text>"))
        .unwrap();
    let clips = clip_rects(&svg);
    assert!(attribute(outcome, "x").parse::<f64>().unwrap() > clips[0].0 + clips[0].2);
}

#[test]
fn multiline_y_axis_title_keeps_every_line_inside_the_canvas() {
    let frame = df!("category" => ["first", "second"], "value" => [2.0_f64, 4.0]).unwrap();
    let svg = render_canvas(
        Canvas::grid(1, 1)
            .unwrap()
            .panel(
                0,
                0,
                Plot::horizontal_bar(
                    data::category(&frame, "category"),
                    BarInput::estimates(data::numeric(&frame, "value")),
                )
                .y_label("Linguistic\nproperty\ncontribution"),
            )
            .unwrap(),
        "multiline-y-title",
    )
    .unwrap();
    for text in ["Linguistic", "property", "contribution"] {
        let line = svg
            .lines()
            .find(|line| line.contains(&format!(">{text}</text>")))
            .unwrap();
        assert!(attribute(line, "x").parse::<f64>().unwrap() >= 12.0);
    }
}

#[test]
fn composed_panel_reserves_categorical_y_axis_text_once() {
    let category = "category label measured exactly once";
    let frame = df!("category" => [category, "short"], "value" => [2.0_f64, 4.0]).unwrap();
    let svg = render_canvas(
        Canvas::grid(1, 1)
            .unwrap()
            .panel(
                0,
                0,
                Plot::horizontal_bar(
                    data::category(&frame, "category"),
                    BarInput::estimates(data::numeric(&frame, "value")),
                ),
            )
            .unwrap(),
        "category-gutter-once",
    )
    .unwrap();
    let label_width = lichtung_plot::layout::deterministic_text_width(category, 12.0);
    assert!(clip_rects(&svg)[0].0 < label_width * 1.75);
}

#[test]
fn composed_horizontal_bars_support_shared_headers_labels_and_range_groups() {
    let left = df!("outcome" => ["alpha", "beta"], "value" => [1.0_f64, -0.5]).unwrap();
    let right = df!("outcome" => ["alpha", "beta"], "value" => [8.0_f64, -6.0]).unwrap();
    let bars = |frame: &DataFrame| {
        Plot::horizontal_bar(
            data::category(frame, "outcome"),
            BarInput::estimates(data::numeric(frame, "value")),
        )
    };
    let svg = render_canvas(
        Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, bars(&left))
            .unwrap()
            .panel(0, 1, bars(&right))
            .unwrap()
            .column_header(0, 1, "Correlation")
            .unwrap()
            .column_header(1, 1, "Regression")
            .unwrap()
            .row_header(0, 1, "Outcome")
            .unwrap()
            .category_labels(CanvasCategoryLabels::FirstColumnPerRow)
            .x_range_group([(0, 0), (0, 1)])
            .unwrap(),
        "shared-headers",
    )
    .unwrap();
    assert_eq!(svg.matches(">Alpha</text>").count(), 1);
    assert_eq!(svg.matches(">Beta</text>").count(), 1);
    for text in ["Correlation", "Regression", "Outcome"] {
        assert!(svg.contains(&format!(">{text}</text>")));
    }
    let headers = svg
        .split("<g id=\"figure-shared-headers\">")
        .nth(1)
        .unwrap()
        .split("</g>")
        .next()
        .unwrap();
    assert!(!headers.contains("font-weight="));
}

#[test]
fn composed_row_title_spans_all_columns_and_rejects_panel_titles() {
    let frame = df!("x" => [1.0_f64, 2.0], "y" => [2.0_f64, 3.0]).unwrap();
    let line = || Plot::line(data::numeric(&frame, "x"), data::numeric(&frame, "y"));
    let svg = render_canvas(
        Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, line())
            .unwrap()
            .panel(0, 1, line())
            .unwrap()
            .row_title(0, "Shared outcome")
            .unwrap(),
        "row-title",
    )
    .unwrap();
    assert_eq!(svg.matches(">Shared outcome</text>").count(), 1);
    assert_eq!(svg.matches("id=\"figure-row-titles\"").count(), 1);
    let error = Canvas::grid(1, 1)
        .unwrap()
        .panel(0, 0, line().title("Panel title"))
        .unwrap()
        .row_title(0, "Shared outcome")
        .unwrap()
        .preflight()
        .unwrap_err();
    assert!(error
        .to_string()
        .contains("cannot coexist with a panel title or subtitle"));
}

#[test]
fn shared_category_labels_reject_different_row_domains_preflight() {
    let left = df!("outcome" => ["alpha", "beta"], "value" => [1.0_f64, 2.0]).unwrap();
    let right = df!("outcome" => ["alpha", "gamma"], "value" => [1.0_f64, 2.0]).unwrap();
    let bar = |frame: &DataFrame| {
        Plot::horizontal_bar(
            data::category(frame, "outcome"),
            BarInput::estimates(data::numeric(frame, "value")),
        )
    };
    let error = Canvas::grid(1, 2)
        .unwrap()
        .panel(0, 0, bar(&left))
        .unwrap()
        .panel(0, 1, bar(&right))
        .unwrap()
        .category_labels(CanvasCategoryLabels::FirstColumnPerRow)
        .preflight()
        .unwrap_err();
    assert!(error.to_string().contains("category domains differ"));
}

#[test]
fn shared_category_labels_compare_rendered_explicit_orders() {
    let left = df!("outcome" => ["alpha", "beta"], "value" => [1.0_f64, 2.0]).unwrap();
    let reversed = df!("outcome" => ["beta", "alpha"], "value" => [2.0_f64, 1.0]).unwrap();
    let bar = |frame: &DataFrame, order| {
        Plot::horizontal_bar(
            data::category(frame, "outcome"),
            BarInput::estimates(data::numeric(frame, "value")),
        )
        .category_order(CategoryOrder::explicit(order))
    };

    Canvas::grid(1, 2)
        .unwrap()
        .panel(0, 0, bar(&left, ["alpha", "beta"]))
        .unwrap()
        .panel(0, 1, bar(&reversed, ["alpha", "beta"]))
        .unwrap()
        .category_labels(CanvasCategoryLabels::FirstColumnPerRow)
        .preflight()
        .unwrap();

    let error = Canvas::grid(1, 2)
        .unwrap()
        .panel(0, 0, bar(&left, ["alpha", "beta"]))
        .unwrap()
        .panel(0, 1, bar(&left, ["beta", "alpha"]))
        .unwrap()
        .category_labels(CanvasCategoryLabels::FirstColumnPerRow)
        .preflight()
        .unwrap_err();
    assert!(error.to_string().contains("category domains differ"));
}

#[test]
fn composed_svg_is_byte_deterministic() {
    let build = || {
        let frame = estimate_data();
        Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, estimate_line(&frame))
            .unwrap()
            .panel(
                0,
                1,
                Plot::forest(
                    data::category(&frame, "term"),
                    data::numeric(&frame, "estimate"),
                    Bounds::new(
                        data::numeric(&frame, "lower"),
                        data::numeric(&frame, "upper"),
                    ),
                    0.0,
                ),
            )
            .unwrap()
            .title("Composed in-memory parity")
    };
    assert_eq!(
        render_canvas(build(), "canvas-bytes-first").unwrap(),
        render_canvas(build(), "canvas-bytes-second").unwrap()
    );
}

#[test]
fn composed_panel_titles_and_subtitles_keep_distinct_measured_positions() {
    let frame = df!("x" => [1.0_f64, 2.0], "y" => [2.0_f64, 3.0]).unwrap();
    let svg = render_canvas(
        Canvas::grid(1, 1)
            .unwrap()
            .panel(
                0,
                0,
                Plot::line(data::numeric(&frame, "x"), data::numeric(&frame, "y"))
                    .title("(a) Panel")
                    .subtitle("Panel subtitle"),
            )
            .unwrap(),
        "panel-title-subtitle",
    )
    .unwrap();
    let title = svg
        .lines()
        .find(|line| line.contains(">(a) Panel</text>"))
        .unwrap();
    let subtitle = svg
        .lines()
        .find(|line| line.contains(">Panel subtitle</text>"))
        .unwrap();
    let panel_y = clip_rects(&svg)[0].1;
    let title_y = attribute(title, "y").parse::<f64>().unwrap();
    let subtitle_y = attribute(subtitle, "y").parse::<f64>().unwrap();
    assert_eq!(attribute(title, "text-anchor"), "start");
    assert_eq!(attribute(title, "font-size"), "20");
    assert_eq!(attribute(subtitle, "font-size"), "15");
    assert!(subtitle_y - title_y > 15.5 && subtitle_y + 7.5 < panel_y);
}

#[test]
fn shared_canvas_scatter_legend_rejects_null_hue_rows_preflight() {
    let left = df!(
        "x" => [1.0_f64, 2.0], "y" => [1.0_f64, 2.0], "trace" => [Some("control"), None]
    )
    .unwrap();
    let right = df!(
        "x" => [1.0_f64, 2.0, 3.0], "y" => [2.0_f64, 3.0, 4.0],
        "trace" => [Some("active"), Some("active"), None]
    )
    .unwrap();
    let scatter = |frame: &DataFrame| {
        Plot::scatter(data::numeric(frame, "x"), data::numeric(frame, "y"))
            .hue(data::category(frame, "trace"))
    };
    let error = Canvas::grid(1, 2)
        .unwrap()
        .panel(0, 0, scatter(&left))
        .unwrap()
        .panel(0, 1, scatter(&right))
        .unwrap()
        .legend_policy(CanvasLegendPolicy::Shared)
        .preflight()
        .unwrap_err();
    assert!(error
        .to_string()
        .contains("value \"trace\" contains null at source row 1"));
}

#[test]
fn all_canvas_grids_precede_any_marks() {
    let frame = estimate_data();
    let svg = render_canvas(
        Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, estimate_line(&frame))
            .unwrap()
            .panel(
                0,
                1,
                Plot::scatter(
                    data::numeric(&frame, "x"),
                    data::numeric(&frame, "estimate"),
                ),
            )
            .unwrap(),
        "grid-before-marks",
    )
    .unwrap();
    assert_grid_layers_before_marks(&svg, &["panel-0-grid", "panel-1-grid"]);
}

#[test]
fn heterogeneous_canvas_is_deterministic_and_clipped_once_per_plot() {
    let build = || {
        let frame = estimate_data();
        Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, estimate_line(&frame))
            .unwrap()
            .panel(
                0,
                1,
                Plot::coefficient(
                    data::category(&frame, "term"),
                    CoefficientInput::estimates_with_intervals(
                        data::numeric(&frame, "estimate"),
                        Bounds::new(
                            data::numeric(&frame, "lower"),
                            data::numeric(&frame, "upper"),
                        ),
                    ),
                ),
            )
            .unwrap()
            .legend_policy(CanvasLegendPolicy::PerPlot)
            .title("Model summary")
    };
    let first = render_canvas(build(), "heterogeneous-first").unwrap();
    let second = render_canvas(build(), "heterogeneous-second").unwrap();
    assert_eq!(first, second);
    assert_eq!(first.matches("<svg ").count(), 1);
    assert_eq!(first.matches("<clipPath id=").count(), 2);
    assert!(first.contains(">Model summary</text>"));
}

#[test]
fn invalid_canvas_layouts_and_panel_data_fail_before_artifact_creation() {
    assert!(Canvas::grid(0, 1).is_err());
    let frame = estimate_data();
    assert!(Canvas::grid(1, 1)
        .unwrap()
        .panel(0, 0, estimate_line(&frame))
        .unwrap()
        .size(100, 100)
        .preflight()
        .is_err());

    let invalid = Plot::scatter(
        lichtung_plot::NumericValues::new("x", [1.0, f64::INFINITY]),
        lichtung_plot::NumericValues::new("y", [2.0, 3.0]),
    );
    let error = Canvas::grid(1, 1)
        .unwrap()
        .panel(0, 0, invalid)
        .unwrap()
        .preflight()
        .unwrap_err();
    assert!(error.to_string().contains("source row 1"));
}

#[test]
fn invalid_panel_band_and_facet_cell_fail_canvas_preflight() {
    let frame = estimate_data();
    let bar = Plot::bar(
        data::category(&frame, "factor"),
        BarInput::samples(data::numeric(&frame, "estimate")),
    )
    .significance([
        Significance::between("before", "after", SignificanceMark::PValue(0.01))
            .facet(FacetCell::single("one")),
    ]);
    let error = Canvas::grid(1, 1)
        .unwrap()
        .panel(0, 0, bar)
        .unwrap()
        .preflight()
        .unwrap_err();
    assert!(error
        .to_string()
        .contains("facet-owned significance annotations require"));
}
