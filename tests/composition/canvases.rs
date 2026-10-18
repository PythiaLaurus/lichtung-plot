use super::*;

fn estimate_line(data: &EstimateData) -> Plot<lichtung_plot::kind::Line> {
    Plot::line(data.x.clone(), data.estimate.clone())
        .group(data.trace.clone())
        .hue(data.trace.clone())
}

fn hued_line(
    x: &NumericValues,
    y: &NumericValues,
    hue: &CategoryValues,
) -> Plot<lichtung_plot::kind::Line> {
    Plot::line(x.clone(), y.clone()).hue(hue.clone())
}

fn grouped_hued_line(
    x: &NumericValues,
    y: &NumericValues,
    group: &CategoryValues,
    hue: &CategoryValues,
) -> Plot<lichtung_plot::kind::Line> {
    Plot::line(x.clone(), y.clone())
        .group(group.clone())
        .hue(hue.clone())
}

fn horizontal_bar(
    category: &CategoryValues,
    value: &NumericValues,
) -> Plot<lichtung_plot::kind::HorizontalBar> {
    Plot::horizontal_bar(category.clone(), BarInput::estimates(value.clone()))
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
fn text_panel_content_stays_top_left_when_its_fill_cell_grows() {
    let render = |width, height| {
        render_canvas(
            Canvas::grid(1, 1)
                .unwrap()
                .size(width, height)
                .text_panel(
                    0,
                    0,
                    lichtung_plot::TextPanel::new("Note", "Short body").unwrap(),
                )
                .unwrap(),
            "text-panel-top-left",
        )
        .unwrap()
    };
    let positions = |svg: &str| {
        let doc = roxmltree::Document::parse(svg).unwrap();
        ["Note", "Short body"].map(|source| {
            let node = doc
                .descendants()
                .find(|node| node.has_tag_name("text") && node.text() == Some(source))
                .unwrap();
            (
                node.attribute("x").unwrap().parse::<f64>().unwrap(),
                node.attribute("y").unwrap().parse::<f64>().unwrap(),
            )
        })
    };
    let compact = render(400, 260);
    let expanded = render(700, 560);
    let [title, body] = positions(&compact);
    let [large_title, large_body] = positions(&expanded);
    assert!((title.0 - large_title.0).abs() <= 1.0);
    assert!((title.1 - large_title.1).abs() <= 1.0);
    assert!((body.0 - large_body.0).abs() <= 1.0);
    assert!((body.1 - large_body.1).abs() <= 1.0);
    assert!(body.1 > title.1);
}

#[test]
fn plot_local_and_canvas_captions_coexist_in_their_own_spans() {
    let svg = render_canvas(
        Canvas::grid(1, 2)
            .unwrap()
            .size(1101, 701)
            .theme(Theme::nature())
            .panel(
                0,
                0,
                Plot::scatter(
                    NumericValues::new("x", [0.0, 1.0]),
                    NumericValues::new("y", [1.0, 2.0]),
                )
                .theme(Theme::science())
                .title("Independent Plot title")
                .subtitle("Independent Plot subtitle")
                .caption("Local Plot caption"),
            )
            .unwrap()
            .text_panel(
                0,
                1,
                lichtung_plot::TextPanel::new("Context", "Neighbor body").unwrap(),
            )
            .unwrap()
            .title("Canvas root title")
            .caption("Canvas root caption"),
        "plot-local-and-canvas-captions",
    )
    .unwrap();
    let doc = roxmltree::Document::parse(&svg).unwrap();
    for source in [
        "Independent Plot title",
        "Independent Plot subtitle",
        "Local Plot caption",
        "Canvas root title",
        "Canvas root caption",
    ] {
        assert_eq!(
            doc.descendants()
                .filter(|node| node.has_tag_name("text") && node.text() == Some(source))
                .count(),
            1,
            "{source} must be drawn exactly once"
        );
    }
    let local = doc
        .descendants()
        .find(|node| node.attribute("id") == Some("panel-0-title"))
        .unwrap();
    assert_eq!(
        local
            .descendants()
            .filter(|node| node.has_tag_name("text"))
            .count(),
        3
    );
    let local_caption = local
        .descendants()
        .find(|node| node.text() == Some("Local Plot caption"))
        .unwrap();
    let outer_caption = doc
        .descendants()
        .find(|node| node.text() == Some("Canvas root caption"))
        .unwrap();
    let caption_y = local_caption
        .attribute("y")
        .unwrap()
        .parse::<f64>()
        .unwrap();
    let root_y = outer_caption
        .attribute("y")
        .unwrap()
        .parse::<f64>()
        .unwrap();
    assert!(
        caption_y < root_y,
        "local caption stays within its cell, above the root caption"
    );
    let plot_clip = doc
        .descendants()
        .find(|node| node.attribute("id") == Some("clip-0"))
        .unwrap();
    let bounds = plot_clip
        .descendants()
        .find(|node| node.has_tag_name("rect"))
        .unwrap();
    let data_bottom = bounds.attribute("y").unwrap().parse::<f64>().unwrap()
        + bounds.attribute("height").unwrap().parse::<f64>().unwrap();
    assert!(
        caption_y > data_bottom,
        "local caption clears the data projection"
    );
    assert_unique_svg_ids(&svg);
}

#[test]
fn narrow_spanning_canvas_header_wraps_completely_above_its_cells() {
    let heading = "Measurement reporting period and independent longitudinal reference";
    let svg = render_canvas(
        Canvas::grid(1, 1)
            .unwrap()
            .size(220, 400)
            .column_header(0, 1, heading)
            .unwrap()
            .text_panel(
                0,
                0,
                lichtung_plot::TextPanel::new("Content", "Body").unwrap(),
            )
            .unwrap(),
        "wrapped-canvas-header",
    )
    .unwrap();
    let document = roxmltree::Document::parse(&svg).unwrap();
    let header = document
        .descendants()
        .find(|node| node.attribute("id") == Some("canvas-shared-headers"))
        .unwrap();
    let words = header
        .descendants()
        .filter(|node| node.has_tag_name("text"))
        .filter_map(|node| node.text())
        .flat_map(str::split_whitespace)
        .collect::<Vec<_>>();
    assert_eq!(words, heading.split_whitespace().collect::<Vec<_>>());
    let body = document
        .descendants()
        .find(|node| node.has_tag_name("text") && node.text() == Some("Content"))
        .unwrap();
    let body_y = body.attribute("y").unwrap().parse::<f64>().unwrap();
    let header_y = header
        .descendants()
        .filter(|node| node.has_tag_name("text"))
        .filter_map(|node| node.attribute("y"))
        .map(|value| value.parse::<f64>().unwrap())
        .fold(0.0_f64, f64::max);
    assert!(
        header_y < body_y,
        "wrapped header must clear its assigned cell"
    );
}

#[test]
fn continuous_shared_heatmaps_draw_one_merged_guide_without_local_colorbars() {
    let plot = |values| {
        Plot::heatmap(
            CategoryValues::new("column", ["one", "two"]).unwrap(),
            CategoryValues::new("row", ["only", "only"]).unwrap(),
            NumericValues::new("fill", values),
        )
    };
    let svg = render_canvas(
        Canvas::grid(1, 2)
            .unwrap()
            .size(1200, 640)
            .panel(0, 0, plot([0.0, 2.0]))
            .unwrap()
            .panel(0, 1, plot([1.0, 5.0]))
            .unwrap()
            .legend_policy(CanvasLegendPolicy::Shared),
        "shared-continuous-heatmaps",
    )
    .unwrap();
    assert_eq!(svg.matches("<linearGradient ").count(), 1);
    let doc = roxmltree::Document::parse(&svg).unwrap();
    let gradient = doc
        .descendants()
        .find(|node| node.has_tag_name("linearGradient"))
        .unwrap();
    assert_eq!(gradient.attribute("x1"), gradient.attribute("x2"));
    assert_ne!(gradient.attribute("y1"), gradient.attribute("y2"));
    assert_eq!(
        gradient
            .children()
            .filter(|node| node.has_tag_name("stop"))
            .map(|node| node.attribute("stop-color").unwrap())
            .collect::<Vec<_>>(),
        ["#F7FBFF", "#C4D0E1", "#92A6C4", "#08306B"]
    );
    assert_eq!(
        gradient
            .children()
            .filter(|node| node.has_tag_name("stop"))
            .map(|node| node.attribute("offset").unwrap())
            .collect::<Vec<_>>(),
        ["0.000000", "0.200000", "0.400000", "1.000000"]
    );
    let guide = doc
        .descendants()
        .find(|node| node.attribute("id") == Some("canvas-shared-legend"))
        .unwrap();
    let texts = guide
        .descendants()
        .filter(|node| node.has_tag_name("text"))
        .filter_map(|node| node.text())
        .collect::<Vec<_>>();
    assert_eq!(texts, ["Fill", "0", "5"]);
    assert_unique_svg_ids(&svg);
}

#[test]
fn shared_beeswarm_guide_has_one_gradient_and_final_domain_labels() {
    let plot = |values| {
        Plot::beeswarm(
            CategoryValues::new("row", ["one", "one"]).unwrap(),
            NumericValues::new("x", [0.0, 1.0]),
            NumericValues::new("fill", values),
        )
    };
    let svg = render_canvas(
        Canvas::grid(1, 2)
            .unwrap()
            .size(1200, 640)
            .panel(0, 0, plot([0.0, 1.0]))
            .unwrap()
            .panel(0, 1, plot([1.0, 5.0]))
            .unwrap()
            .legend_policy(CanvasLegendPolicy::Shared),
        "shared-continuous-beeswarm",
    )
    .unwrap();
    assert_eq!(svg.matches("<linearGradient ").count(), 1);
    let doc = roxmltree::Document::parse(&svg).unwrap();
    let guide = doc
        .descendants()
        .find(|node| node.attribute("id") == Some("canvas-shared-legend"))
        .unwrap();
    let texts = guide
        .descendants()
        .filter(|node| node.has_tag_name("text"))
        .filter_map(|node| node.text())
        .collect::<Vec<_>>();
    assert_eq!(texts, ["Fill", "0", "5"]);
    assert_unique_svg_ids(&svg);
}

#[test]
fn constant_shared_beeswarm_has_one_midpoint_label_and_gradient() {
    let plot = || {
        Plot::beeswarm(
            CategoryValues::new("row", ["one", "one"]).unwrap(),
            NumericValues::new("x", [0.0, 1.0]),
            NumericValues::new("fill", [1.0, 1.0]),
        )
    };
    let svg = render_canvas(
        Canvas::grid(1, 2)
            .unwrap()
            .size(1200, 640)
            .panel(0, 0, plot())
            .unwrap()
            .panel(0, 1, plot())
            .unwrap()
            .legend_policy(CanvasLegendPolicy::Shared),
        "constant-shared-beeswarm",
    )
    .unwrap();
    assert_eq!(svg.matches("<linearGradient ").count(), 1);
    let doc = roxmltree::Document::parse(&svg).unwrap();
    let gradient = doc
        .descendants()
        .find(|node| node.has_tag_name("linearGradient"))
        .unwrap();
    let stops = gradient
        .children()
        .filter(|node| node.has_tag_name("stop"))
        .map(|node| node.attribute("stop-color").unwrap())
        .collect::<Vec<_>>();
    assert_eq!(stops.len(), 2);
    assert_eq!(stops[0], stops[1]);
    let guide = doc
        .descendants()
        .find(|node| node.attribute("id") == Some("canvas-shared-legend"))
        .unwrap();
    let texts = guide
        .descendants()
        .filter(|node| node.has_tag_name("text"))
        .filter_map(|node| node.text())
        .collect::<Vec<_>>();
    assert_eq!(texts, ["Fill", "1"]);
}

#[test]
fn shared_diverging_heatmap_keeps_center_and_one_missing_key() {
    let plot = |values| {
        Plot::heatmap(
            CategoryValues::new("column", ["one", "two", "one"]).unwrap(),
            CategoryValues::new("row", ["top", "top", "bottom"]).unwrap(),
            NumericValues::new("fill", values),
        )
        .continuous_color_scale(
            lichtung_plot::ContinuousColorScale::diverging(0.0, None, None).unwrap(),
        )
    };
    let svg = render_canvas(
        Canvas::grid(1, 2)
            .unwrap()
            .size(1400, 700)
            .panel(0, 0, plot([-2.0, 0.0, 1.0]))
            .unwrap()
            .panel(0, 1, plot([-3.0, 0.0, 2.0]))
            .unwrap()
            .legend_policy(CanvasLegendPolicy::Shared),
        "shared-diverging-heatmap",
    )
    .unwrap();
    assert_eq!(svg.matches("<linearGradient ").count(), 1);
    let doc = roxmltree::Document::parse(&svg).unwrap();
    let stops = doc
        .descendants()
        .find(|node| node.has_tag_name("linearGradient"))
        .unwrap()
        .children()
        .filter(|node| node.has_tag_name("stop"))
        .map(|node| {
            (
                node.attribute("offset").unwrap(),
                node.attribute("stop-color").unwrap(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        stops,
        [
            ("0.000000", "#3182BD"),
            ("0.166667", "#78A9D2"),
            ("0.500000", "#F7F7F7"),
            ("0.666667", "#F7BCB3"),
            ("0.833333", "#EF7D70"),
            ("1.000000", "#DE2D26")
        ]
    );
    let guide = doc
        .descendants()
        .find(|node| node.attribute("id") == Some("canvas-shared-legend"))
        .unwrap();
    let texts = guide
        .descendants()
        .filter(|node| node.has_tag_name("text"))
        .filter_map(|node| node.text())
        .collect::<Vec<_>>();
    assert_eq!(texts, ["Fill", "-3", "0", "3", "Missing"]);
    assert_unique_svg_ids(&svg);
}

#[test]
fn shared_legend_is_global_complete_and_semantically_validated() {
    let first_x = NumericValues::new("x", [1.0, 2.0]);
    let first_y = NumericValues::new("y", [1.0, 2.0]);
    let first_trace = CategoryValues::new("trace", ["control", "control"]).unwrap();
    let second_x = NumericValues::new("x", [1.0, 2.0]);
    let second_y = NumericValues::new("y", [2.0, 3.0]);
    let second_trace = CategoryValues::new("trace", ["active", "active"]).unwrap();
    let second_other = CategoryValues::new("other", ["active", "active"]).unwrap();
    let svg = render_canvas(
        Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, hued_line(&first_x, &first_y, &first_trace))
            .unwrap()
            .panel(0, 1, hued_line(&second_x, &second_y, &second_trace))
            .unwrap()
            .legend_policy(CanvasLegendPolicy::Shared),
        "shared-global-legend",
    )
    .unwrap();
    assert_eq!(svg.matches("id=\"canvas-shared-legend\"").count(), 1);
    assert!(svg.contains(">Control</text>") && svg.contains(">Active</text>"));
    assert_unique_svg_ids(&svg);
    assert_grid_layers_before_marks(&svg, &["panel-0-grid", "panel-1-grid"]);

    let error = Canvas::grid(1, 2)
        .unwrap()
        .panel(0, 0, hued_line(&first_x, &first_y, &first_trace))
        .unwrap()
        .panel(0, 1, hued_line(&second_x, &second_y, &second_other))
        .unwrap()
        .legend_policy(CanvasLegendPolicy::Shared)
        .preflight()
        .unwrap_err();
    assert!(error.to_string().contains("compatible semantic titles"));
}

#[test]
fn shared_canvas_legend_merges_typed_collective_pairs() {
    let first_x = NumericValues::new("x", [0.0, 1.0]);
    let first_y = NumericValues::new("y", [1.0, 2.0]);
    let first_group = CategoryValues::new("group", ["g1", "g1"]).unwrap();
    let first_hue = CategoryValues::new("hue", ["h1", "h1"]).unwrap();
    let second_x = NumericValues::new("x", [0.0, 1.0]);
    let second_y = NumericValues::new("y", [3.0, 4.0]);
    let second_group = CategoryValues::new("group", ["g2", "g2"]).unwrap();
    let second_hue = CategoryValues::new("hue", ["h2", "h2"]).unwrap();
    let svg = render_canvas(
        Canvas::grid(1, 2)
            .unwrap()
            .panel(
                0,
                0,
                grouped_hued_line(&first_x, &first_y, &first_group, &first_hue),
            )
            .unwrap()
            .panel(
                0,
                1,
                grouped_hued_line(&second_x, &second_y, &second_group, &second_hue),
            )
            .unwrap()
            .legend_policy(CanvasLegendPolicy::Shared),
        "shared-pair-legend",
    )
    .unwrap();

    assert_eq!(svg.matches(">Group × Hue</text>").count(), 1);
    assert_eq!(svg.matches(">G 1, H 1</text>").count(), 1);
    assert_eq!(svg.matches(">G 2, H 2</text>").count(), 1);
    let legend = svg
        .split_once("<g id=\"canvas-shared-legend\">")
        .unwrap()
        .1
        .split_once("</g>")
        .unwrap()
        .0;
    assert_eq!(legend.matches("<polyline ").count(), 2);
    assert_eq!(legend.matches("<circle ").count(), 2);
}

#[test]
fn shared_legend_gap_reduces_canvas_legend_separation() {
    let x = NumericValues::new("x", [1.0, 2.0]);
    let y = NumericValues::new("y", [1.0, 2.0]);
    let trace = CategoryValues::new("trace", ["control", "control"]).unwrap();
    let render = |gap: Option<lichtung_plot::RelativeSpacing>| {
        let canvas = Canvas::grid(1, 1)
            .unwrap()
            .panel(0, 0, hued_line(&x, &y, &trace))
            .unwrap()
            .legend_policy(CanvasLegendPolicy::Shared);
        let canvas = match gap {
            Some(gap) => {
                let theme = Theme::publication();
                let legend = theme.legend().clone().with_shared_gap(gap).unwrap();
                canvas.theme(theme.customize().with_legend(legend).build().unwrap())
            }
            None => canvas,
        };
        render_canvas(canvas, "shared-legend-gap").unwrap()
    };
    let legend_x = |svg: &str| {
        let layer = svg.split("<g id=\"canvas-shared-legend\">").nth(1).unwrap();
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
    // Role-local 11px legend typography gives a 17.6px default shared gap.
    let compact = render(Some(lichtung_plot::RelativeSpacing::ratio(2, 11).unwrap()));
    assert!(
        legend_x(&compact) - panel_right(&compact) < legend_x(&default) - panel_right(&default)
    );
    assert!(lichtung_plot::RelativeSpacing::new(-1.0).is_err());
    let theme = Theme::publication();
    let legend = theme
        .legend()
        .clone()
        .with_shared_gap(lichtung_plot::RelativeSpacing::ratio(2, 11).unwrap())
        .unwrap();
    let theme = theme.customize().with_legend(legend).build().unwrap();
    for policy in [CanvasLegendPolicy::PerPlot, CanvasLegendPolicy::Hidden] {
        Canvas::grid(1, 1)
            .unwrap()
            .theme(theme.clone())
            .panel(0, 0, hued_line(&x, &y, &trace))
            .unwrap()
            .legend_policy(policy)
            .preflight()
            .unwrap();
    }
}

#[test]
fn per_plot_legends_keep_locally_observed_hue_levels() {
    let first_x = NumericValues::new("x", [1.0, 2.0]);
    let first_y = NumericValues::new("y", [1.0, 2.0]);
    let first_trace = CategoryValues::new("trace", ["control", "control"]).unwrap();
    let second_x = NumericValues::new("x", [1.0, 2.0]);
    let second_y = NumericValues::new("y", [2.0, 3.0]);
    let second_trace = CategoryValues::new("trace", ["active", "active"]).unwrap();
    let svg = render_canvas(
        Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, hued_line(&first_x, &first_y, &first_trace))
            .unwrap()
            .panel(0, 1, hued_line(&second_x, &second_y, &second_trace))
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
fn hidden_canvas_legends_do_not_preflight_suppressed_glyphs() {
    let absent = "\u{10FFFF}";
    let plot = Plot::scatter(
        NumericValues::new("x", [0.0, 1.0]),
        NumericValues::new("y", [0.0, 1.0]),
    )
    .hue(CategoryValues::new("cohort", [absent, absent]).unwrap());

    Canvas::grid(1, 1)
        .unwrap()
        .panel(0, 0, plot)
        .unwrap()
        .legend_policy(CanvasLegendPolicy::Hidden)
        .preflight()
        .unwrap();
}

#[test]
fn shared_canvas_legend_skips_plot_local_hidden_entries() {
    let absent = "\u{10FFFF}";
    let hidden = Plot::bar(
        CategoryValues::new("category", ["a", "b"]).unwrap(),
        BarInput::estimates(NumericValues::new("value", [1.0, 2.0])),
    )
    .series(CategoryValues::new("cohort", [absent, absent]).unwrap())
    .legend(LegendPolicy::Hidden);
    let visible = Plot::bar(
        CategoryValues::new("category", ["a", "b"]).unwrap(),
        BarInput::estimates(NumericValues::new("value", [2.0, 3.0])),
    )
    .series(CategoryValues::new("cohort", ["visible", "visible"]).unwrap());

    Canvas::grid(1, 2)
        .unwrap()
        .panel(0, 0, hidden)
        .unwrap()
        .panel(0, 1, visible)
        .unwrap()
        .legend_policy(CanvasLegendPolicy::Shared)
        .preflight()
        .unwrap();
}

#[test]
fn hidden_discrete_guide_still_coordinates_visible_mark_colors() {
    let hidden = Plot::interaction(
        CategoryValues::new("condition", ["before", "after"]).unwrap(),
        NumericValues::new("estimate", [2.0, 3.0]),
        CategoryValues::new("cohort", ["hidden", "hidden"]).unwrap(),
        lichtung_plot::Bounds::new(
            NumericValues::new("lower", [1.0, 2.0]),
            NumericValues::new("upper", [3.0, 4.0]),
        ),
    )
    .legend(LegendPolicy::Hidden);
    let scatter = |hue: &str| {
        Plot::scatter(
            NumericValues::new("x", [1.0, 2.0]),
            NumericValues::new("y", [2.0, 3.0]),
        )
        .hue(CategoryValues::new("cohort", [hue, hue]).unwrap())
    };
    let svg = render_canvas(
        Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, hidden)
            .unwrap()
            .panel(0, 1, scatter("visible"))
            .unwrap()
            .legend_policy(CanvasLegendPolicy::Shared),
        "shared-hidden-visible-color-domain",
    )
    .unwrap();
    let guide = svg.split_once("<g id=\"canvas-shared-legend\">").unwrap().1;
    assert!(guide.contains(">Visible</text>"));
    assert!(!guide.contains(">Hidden</text>"));
    // Group-only Interaction has no hue list or guide contribution, yet its
    // marks own the first categorical color. Scatter retains the second slot.
    assert!(svg.contains("fill=\"#0072B2\""));
    assert!(svg.contains("fill=\"#D55E00\""));
}

#[test]
fn canvas_filters_point_labels_outside_an_exact_y_range() {
    let outside = lichtung_plot::PointLayer::new(
        NumericValues::new("point x", [2.0]),
        NumericValues::new("point y", [2.0]),
    )
    .labels(CategoryValues::new("labels", ["\u{10FFFF}"]).unwrap());
    let plot = Plot::line(
        NumericValues::new("x", [0.0, 1.0]),
        NumericValues::new("y", [0.0, 1.0]),
    )
    .y_limits(0.0, 1.0)
    .point_layer(outside);
    let range_peer = Plot::line(
        NumericValues::new("x", [0.0, 3.0]),
        NumericValues::new("y", [0.0, 1.0]),
    );

    Canvas::grid(1, 2)
        .unwrap()
        .panel(0, 0, plot)
        .unwrap()
        .panel(0, 1, range_peer)
        .unwrap()
        .x_range_group([(0, 0), (0, 1)])
        .unwrap()
        .preflight()
        .unwrap();
}

#[test]
fn canvas_x_range_groups_reject_explicit_and_categorical_axes() {
    let continuous = || {
        Plot::line(
            NumericValues::new("x", [0.0, 1.0]),
            NumericValues::new("y", [0.0, 1.0]),
        )
    };
    let explicit_error = Canvas::grid(1, 2)
        .unwrap()
        .panel(0, 0, continuous().x_limits(0.0, 1.0))
        .unwrap()
        .panel(0, 1, continuous())
        .unwrap()
        .x_range_group([(0, 0), (0, 1)])
        .unwrap()
        .preflight()
        .unwrap_err()
        .to_string();
    assert!(explicit_error.contains("explicit x limits"));

    let categorical = Plot::bar(
        CategoryValues::new("category", ["a", "b"]).unwrap(),
        BarInput::estimates(NumericValues::new("value", [1.0, 2.0])),
    );
    let categorical_error = Canvas::grid(1, 2)
        .unwrap()
        .panel(0, 0, categorical)
        .unwrap()
        .panel(0, 1, continuous())
        .unwrap()
        .x_range_group([(0, 0), (0, 1)])
        .unwrap()
        .preflight()
        .unwrap_err()
        .to_string();
    assert!(categorical_error.contains("categorical x axis"));
}

#[test]
fn composed_offset_panel_reserves_long_y_tick_labels_locally() {
    let line_data = estimate_data();
    let category = CategoryValues::new(
        "category",
        ["a very long category label in the second panel", "short"],
    )
    .unwrap();
    let value = NumericValues::new("value", [2.0, 4.0]);
    let svg = render_canvas(
        Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, estimate_line(&line_data))
            .unwrap()
            .panel(0, 1, horizontal_bar(&category, &value))
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
    let line_data = estimate_data();
    let category = CategoryValues::new("category", ["3D-SART Understanding", "short"]).unwrap();
    let value = NumericValues::new("value", [2.0, 4.0]);
    let svg = render_canvas(
        Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, estimate_line(&line_data))
            .unwrap()
            .panel(0, 1, horizontal_bar(&category, &value).y_label("Outcome"))
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
    let category = CategoryValues::new("category", ["first", "second"]).unwrap();
    let value = NumericValues::new("value", [2.0, 4.0]);
    let svg = render_canvas(
        Canvas::grid(1, 1)
            .unwrap()
            .panel(
                0,
                0,
                horizontal_bar(&category, &value).y_label("Linguistic\nproperty\ncontribution"),
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
        let pivot_x = attribute(line, "transform")
            .strip_prefix("rotate(-90.0,")
            .unwrap()
            .split_once(',')
            .unwrap()
            .0
            .parse::<f64>()
            .unwrap();
        assert!(pivot_x >= 12.0);
    }
}

#[test]
fn composed_panel_reserves_categorical_y_axis_text_once() {
    let category = "category label measured exactly once";
    let categories = CategoryValues::new("category", [category, "short"]).unwrap();
    let value = NumericValues::new("value", [2.0, 4.0]);
    let svg = render_canvas(
        Canvas::grid(1, 1)
            .unwrap()
            .panel(0, 0, horizontal_bar(&categories, &value))
            .unwrap(),
        "category-gutter-once",
    )
    .unwrap();
    assert!(clip_rects(&svg)[0].0 < 300.0);
}

#[test]
fn composed_horizontal_bars_support_shared_headers_labels_and_range_groups() {
    let outcome = CategoryValues::new("outcome", ["alpha", "beta"]).unwrap();
    let left = NumericValues::new("value", [1.0, -0.5]);
    let right = NumericValues::new("value", [8.0, -6.0]);
    let svg = render_canvas(
        Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, horizontal_bar(&outcome, &left))
            .unwrap()
            .panel(0, 1, horizontal_bar(&outcome, &right))
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
        .split("<g id=\"canvas-shared-headers\">")
        .nth(1)
        .unwrap()
        .split("</g>")
        .next()
        .unwrap();
    assert!(!headers.contains("font-weight="));
}

#[test]
fn composed_row_title_spans_all_columns_and_rejects_panel_titles() {
    let x = NumericValues::new("x", [1.0, 2.0]);
    let y = NumericValues::new("y", [2.0, 3.0]);
    let line = || Plot::line(x.clone(), y.clone());
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
    assert_eq!(svg.matches("id=\"canvas-row-titles\"").count(), 1);
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
fn wrapped_row_title_keeps_all_lines_above_measured_cell_content() {
    let title = "Independent longitudinal reference measurement period";
    let svg = render_canvas(
        Canvas::grid(1, 1)
            .unwrap()
            .size(220, 350)
            .row_title(0, title)
            .unwrap()
            .text_panel(
                0,
                0,
                lichtung_plot::TextPanel::new("Content", "Body").unwrap(),
            )
            .unwrap(),
        "wrapped-row-title",
    )
    .unwrap();
    let document = roxmltree::Document::parse(&svg).unwrap();
    let row_titles = document
        .descendants()
        .find(|node| node.attribute("id") == Some("canvas-row-titles"))
        .unwrap();
    let lines = row_titles
        .descendants()
        .filter(|node| node.has_tag_name("text"))
        .collect::<Vec<_>>();
    assert!(lines.len() > 1);
    let words = lines
        .iter()
        .filter_map(roxmltree::Node::text)
        .flat_map(str::split_whitespace)
        .collect::<Vec<_>>();
    assert_eq!(words, title.split_whitespace().collect::<Vec<_>>());
    let body_y = document
        .descendants()
        .find(|node| node.has_tag_name("text") && node.text() == Some("Content"))
        .unwrap()
        .attribute("y")
        .unwrap()
        .parse::<f64>()
        .unwrap();
    assert!(lines
        .iter()
        .all(|line| line.attribute("y").unwrap().parse::<f64>().unwrap() < body_y));
}

#[test]
fn shared_category_labels_reject_different_row_domains_preflight() {
    let left = CategoryValues::new("outcome", ["alpha", "beta"]).unwrap();
    let right = CategoryValues::new("outcome", ["alpha", "gamma"]).unwrap();
    let value = NumericValues::new("value", [1.0, 2.0]);
    let error = Canvas::grid(1, 2)
        .unwrap()
        .panel(0, 0, horizontal_bar(&left, &value))
        .unwrap()
        .panel(0, 1, horizontal_bar(&right, &value))
        .unwrap()
        .category_labels(CanvasCategoryLabels::FirstColumnPerRow)
        .preflight()
        .unwrap_err();
    assert!(error.to_string().contains("category domains differ"));
}

#[test]
fn shared_category_labels_compare_rendered_explicit_orders() {
    let left = CategoryValues::new("outcome", ["alpha", "beta"]).unwrap();
    let left_value = NumericValues::new("value", [1.0, 2.0]);
    let reversed = CategoryValues::new("outcome", ["beta", "alpha"]).unwrap();
    let reversed_value = NumericValues::new("value", [2.0, 1.0]);
    let bar = |category: &CategoryValues, value: &NumericValues, order| {
        horizontal_bar(category, value).category_order(CategoryOrder::explicit(order))
    };

    Canvas::grid(1, 2)
        .unwrap()
        .panel(0, 0, bar(&left, &left_value, ["alpha", "beta"]))
        .unwrap()
        .panel(0, 1, bar(&reversed, &reversed_value, ["alpha", "beta"]))
        .unwrap()
        .category_labels(CanvasCategoryLabels::FirstColumnPerRow)
        .preflight()
        .unwrap();

    let error = Canvas::grid(1, 2)
        .unwrap()
        .panel(0, 0, bar(&left, &left_value, ["alpha", "beta"]))
        .unwrap()
        .panel(0, 1, bar(&left, &left_value, ["beta", "alpha"]))
        .unwrap()
        .category_labels(CanvasCategoryLabels::FirstColumnPerRow)
        .preflight()
        .unwrap_err();
    assert!(error.to_string().contains("category domains differ"));
}

#[test]
fn composed_svg_is_byte_deterministic() {
    let build = || {
        let data = estimate_data();
        Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, estimate_line(&data))
            .unwrap()
            .panel(
                0,
                1,
                Plot::forest(
                    data.term,
                    data.estimate,
                    Bounds::new(data.lower, data.upper),
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
    let x = NumericValues::new("x", [1.0, 2.0]);
    let y = NumericValues::new("y", [2.0, 3.0]);
    let svg = render_canvas(
        Canvas::grid(1, 1)
            .unwrap()
            .panel_title_alignment(lichtung_plot::TitleAlignment::Start)
            .panel_subtitle_alignment(lichtung_plot::TitleAlignment::End)
            .panel(
                0,
                0,
                Plot::line(x, y)
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
    assert!(
        attribute(title, "x").parse::<f64>().unwrap()
            < attribute(subtitle, "x").parse::<f64>().unwrap()
    );
    assert!(subtitle_y - title_y > 15.5 && subtitle_y + 7.5 < panel_y);
}

#[test]
fn shared_canvas_scatter_legend_rejects_null_hue_rows_preflight() {
    let left_x = NumericValues::new("x", [1.0, 2.0]);
    let left_y = NumericValues::new("y", [1.0, 2.0]);
    let left_trace = CategoryValues::nullable("trace", [Some("control"), None]).unwrap();
    let right_x = NumericValues::new("x", [1.0, 2.0, 3.0]);
    let right_y = NumericValues::new("y", [2.0, 3.0, 4.0]);
    let right_trace =
        CategoryValues::nullable("trace", [Some("active"), Some("active"), None]).unwrap();
    let error = Canvas::grid(1, 2)
        .unwrap()
        .panel(0, 0, Plot::scatter(left_x, left_y).hue(left_trace))
        .unwrap()
        .panel(0, 1, Plot::scatter(right_x, right_y).hue(right_trace))
        .unwrap()
        .legend_policy(CanvasLegendPolicy::Shared)
        .preflight()
        .unwrap_err();
    assert!(error
        .to_string()
        .contains("value \"trace\" contains null at source row 1"));
}

#[test]
fn shared_canvas_equal_color_entries_keep_a_typed_bar_target_in_its_owner() {
    let bar = |values: [f64; 2]| {
        Plot::bar(
            CategoryValues::new("category", ["same", "same"]).unwrap(),
            BarInput::estimates(NumericValues::new("value", values)),
        )
        .series(CategoryValues::new("series", ["g1", "g2"]).unwrap())
        .categorical_color_scale(CategoricalColorScale::from_map(
            CategoricalColorMap::new([
                ("g1".into(), OpaqueColor::rgb(20, 40, 60)),
                ("g2".into(), OpaqueColor::rgb(20, 40, 60)),
            ])
            .unwrap(),
        ))
    };
    let first = bar([1.0, 2.0]).significance([Significance::between(
        SignificanceTarget::category_series("same", "g1"),
        SignificanceTarget::category_series("same", "g2"),
        SignificanceMark::OneStar,
    )]);
    let svg = render_canvas(
        Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, first)
            .unwrap()
            .panel(0, 1, bar([2.0, 3.0]))
            .unwrap()
            .legend_policy(CanvasLegendPolicy::Shared)
            .size(1200, 640),
        "shared-typed-bar-target",
    )
    .unwrap();
    assert_eq!(svg.matches("class=\"significance-bracket\"").count(), 1);
    let guide = svg.split_once("<g id=\"canvas-shared-legend\">").unwrap().1;
    assert!(guide.contains(">G 1</text>") && guide.contains(">G 2</text>"));
    assert_eq!(svg.matches("fill=\"#14283C\"").count(), 6);
}

#[test]
fn all_canvas_grids_precede_any_marks() {
    let data = estimate_data();
    let svg = render_canvas(
        Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, estimate_line(&data))
            .unwrap()
            .panel(0, 1, Plot::scatter(data.x, data.estimate))
            .unwrap(),
        "grid-before-marks",
    )
    .unwrap();
    assert_grid_layers_before_marks(&svg, &["panel-0-grid", "panel-1-grid"]);
}

#[test]
fn heterogeneous_canvas_is_deterministic_and_clipped_once_per_plot() {
    let build = || {
        let data = estimate_data();
        Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, estimate_line(&data))
            .unwrap()
            .panel(
                0,
                1,
                Plot::coefficient(
                    data.term,
                    CoefficientInput::estimates_with_intervals(
                        data.estimate,
                        Bounds::new(data.lower, data.upper),
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
    let data = estimate_data();
    assert!(Canvas::grid(1, 1)
        .unwrap()
        .panel(0, 0, estimate_line(&data))
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
fn canvas_and_plot_reject_misaligned_values_with_the_same_error() {
    let plot = Plot::line(
        lichtung_plot::NumericValues::new("x", [1.0, 2.0]),
        lichtung_plot::NumericValues::new("y", [3.0, 4.0, 5.0]),
    );
    let plot_error = plot.clone().preflight().unwrap_err().to_string();
    let canvas_error = Canvas::grid(1, 1)
        .unwrap()
        .panel(0, 0, plot)
        .unwrap()
        .preflight()
        .unwrap_err()
        .to_string();

    assert_eq!(plot_error, canvas_error);
    assert_eq!(
        canvas_error,
        "data error: plot value \"y\" has length 3, expected 2"
    );
}

#[test]
fn canvas_and_plot_reject_misaligned_estimate_status_with_the_same_error() {
    let plot = Plot::point_interval(
        lichtung_plot::CategoryValues::new("term", ["first", "second"]).unwrap(),
        lichtung_plot::NumericValues::new("estimate", [1.0, 2.0]),
        Bounds::new(
            lichtung_plot::NumericValues::new("lower", [0.8, 1.8]),
            lichtung_plot::NumericValues::new("upper", [1.2, 2.2]),
        ),
    )
    .estimate_status([
        EstimateStatus::Estimated,
        EstimateStatus::Estimated,
        EstimateStatus::Estimated,
    ]);
    let plot_error = plot.clone().preflight().unwrap_err().to_string();
    let canvas_error = Canvas::grid(1, 1)
        .unwrap()
        .panel(0, 0, plot)
        .unwrap()
        .preflight()
        .unwrap_err()
        .to_string();

    assert_eq!(plot_error, canvas_error);
    assert_eq!(
        canvas_error,
        "data error: plot estimate status values have length 3, expected 2"
    );
}

#[test]
fn canvas_and_plot_reject_incompatible_guides_with_the_same_error() {
    let style = GuideStyle::new(
        lichtung_plot::Color::rgb(31, 73, 125),
        1.0,
        LinePattern::Solid,
    )
    .unwrap();
    let plot = Plot::bar(
        lichtung_plot::CategoryValues::new("category", ["a", "b"]).unwrap(),
        BarInput::samples(lichtung_plot::NumericValues::new("value", [1.0, 2.0])),
    )
    .guide(GuideSpec::segment(0.0, 0.5, 1.0, 1.5, style));
    let plot_error = plot.clone().preflight().unwrap_err().to_string();
    let canvas_error = Canvas::grid(1, 1)
        .unwrap()
        .panel(0, 0, plot)
        .unwrap()
        .preflight()
        .unwrap_err()
        .to_string();

    assert_eq!(plot_error, canvas_error);
    assert!(canvas_error.contains("requires numeric x and y axes"));
}

#[test]
fn canvas_renders_valid_supplemental_numeric_geometry() {
    let style = GuideStyle::new(
        lichtung_plot::Color::rgb(123, 45, 67),
        1.5,
        LinePattern::Dashed,
    )
    .unwrap();
    let plot = Plot::line(
        lichtung_plot::NumericValues::new("x", [0.0, 1.0, 2.0]),
        lichtung_plot::NumericValues::new("y", [1.0, 2.0, 3.0]),
    )
    .guide(GuideSpec::segment(0.5, 1.5, 1.5, 2.5, style));
    let output = render_canvas(
        Canvas::grid(1, 1).unwrap().panel(0, 0, plot).unwrap(),
        "canvas-valid-guide",
    )
    .unwrap();

    assert!(output.contains("stroke=\"#7B2D43\"") && output.contains("stroke-dasharray="));
}

#[test]
fn invalid_panel_band_and_facet_cell_fail_canvas_preflight() {
    let data = estimate_data();
    let bar = Plot::bar(data.factor, BarInput::samples(data.estimate)).significance([
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
