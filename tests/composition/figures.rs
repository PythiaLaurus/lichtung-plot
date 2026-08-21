use super::*;

#[test]
fn shared_legend_is_global_complete_and_semantically_validated() {
    let first = df!(
        "x" => [1.0_f64, 2.0],
        "y" => [1.0_f64, 2.0],
        "trace" => ["control", "control"]
    )
    .unwrap();
    let second = df!(
        "x" => [1.0_f64, 2.0],
        "y" => [2.0_f64, 3.0],
        "trace" => ["active", "active"],
        "other" => ["active", "active"]
    )
    .unwrap();
    let mut left = Chart::new(&first).hue("trace");
    left.x("x").y("y").line();
    let mut right = Chart::new(&second).hue("trace");
    right.x("x").y("y").line();
    let figure = Figure::grid(1, 2)
        .unwrap()
        .panel(0, 0, left)
        .unwrap()
        .panel(0, 1, right)
        .unwrap()
        .legend_policy(FigureLegendPolicy::Shared);
    let dir = tempdir("shared-global-legend");
    let output = dir.join("figure.svg");
    figure.render_to_svg(&output).unwrap();
    let svg = std::fs::read_to_string(output).unwrap();
    assert_eq!(svg.matches("id=\"figure-shared-legend\"").count(), 1);
    assert!(svg.contains(">Control</text>") && svg.contains(">Active</text>"));
    assert_unique_svg_ids(&svg);
    assert_grid_layers_before_marks(&svg, &["panel-0-grid", "panel-1-grid"]);

    let mut left = Chart::new(&first).hue("trace");
    left.x("x").y("y").line();
    let mut incompatible = Chart::new(&second).hue("other");
    incompatible.x("x").y("y").line();
    let figure = Figure::grid(1, 2)
        .unwrap()
        .panel(0, 0, left)
        .unwrap()
        .panel(0, 1, incompatible)
        .unwrap()
        .legend_policy(FigureLegendPolicy::Shared);
    let error = figure
        .render_with_backend(&mut LifecycleBackend::default())
        .unwrap_err()
        .to_string();
    assert!(error.contains("compatible semantic titles"));
}

#[test]
fn shared_legend_gap_reduces_figure_legend_separation() {
    let data = df!(
        "x" => [1.0_f64, 2.0],
        "y" => [1.0_f64, 2.0],
        "trace" => ["control", "control"]
    )
    .unwrap();
    let render = |gap| {
        let mut chart = Chart::new(&data).hue("trace");
        chart.x("x").y("y").line();
        let figure = Figure::grid(1, 1)
            .unwrap()
            .panel(0, 0, chart)
            .unwrap()
            .legend_policy(FigureLegendPolicy::Shared);
        let figure = if let Some(gap) = gap {
            figure.shared_legend_gap(gap).unwrap()
        } else {
            figure
        };
        String::from_utf8(figure.render_to_svg_bytes().unwrap()).unwrap()
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
    assert!(Figure::grid(1, 1).unwrap().shared_legend_gap(-1.0).is_err());
}

#[test]
fn per_panel_legends_keep_panel_observed_hue_levels() {
    let first = df!(
        "x" => [1.0_f64, 2.0],
        "y" => [1.0_f64, 2.0],
        "trace" => ["control", "control"]
    )
    .unwrap();
    let second = df!(
        "x" => [1.0_f64, 2.0],
        "y" => [2.0_f64, 3.0],
        "trace" => ["active", "active"]
    )
    .unwrap();
    let mut left = Chart::new(&first).hue("trace");
    left.x("x").y("y").line();
    let mut right = Chart::new(&second).hue("trace");
    right.x("x").y("y").line();
    let figure = Figure::grid(1, 2)
        .unwrap()
        .panel(0, 0, left)
        .unwrap()
        .panel(0, 1, right)
        .unwrap()
        .legend_policy(FigureLegendPolicy::PerPanel);
    let dir = tempdir("per-panel-local-legends");
    let output = dir.join("figure.svg");

    figure.render_to_svg(&output).unwrap();
    let svg = std::fs::read_to_string(output).unwrap();

    assert_eq!(svg.matches(">Control</text>").count(), 1);
    assert_eq!(svg.matches(">Active</text>").count(), 1);
    assert!(svg.contains("id=\"panel-0-legend\"") && svg.contains("id=\"panel-1-legend\""));
}

#[test]
fn composed_offset_panel_reserves_long_y_tick_labels_locally() {
    let line_data = estimate_data();
    let bar_data = df!(
        "category" => ["a very long category label in the second panel", "short"],
        "value" => [2.0_f64, 4.0]
    )
    .unwrap();
    let mut line = Chart::new(&line_data).group("trace").hue("trace");
    line.x("x").y("estimate").line();
    let mut horizontal = Chart::new(&bar_data).bar_aggregation(BarAggregation::Identity);
    horizontal.x("category").y("value").horizontal_bar();
    let figure = Figure::grid(1, 2)
        .unwrap()
        .panel(0, 0, line)
        .unwrap()
        .panel(0, 1, horizontal)
        .unwrap();
    let mut backend = LifecycleBackend::default();

    figure.render_with_backend(&mut backend).unwrap();

    assert_eq!(backend.clips, 2);
    assert!(
        backend.clip_regions[1].x0 > backend.clip_regions[0].x1 + 40.0,
        "long categorical labels should reserve a local inter-panel gutter: {:?}",
        backend.clip_regions
    );
}

#[test]
fn composed_right_panel_keeps_y_axis_title_out_of_the_left_panel() {
    let line_data = estimate_data();
    let bar_data = df!(
        "category" => ["3D-SART Understanding", "short"],
        "value" => [2.0_f64, 4.0]
    )
    .unwrap();
    let mut line = Chart::new(&line_data).group("trace").hue("trace");
    line.x("x").y("estimate").line();
    let mut horizontal = Chart::new(&bar_data)
        .bar_aggregation(BarAggregation::Identity)
        .y_label("Outcome");
    horizontal.x("category").y("value").horizontal_bar();
    let figure = Figure::grid(1, 2)
        .unwrap()
        .panel(0, 0, line)
        .unwrap()
        .panel(0, 1, horizontal)
        .unwrap();
    let mut backend = LifecycleBackend::default();

    figure.render_with_backend(&mut backend).unwrap();

    let outcome_x = backend
        .texts
        .iter()
        .find(|(_, _, text)| text == "Outcome")
        .map(|(x, _, _)| f64::from(*x))
        .unwrap();
    assert!(
        outcome_x > backend.clip_regions[0].x1,
        "right-panel y-axis title entered the left panel: title x={outcome_x}, panels={:?}",
        backend.clip_regions
    );
}

#[test]
fn multiline_y_axis_title_keeps_every_line_inside_the_canvas() {
    let data = df!(
        "category" => ["first", "second"],
        "value" => [2.0_f64, 4.0]
    )
    .unwrap();
    let mut chart = Chart::new(&data)
        .bar_aggregation(BarAggregation::Identity)
        .y_label("Linguistic\nproperty\ncontribution");
    chart.x("category").y("value").horizontal_bar();
    let figure = Figure::grid(1, 1).unwrap().panel(0, 0, chart).unwrap();
    let mut backend = LifecycleBackend::default();

    figure.render_with_backend(&mut backend).unwrap();

    let axis_line_x = backend
        .texts
        .iter()
        .filter(|(_, _, text)| matches!(text.as_str(), "Linguistic" | "property" | "contribution"))
        .map(|(x, _, _)| *x)
        .collect::<Vec<_>>();
    assert_eq!(axis_line_x.len(), 3);
    assert!(
        axis_line_x.iter().all(|x| *x >= 12),
        "multiline y-axis title crossed the canvas edge: {axis_line_x:?}"
    );
}

#[test]
fn composed_panel_reserves_categorical_y_axis_text_once() {
    let category = "category label measured exactly once";
    let data = df!(
        "category" => [category, "short"],
        "value" => [2.0_f64, 4.0]
    )
    .unwrap();
    let mut chart = Chart::new(&data).bar_aggregation(BarAggregation::Identity);
    chart.x("category").y("value").horizontal_bar();
    let figure = Figure::grid(1, 1).unwrap().panel(0, 0, chart).unwrap();
    let mut backend = LifecycleBackend::default();

    figure.render_with_backend(&mut backend).unwrap();

    let label_width = lichtung_plot::layout::deterministic_text_width(category, 12.0);
    assert!(
        backend.clip_regions[0].x0 < label_width * 1.75,
        "categorical y-axis gutter was reserved more than once: panel x0={}, label width={label_width}",
        backend.clip_regions[0].x0
    );
}

#[test]
fn composed_horizontal_bars_support_shared_headers_labels_and_range_groups() {
    let left_data = df!(
        "outcome" => ["alpha", "beta"],
        "value" => [1.0_f64, -0.5]
    )
    .unwrap();
    let right_data = df!(
        "outcome" => ["alpha", "beta"],
        "value" => [8.0_f64, -6.0]
    )
    .unwrap();
    let mut left = Chart::new(&left_data).bar_aggregation(BarAggregation::Identity);
    left.x("outcome").y("value").horizontal_bar();
    let mut right = Chart::new(&right_data).bar_aggregation(BarAggregation::Identity);
    right.x("outcome").y("value").horizontal_bar();
    let figure = Figure::grid(1, 2)
        .unwrap()
        .panel(0, 0, left)
        .unwrap()
        .panel(0, 1, right)
        .unwrap()
        .column_header(0, 1, "Correlation")
        .unwrap()
        .column_header(1, 1, "Regression")
        .unwrap()
        .row_header(0, 1, "Outcome")
        .unwrap()
        .category_labels(FigureCategoryLabels::FirstColumnPerRow)
        .x_range_group([(0, 0), (0, 1)])
        .unwrap();

    let svg = String::from_utf8(figure.render_to_svg_bytes().unwrap()).unwrap();

    assert_eq!(svg.matches(">Alpha</text>").count(), 1);
    assert_eq!(svg.matches(">Beta</text>").count(), 1);
    assert!(svg.contains(">Correlation</text>"));
    assert!(svg.contains(">Regression</text>"));
    assert!(svg.contains(">Outcome</text>"));
    assert_eq!(svg.matches("id=\"figure-shared-headers\"").count(), 1);
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
fn composed_row_title_spans_all_columns_and_renders_once() {
    let data = df!(
        "outcome" => ["alpha", "beta"],
        "value" => [1.0_f64, -0.5]
    )
    .unwrap();
    let mut left = Chart::new(&data).bar_aggregation(BarAggregation::Identity);
    left.x("outcome").y("value").horizontal_bar();
    let mut right = Chart::new(&data).bar_aggregation(BarAggregation::Identity);
    right.x("outcome").y("value").horizontal_bar();
    let svg = String::from_utf8(
        Figure::grid(1, 2)
            .unwrap()
            .panel(0, 0, left)
            .unwrap()
            .panel(0, 1, right)
            .unwrap()
            .row_title(0, "Shared outcome")
            .unwrap()
            .render_to_svg_bytes()
            .unwrap(),
    )
    .unwrap();

    assert_eq!(svg.matches(">Shared outcome</text>").count(), 1);
    assert_eq!(svg.matches("id=\"figure-row-titles\"").count(), 1);
    assert!(!svg.contains("id=\"panel-0-title\"") && !svg.contains("id=\"panel-1-title\""));
}

#[test]
fn composed_row_title_rejects_panel_title_in_same_row() {
    let data = df!("x" => [1.0_f64, 2.0], "y" => [2.0_f64, 3.0]).unwrap();
    let mut chart = Chart::new(&data).title("Panel title");
    chart.x("x").y("y").line();
    let error = Figure::grid(1, 1)
        .unwrap()
        .panel(0, 0, chart)
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
fn shared_category_labels_reject_different_row_domains_before_begin() {
    let left_data = df!("outcome" => ["alpha", "beta"], "value" => [1.0_f64, 2.0]).unwrap();
    let right_data = df!("outcome" => ["alpha", "gamma"], "value" => [1.0_f64, 2.0]).unwrap();
    let mut left = Chart::new(&left_data).bar_aggregation(BarAggregation::Identity);
    left.x("outcome").y("value").horizontal_bar();
    let mut right = Chart::new(&right_data).bar_aggregation(BarAggregation::Identity);
    right.x("outcome").y("value").horizontal_bar();
    let figure = Figure::grid(1, 2)
        .unwrap()
        .panel(0, 0, left)
        .unwrap()
        .panel(0, 1, right)
        .unwrap()
        .category_labels(FigureCategoryLabels::FirstColumnPerRow);
    let mut backend = LifecycleBackend::default();

    let error = figure
        .render_with_backend(&mut backend)
        .unwrap_err()
        .to_string();

    assert!(error.contains("category domains differ"), "{error}");
    assert_eq!(backend.begins, 0);
}

#[test]
fn composed_svg_bytes_match_file_output() {
    let data = estimate_data();
    let mut left = Chart::new(&data).group("trace").hue("trace");
    left.x("x").y("estimate").line();
    let mut right = Chart::new(&data)
        .interval_bounds("lower", "upper")
        .interval_semantics(confidence(InferentialScale::Response))
        .reference_line(0.0);
    right.x("estimate").y("term").forest();
    let figure = Figure::grid(1, 2)
        .unwrap()
        .panel(0, 0, left)
        .unwrap()
        .panel(0, 1, right)
        .unwrap()
        .title("Composed in-memory parity");
    let dir = tempdir("composed-svg-bytes");
    let path = dir.join("figure.svg");

    figure.render_to_svg(&path).unwrap();

    assert_eq!(
        figure.render_to_svg_bytes().unwrap(),
        std::fs::read(path).unwrap()
    );
}

#[test]
fn composed_panel_titles_are_anchored_at_the_upper_left() {
    let data = df!("x" => [1.0_f64, 2.0], "y" => [2.0_f64, 3.0]).unwrap();
    let mut chart = Chart::new(&data).title("(a) Panel");
    chart.x("x").y("y").line();
    let svg = String::from_utf8(
        Figure::grid(1, 1)
            .unwrap()
            .panel(0, 0, chart)
            .unwrap()
            .render_to_svg_bytes()
            .unwrap(),
    )
    .unwrap();
    let marker = ">(a) Panel</text>";
    let end = svg.find(marker).unwrap();
    let start = svg[..end].rfind("<text ").unwrap();

    assert!(svg[start..end].contains("text-anchor=\"start\""));
}

#[test]
fn composed_panel_subtitle_is_distinct_and_clear_of_title_and_panel() {
    let data = df!("x" => [1.0_f64, 2.0], "y" => [2.0_f64, 3.0]).unwrap();
    let mut chart = Chart::new(&data)
        .title("Panel title")
        .subtitle("Panel subtitle");
    chart.x("x").y("y").line();
    let svg = String::from_utf8(
        Figure::grid(1, 1)
            .unwrap()
            .panel(0, 0, chart)
            .unwrap()
            .render_to_svg_bytes()
            .unwrap(),
    )
    .unwrap();
    let title = svg
        .lines()
        .find(|line| line.contains(">Panel title</text>"))
        .unwrap();
    let subtitle = svg
        .lines()
        .find(|line| line.contains(">Panel subtitle</text>"))
        .unwrap();
    let clip = svg
        .lines()
        .find(|line| line.starts_with("<clipPath "))
        .unwrap();
    let title_y = attribute(title, "y").parse::<f64>().unwrap();
    let subtitle_y = attribute(subtitle, "y").parse::<f64>().unwrap();
    let panel_y = attribute(clip, "y").parse::<f64>().unwrap();

    assert_eq!(attribute(title, "font-size"), "20");
    assert_eq!(attribute(subtitle, "font-size"), "15");
    assert!(subtitle_y - title_y > 15.5);
    assert!(subtitle_y + 7.5 < panel_y);
}

#[test]
fn shared_figure_scatter_legend_rejects_null_hue_rows_before_begin() {
    let left_data = df!(
        "x" => [1.0_f64, 2.0],
        "y" => [1.0_f64, 2.0],
        "trace" => [Some("control"), None]
    )
    .unwrap();
    let right_data = df!(
        "x" => [1.0_f64, 2.0, 3.0],
        "y" => [2.0_f64, 3.0, 4.0],
        "trace" => [Some("active"), Some("active"), None]
    )
    .unwrap();
    let mut left = Chart::new(&left_data).hue("trace");
    left.x("x").y("y").scatter();
    let mut right = Chart::new(&right_data).hue("trace");
    right.x("x").y("y").scatter();
    let figure = Figure::grid(1, 2)
        .unwrap()
        .panel(0, 0, left)
        .unwrap()
        .panel(0, 1, right)
        .unwrap()
        .legend_policy(FigureLegendPolicy::Shared);
    let mut backend = LifecycleBackend::default();

    let error = figure.render_with_backend(&mut backend).unwrap_err();

    assert!(error.to_string().contains("required categorical column"));
    assert_eq!(backend.begins, 0);
}

#[test]
fn immediate_backend_receives_all_figure_grids_before_any_marks() {
    let data = estimate_data();
    let mut left = Chart::new(&data).group("trace").hue("trace");
    left.x("x").y("estimate").line();
    let mut right = Chart::new(&data);
    right.x("x").y("estimate").scatter();
    let figure = Figure::grid(1, 2)
        .unwrap()
        .panel(0, 0, left)
        .unwrap()
        .panel(0, 1, right)
        .unwrap();
    let mut backend = LifecycleBackend::default();

    figure.render_with_backend(&mut backend).unwrap();

    let last_grid = backend
        .events
        .iter()
        .rposition(|event| *event == RenderEvent::Grid)
        .unwrap();
    let first_marks = backend
        .events
        .iter()
        .position(|event| *event == RenderEvent::Marks)
        .unwrap();
    assert!(last_grid < first_marks, "events={:?}", backend.events);
}

#[test]
fn heterogeneous_figure_has_one_lifecycle_and_deterministic_clipped_svg() {
    let data = estimate_data();
    let mut line = Chart::new(&data).group("trace").hue("trace");
    line.x("x").y("estimate").line();
    let mut coefficient = Chart::new(&data)
        .interval_bounds("lower", "upper")
        .interval_semantics(confidence(InferentialScale::Coefficient));
    coefficient.x("estimate").y("term").coefficient();
    let figure = Figure::grid(1, 2)
        .unwrap()
        .panel(0, 0, line)
        .unwrap()
        .panel(0, 1, coefficient)
        .unwrap()
        .legend_policy(FigureLegendPolicy::PerPanel)
        .title("Model summary");
    let mut backend = LifecycleBackend::default();
    figure.render_with_backend(&mut backend).unwrap();
    assert_eq!((backend.begins, backend.ends), (1, 1));
    assert_eq!((backend.clips, backend.clip_ends), (2, 2));

    let dir = tempdir("figure");
    let first = dir.join("first.svg");
    let second = dir.join("second.svg");
    figure.render_to_svg(&first).unwrap();
    figure.render_to_svg(&second).unwrap();
    let svg = std::fs::read_to_string(first).unwrap();
    assert_eq!(svg, std::fs::read_to_string(second).unwrap());
    assert_eq!(svg.matches("<svg ").count(), 1);
    assert_eq!(svg.matches("<clipPath id=").count(), 2);
    assert!(svg.contains(">Model summary</text>"));
}

#[test]
fn invalid_layouts_fail_before_composed_backend_begin() {
    assert!(Figure::grid(0, 1).is_err());
    let data = estimate_data();
    let mut line = Chart::new(&data);
    line.x("x").y("estimate").line();
    let figure = Figure::grid(1, 1)
        .unwrap()
        .panel(0, 0, line)
        .unwrap()
        .size(100, 100);
    let mut backend = LifecycleBackend::default();
    assert!(figure.render_with_backend(&mut backend).is_err());
    assert_eq!(backend.begins, 0);
}

#[test]
fn invalid_panel_data_does_not_begin_figure_canvas() {
    let x = Series::new("x".into(), [1.0_f64, f64::INFINITY]);
    let y = Series::new("y".into(), [2.0_f64, 3.0]);
    let data = DataFrame::new(x.len(), vec![x.into(), y.into()]).unwrap();
    let mut scatter = Chart::new(&data);
    scatter.x("x").y("y").scatter();
    let figure = Figure::grid(1, 1).unwrap().panel(0, 0, scatter).unwrap();
    let mut backend = LifecycleBackend::default();

    let error = figure.render_with_backend(&mut backend).unwrap_err();

    assert!(error.to_string().contains("source row 1"));
    assert_eq!(backend.begins, 0);
}

#[test]
fn invalid_panel_interval_semantics_fail_before_composed_backend_begin() {
    let data = estimate_data();
    let mut scatter = Chart::new(&data)
        .interval_bounds("lower", "upper")
        .interval_semantics(confidence(InferentialScale::Response));
    scatter.x("x").y("estimate").scatter();
    let figure = Figure::grid(1, 1).unwrap().panel(0, 0, scatter).unwrap();
    let mut backend = LifecycleBackend::default();

    let error = figure.render_with_backend(&mut backend).unwrap_err();

    assert!(error
        .to_string()
        .contains("interval semantics are not consumed by Scatter"));
    assert_eq!(backend.begins, 0);
}

#[test]
fn facet_owned_panel_comparison_fails_before_composed_backend_begin() {
    let data = estimate_data();
    let mut bar = Chart::new(&data);
    bar.x("factor")
        .y("estimate")
        .add_comparisons([
            ComparisonAnnotation::new("before", "after", 0.01).in_facet("facet", "one")
        ])
        .bar();
    let figure = Figure::grid(1, 1).unwrap().panel(0, 0, bar).unwrap();
    let mut backend = LifecycleBackend::default();

    let error = figure.render_with_backend(&mut backend).unwrap_err();

    assert!(error
        .to_string()
        .contains("facet-owned significance comparisons require"));
    assert_eq!(backend.begins, 0);
}
