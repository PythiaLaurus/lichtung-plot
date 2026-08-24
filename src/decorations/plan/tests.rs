use super::*;
use crate::test_support::Fixture;

fn line_data(labels: [&str; 4]) -> Fixture {
    crate::test_support::test_data!(
        "x" => [0.0_f64, 1.0, 0.0, 1.0],
        "y" => [8.0_f64, 10.0, 1.0, 2.0],
        "low" => [7.0_f64, 9.0, 0.5, 1.5],
        "high" => [9.0_f64, 11.0, 1.5, 2.5],
        "hue" => labels,
    )
    .unwrap()
}

fn line_chart(data: &Fixture) -> Plot<crate::kind::Line> {
    Plot::line(
        crate::test_support::numeric(data, "x"),
        crate::test_support::numeric(data, "y"),
    )
    .group(crate::test_support::category(data, "hue"))
    .hue(crate::test_support::category(data, "hue"))
    .band(crate::Bounds::new(
        crate::test_support::numeric(data, "low"),
        crate::test_support::numeric(data, "high"),
    ))
}

fn dense_bar_chart(data: &Fixture, horizontal: bool) -> Plot {
    let category = crate::test_support::category(data, "category");
    let input = crate::BarInput::estimates(crate::test_support::numeric(data, "value"));
    if horizontal {
        Plot::horizontal_bar(category, input)
            .series(crate::test_support::category(data, "hue"))
            .erase()
    } else {
        Plot::bar(category, input)
            .series(crate::test_support::category(data, "hue"))
            .erase()
    }
}

fn dense_horizontal_data() -> Fixture {
    signed_dense_bar_data(1.0)
}

fn signed_dense_bar_data(sign: f64) -> Fixture {
    crate::test_support::test_data!(
        "category" => ["a", "a", "b", "b", "c", "c", "d", "d", "e", "e", "f", "f"],
        "value" => [
            10.0_f64, 9.0, 10.0, 9.0, 10.0, 9.0, 10.0, 9.0, 10.0, 9.0, 10.0, 9.0,
        ].map(|value| value * sign),
        "hue" => [
            "one", "two", "one", "two", "one", "two", "one", "two", "one", "two", "one", "two",
        ],
    )
    .unwrap()
}

#[test]
fn auto_legend_avoids_recorded_line_and_ribbon_geometry() {
    let data = line_data(["upper", "upper", "lower", "lower"]);
    let chart = line_chart(&data);
    let cell = PanelBox::new(60.0, 440.0, 50.0, 310.0);

    let plan = plan_panel(&chart, PlotKind::Line, cell, LegendRole::Panel).unwrap();
    let legend = plan.legend.as_ref().unwrap();
    let final_chart = plan.chart_for_render(&chart);
    let mut geometry = GeometryBackend::default();
    let result = render(
        &final_chart,
        PlotKind::Line,
        PlotRenderContext::new(crate::data::PlotData::new(&final_chart), plan.panel),
        &mut geometry,
    )
    .unwrap();
    record_brackets(&final_chart, &result, &mut geometry).unwrap();
    let margin = legend.legend.metrics.font_size.max(6.0);

    assert!(
        overlap_score_within_panel(
            legend.bounds,
            geometry.bounds(),
            margin,
            result.layout.projection.panel_box(),
        ) <= f64::EPSILON
    );
}

#[test]
fn outside_legend_reservation_uses_measured_label_width_and_stays_in_cell() {
    let short_data = line_data(["a", "a", "b", "b"]);
    let long_data = line_data([
        "a very long treatment label",
        "a very long treatment label",
        "another long control label",
        "another long control label",
    ]);
    let cell = PanelBox::new(40.0, 620.0, 50.0, 390.0);
    let short = line_chart(&short_data).legend(LegendPolicy::Visible(LegendPosition::OutsideRight));
    let long = line_chart(&long_data).legend(LegendPolicy::Visible(LegendPosition::OutsideRight));

    let short = plan_panel(&short, PlotKind::Line, cell, LegendRole::Panel).unwrap();
    let long = plan_panel(&long, PlotKind::Line, cell, LegendRole::Panel).unwrap();
    let long_legend = long.legend.unwrap().bounds;

    assert!(long.panel.x1 < short.panel.x1);
    assert!(long_legend.left > long.panel.x1);
    assert!(long_legend.right <= cell.x1);
}

#[test]
fn outside_legend_remeasures_and_reserves_newly_wrapped_axis_title() {
    let data = line_data([
        "first deliberately wide legend entry",
        "first deliberately wide legend entry",
        "second deliberately wide legend entry",
        "second deliberately wide legend entry",
    ]);
    let cell = PanelBox::new(40.0, 700.0, 50.0, 390.0);
    let initial = line_chart(&data).legend(LegendPolicy::Visible(LegendPosition::OutsideRight));
    let initial = plan_panel(&initial, PlotKind::Line, cell, LegendRole::Panel).unwrap();
    let font_size = line_chart(&data).palette().label_font_size;
    let label = (1..=200)
        .map(|length| vec!["x"; length].join(" "))
        .find(|label| {
            crate::layout::TextBlock::layout(label, font_size, cell.width(), "x-axis title")
                .unwrap()
                .lines()
                .len()
                == 1
                && crate::layout::TextBlock::layout(
                    label,
                    font_size,
                    initial.panel.width(),
                    "x-axis title",
                )
                .unwrap()
                .lines()
                .len()
                    > 1
        })
        .expect("legend reservation should expose an axis-title rewrap threshold");
    let chart = line_chart(&data)
        .x_label(label)
        .legend(LegendPolicy::Visible(LegendPosition::OutsideRight));

    let plan = plan_panel(&chart, PlotKind::Line, cell, LegendRole::Panel).unwrap();

    assert!((plan.panel.x1 - initial.panel.x1).abs() <= f64::EPSILON);
    assert!(plan.panel.y0 > initial.panel.y0);
}

#[test]
fn standalone_plan_reserves_wrapped_explicit_x_tick_height() {
    let data = crate::test_support::test_data!(
        "x" => [0.0_f64, 1.0, 2.0],
        "y" => [1.0_f64, 2.0, 3.0],
    )
    .unwrap();
    let chart = Plot::scatter(
        crate::test_support::numeric(&data, "x"),
        crate::test_support::numeric(&data, "y"),
    )
    .x_limits(0.0, 2.0)
    .x_ticks(vec![
        crate::layout::AxisTick::new(0.0, "first wide tick"),
        crate::layout::AxisTick::new(1.0, "second broad mark"),
        crate::layout::AxisTick::new(2.0, "third long label"),
    ]);
    let cell = PanelBox::new(60.0, 220.0, 30.0, 180.0);

    let plan = plan_panel(&chart, PlotKind::Scatter, cell, LegendRole::Panel).unwrap();

    assert!(plan.panel.y0 > cell.y0);
}

#[test]
fn hidden_outside_and_shared_suppression_leave_automatic_domains_unchanged() {
    let data = line_data(["upper", "upper", "lower", "lower"]);
    let cell = PanelBox::new(60.0, 500.0, 50.0, 350.0);
    let auto = line_chart(&data);
    let outside = line_chart(&data).legend(LegendPolicy::Visible(LegendPosition::OutsideRight));
    let hidden = line_chart(&data).legend(LegendPolicy::Hidden);

    let auto = plan_panel(&auto, PlotKind::Line, cell, LegendRole::Panel).unwrap();
    let outside = plan_panel(&outside, PlotKind::Line, cell, LegendRole::Panel).unwrap();
    let hidden = plan_panel(&hidden, PlotKind::Line, cell, LegendRole::Panel).unwrap();
    let shared = plan_panel(
        &line_chart(&data),
        PlotKind::Line,
        cell,
        LegendRole::Suppressed,
    )
    .unwrap();

    assert!(auto.planned_x_limits().is_some() || auto.planned_y_limits().is_some());
    assert_eq!(
        outside.result.layout.projection.x_range(),
        hidden.result.layout.projection.x_range()
    );
    assert_eq!(
        outside.result.layout.projection.y_range(),
        hidden.result.layout.projection.y_range()
    );
    assert_eq!(
        shared.result.layout.projection.x_range(),
        hidden.result.layout.projection.x_range()
    );
    assert_eq!(
        shared.result.layout.projection.y_range(),
        hidden.result.layout.projection.y_range()
    );
}

#[test]
fn scatter_with_automatic_y_limits_keeps_top_right_default() {
    let data = crate::test_support::test_data!(
        "x" => [0.0_f64, 0.5, 9.5, 10.0],
        "y" => [0.0_f64, 0.5, 9.5, 10.0],
        "hue" => ["a", "a", "b", "b"],
    )
    .unwrap();
    let visible = Plot::scatter(
        crate::test_support::numeric(&data, "x"),
        crate::test_support::numeric(&data, "y"),
    )
    .hue(crate::test_support::category(&data, "hue"));
    let hidden = visible.clone().legend(LegendPolicy::Hidden);
    let cell = PanelBox::new(60.0, 500.0, 50.0, 350.0);

    let visible = plan_panel(&visible, PlotKind::Scatter, cell, LegendRole::Panel).unwrap();
    let hidden = plan_panel(&hidden, PlotKind::Scatter, cell, LegendRole::Panel).unwrap();

    let legend = visible.legend.as_ref().unwrap();
    let top_right = legend
        .legend
        .inside_candidates(visible.result.layout.projection.pixel_box())[0];

    assert!(visible.planned_x_limits().is_none());
    assert!(visible.planned_y_limits().is_some());
    assert_eq!(legend.bounds, top_right);
    assert!(
        visible.result.layout.projection.y_range().1 > hidden.result.layout.projection.y_range().1
    );
}

#[test]
fn right_edge_legend_expands_generic_continuous_x_axis() {
    let data = crate::test_support::test_data!(
        "x" => [8.0_f64, 10.0, 8.0, 10.0],
        "y" => [5.0_f64; 4],
        "hue" => ["a", "a", "b", "b"],
    )
    .unwrap();
    let chart = Plot::line(
        crate::test_support::numeric(&data, "x"),
        crate::test_support::numeric(&data, "y"),
    )
    .group(crate::test_support::category(&data, "hue"))
    .hue(crate::test_support::category(&data, "hue"))
    .legend(LegendPolicy::Visible(LegendPosition::InsideRight));
    let cell = PanelBox::new(60.0, 500.0, 50.0, 350.0);

    let plan = plan_panel(&chart, PlotKind::Line, cell, LegendRole::Panel).unwrap();
    let legend = plan.legend.as_ref().unwrap().bounds;
    let rightmost_point = plan.result.layout.projection.project(10.0, 5.0).0;

    assert!(plan.planned_x_limits().is_some());
    assert!(rightmost_point < legend.left);
}

#[test]
fn auto_legend_expands_continuous_x_for_dense_horizontal_categorical_scene() {
    let data = dense_horizontal_data();
    let chart = dense_bar_chart(&data, true);
    let cell = PanelBox::new(90.0, 500.0, 50.0, 310.0);

    let mut initial_geometry = GeometryBackend::default();
    let initial = render(
        &chart,
        PlotKind::HorizontalBar,
        PlotRenderContext::new(crate::data::PlotData::new(&chart), cell),
        &mut initial_geometry,
    )
    .unwrap();
    let initial_legend = Legend::new(
        "hue".to_string(),
        initial.legend_entries,
        f64::from(chart.width()),
        f64::from(chart.height()),
    );
    let margin = initial_legend.metrics.font_size.max(6.0);
    assert!(
        initial_legend
            .inside_candidates(initial.layout.projection.pixel_box())
            .iter()
            .all(|bounds| {
                overlap_score_within_panel(
                    *bounds,
                    initial_geometry.bounds(),
                    margin,
                    initial.layout.projection.panel_box(),
                ) > 0.0
            }),
        "fixture must collide at every initial corner"
    );

    let plan = plan_panel(&chart, PlotKind::HorizontalBar, cell, LegendRole::Panel).unwrap();
    let legend = plan.legend.as_ref().unwrap();
    let mut final_geometry = GeometryBackend::default();
    let final_chart = plan.chart_for_render(&chart);
    let final_result = render(
        &final_chart,
        PlotKind::HorizontalBar,
        PlotRenderContext::new(crate::data::PlotData::new(&final_chart), plan.panel),
        &mut final_geometry,
    )
    .unwrap();

    assert!(plan.planned_x_limits().is_some());
    assert!(plan.planned_y_limits().is_none());
    assert_eq!(
        final_result.layout.projection.x_range().0.to_bits(),
        0.0_f64.to_bits()
    );
    assert_eq!(
        legend.bounds,
        legend
            .legend
            .inside_candidates(final_result.layout.projection.pixel_box())[0]
    );
    assert!(legend_fits_panel(
        legend.bounds,
        final_result.layout.projection.panel_box()
    ));
    assert!(
        overlap_score_within_panel(
            legend.bounds,
            final_geometry.bounds(),
            margin,
            final_result.layout.projection.panel_box(),
        ) <= f64::EPSILON
    );
}

#[test]
fn negative_horizontal_bars_keep_zero_maximum_with_deterministic_top_left_fallback() {
    let data = signed_dense_bar_data(-1.0);
    let chart = dense_bar_chart(&data, true);
    let cell = PanelBox::new(90.0, 500.0, 50.0, 310.0);

    let first = plan_panel(&chart, PlotKind::HorizontalBar, cell, LegendRole::Panel).unwrap();
    let second = plan_panel(&chart, PlotKind::HorizontalBar, cell, LegendRole::Panel).unwrap();
    let legend = first.legend.as_ref().unwrap();
    let top_left = legend
        .legend
        .inside_candidates(first.result.layout.projection.pixel_box())[1];

    assert_eq!(
        first.result.layout.projection.x_range().1.to_bits(),
        0.0_f64.to_bits()
    );
    assert_eq!(legend.bounds, top_left);
    assert_eq!(first.planned_x_limits(), second.planned_x_limits());
    assert_eq!(legend.bounds, second.legend.unwrap().bounds);
}

#[test]
fn positive_vertical_bars_keep_zero_minimum_at_top_right() {
    let data = signed_dense_bar_data(1.0);
    let chart = dense_bar_chart(&data, false);
    let cell = PanelBox::new(90.0, 500.0, 50.0, 310.0);

    let plan = plan_panel(&chart, PlotKind::Bar, cell, LegendRole::Panel).unwrap();
    let legend = plan.legend.as_ref().unwrap();
    let top_right = legend
        .legend
        .inside_candidates(plan.result.layout.projection.pixel_box())[0];

    assert_eq!(
        plan.result.layout.projection.y_range().0.to_bits(),
        0.0_f64.to_bits()
    );
    assert_eq!(legend.bounds, top_right);
    assert!(plan.planned_y_limits().is_some());
}

#[test]
fn negative_vertical_bars_keep_zero_maximum_with_bottom_right_fallback() {
    let data = signed_dense_bar_data(-1.0);
    let chart = dense_bar_chart(&data, false);
    let cell = PanelBox::new(90.0, 500.0, 50.0, 310.0);

    let plan = plan_panel(&chart, PlotKind::Bar, cell, LegendRole::Panel).unwrap();
    let legend = plan.legend.as_ref().unwrap();
    let bottom_right = legend
        .legend
        .inside_candidates(plan.result.layout.projection.pixel_box())[2];

    assert_eq!(
        plan.result.layout.projection.y_range().1.to_bits(),
        0.0_f64.to_bits()
    );
    assert_eq!(legend.bounds, bottom_right);
    assert!(plan.planned_y_limits().is_some());
}

#[test]
fn automatic_clearance_does_not_expand_categorical_domains() {
    let data = dense_horizontal_data();
    let chart = dense_bar_chart(&data, true);
    let cell = PanelBox::new(90.0, 500.0, 50.0, 310.0);
    let mut geometry = GeometryBackend::default();
    let mut result = render(
        &chart,
        PlotKind::HorizontalBar,
        PlotRenderContext::new(crate::data::PlotData::new(&chart), cell),
        &mut geometry,
    )
    .unwrap();
    result.x_domain = crate::spec::AxisDomain::Categorical;
    result.y_domain = crate::spec::AxisDomain::Categorical;
    let legend = Legend::new(
        "hue".to_string(),
        result.legend_entries.clone(),
        f64::from(chart.width()),
        f64::from(chart.height()),
    );

    assert!(automatic_clearance(
        PlotKind::HorizontalBar,
        &chart,
        &result,
        &legend,
        geometry.bounds(),
        PlotKind::HorizontalBar.legend_default(false),
        0,
    )
    .is_none());
}
