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

fn horizontal_bar(
    category: &CategoryValues,
    value: &NumericValues,
) -> Plot<lichtung_plot::kind::HorizontalBar> {
    Plot::horizontal_bar(category.clone(), BarInput::estimates(value.clone()))
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
fn shared_point_guides_merge_across_families_only_with_equal_complete_keys() {
    use lichtung_plot::{FontFamily, FontStack, LegendComposition, PointShape, PointTheme};
    let canvas = |composition, trajectory_radius, absent_font| {
        let base = Theme::publication();
        let radii = base
            .marks()
            .get_points()
            .get_radii()
            .clone()
            .scatter(6)
            .unwrap()
            .trajectory(trajectory_radius)
            .unwrap();
        let points = base
            .marks()
            .get_points()
            .clone()
            .primary(PointTheme::new(91, PointShape::Circle, None).unwrap())
            .unwrap()
            .radii(radii)
            .unwrap();
        let marks = base.marks().clone().points(points).unwrap();
        let theme = base.customize().marks(marks);
        let theme = if absent_font {
            theme
                .font_stack(
                    FontStack::new([FontFamily::named("Absent cross-family guide font").unwrap()])
                        .unwrap(),
                )
                .unwrap()
        } else {
            theme
        }
        .build()
        .unwrap();
        let x = NumericValues::new("x", [0.0, 1.0]);
        let y = NumericValues::new("y", [1.0, 2.0]);
        let hue = CategoryValues::new("cohort", ["seen", "seen"]).unwrap();
        Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, Plot::scatter(x.clone(), y.clone()).hue(hue.clone()))
            .unwrap()
            .panel(
                0,
                1,
                Plot::trajectory(
                    x,
                    y,
                    CategoryValues::new("subjects", ["\u{10FFFF}"; 2]).unwrap(),
                )
                .hue(hue),
            )
            .unwrap()
            .theme(theme)
            .labels(Labels::raw())
            .legend_policy(CanvasLegendPolicy::Shared)
            .legend_composition(composition)
            .size(1000, 500)
    };
    for composition in [LegendComposition::Combined, LegendComposition::Separate] {
        let output =
            render_canvas(canvas(composition, 6, false), "cross-family-point-key").unwrap();
        let legend = output
            .split_once("<g id=\"canvas-shared-legend\">")
            .unwrap()
            .1
            .split_once("</g>")
            .unwrap()
            .0;
        assert_eq!(legend.matches(">cohort</text>").count(), 1);
        assert_eq!(legend.matches(">seen</text>").count(), 1);
        assert_eq!(legend.matches("<circle ").count(), 1);
        assert!(!legend.contains("<polyline "));
        assert_eq!(output.matches("<circle ").count(), 5);
        for point in output.lines().filter(|line| line.starts_with("<circle ")) {
            assert_eq!(
                attribute(point, "r").parse::<f64>().unwrap().to_bits(),
                6.0_f64.to_bits()
            );
            assert_eq!(
                attribute(point, "fill-opacity")
                    .parse::<f64>()
                    .unwrap()
                    .to_bits(),
                (91.0_f64 / 255.0).to_bits()
            );
        }
        let error = canvas(composition, 7, true).preflight().unwrap_err();
        assert!(error
            .to_string()
            .contains("compatible semantic titles, labels, and styles"));
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
            .max_width(901)
            .legend_policy(CanvasLegendPolicy::PerPlot),
        "per-plot-local-legends",
    )
    .unwrap();
    let document = roxmltree::Document::parse(&svg).unwrap();
    assert!(
        document
            .root_element()
            .attribute("width")
            .unwrap()
            .parse::<u32>()
            .unwrap()
            <= 901
    );
    assert_eq!(svg.matches(">Control</text>").count(), 1);
    assert_eq!(svg.matches(">Active</text>").count(), 1);
    assert!(svg.contains("id=\"panel-0-legend\"") && svg.contains("id=\"panel-1-legend\""));
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
fn spanning_canvas_keeps_three_complete_owners_and_all_grids_precede_marks() {
    let data = estimate_data();
    let svg = render_canvas(
        Canvas::grid(2, 2)
            .unwrap()
            .panel(0, 0, estimate_line(&data))
            .unwrap()
            .panel(1, 0, estimate_line(&data))
            .unwrap()
            .panel_span(
                0,
                1,
                2,
                1,
                Plot::scatter(data.x, data.estimate).title("Tall owner"),
            )
            .unwrap(),
        "grid-before-marks",
    )
    .unwrap();
    assert_grid_layers_before_marks(&svg, &["panel-0-grid", "panel-1-grid", "panel-2-grid"]);
    assert_eq!(svg.matches("<clipPath ").count(), 3);
    assert_eq!(svg.matches(">Tall owner<").count(), 1);
}
