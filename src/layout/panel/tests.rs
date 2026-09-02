use super::*;
use crate::render::backend::Backend;

#[test]
fn continuous_auto_range_pads_when_data_hits_nice_bounds() {
    let range = continuous_auto_range((0.0, 10.0), &AxisSpec::default());

    assert!(
        range.0 < 0.0 && range.1 > 10.0,
        "continuous auto range should pad both data endpoints: {range:?}"
    );
}

#[test]
fn continuous_auto_range_preserves_finite_extreme_endpoints_when_padding_overflows() {
    let range = continuous_auto_range((-f64::MAX, f64::MAX), &AxisSpec::default());

    assert_eq!(range, (-f64::MAX, f64::MAX));
    let projection = Projection::new(range, range, (0.0, 100.0, 0.0, 100.0));
    assert_eq!(projection.project(0.0, 0.0), (50.0, 50.0));
}

#[test]
fn anchored_auto_range_keeps_zero_side_exact_for_positive_data() {
    let range = anchored_auto_range((0.0, 10.0), &AxisSpec::default(), 0.0);

    assert_eq!(range, (0.0, 10.5));
}

#[test]
fn anchored_auto_range_uses_relative_padding_for_tiny_positive_density() {
    let maximum = 2.4e-294;

    let range = anchored_auto_range((0.0, maximum), &AxisSpec::default(), 0.0);

    assert!(range.0.total_cmp(&0.0).is_eq());
    assert!(range.1 > maximum && range.1 < 1e-292, "{range:?}");
}

#[test]
fn anchored_auto_range_advances_adjacent_subnormal_density_endpoint() {
    let maximum = f64::from_bits(1);

    let range = anchored_auto_range((0.0, maximum), &AxisSpec::default(), 0.0);

    assert_eq!(range, (0.0, maximum.next_up()));
}

#[test]
fn continuous_auto_range_keeps_nice_ticks_inside_limits() {
    let range = continuous_auto_range((0.0, 10.0), &AxisSpec::default());
    let ticks = nice_ticks(range.0, range.1, AUTOMATIC_TICK_TARGET);

    assert!(
        ticks.iter().any(|tick| tick.abs() < f64::EPSILON)
            && ticks
                .iter()
                .any(|tick| (*tick - 10.0).abs() < f64::EPSILON)
            && range.0 < 0.0
            && range.1 > 10.0,
        "auto range should broaden limits while preserving nice endpoint ticks: range={range:?}, ticks={ticks:?}"
    );
}

#[test]
fn narrower_x_panel_uses_fewer_automatic_ticks_without_changing_resolved_range() {
    let theme = crate::Theme::Publication.resolve();
    let wide = Layout::new_with_axis_specs_in_panel_box(
        (0.0, 100.0),
        (0.0, 100.0),
        crate::layout::PanelBox::new(0.0, 400.0, 0.0, 400.0),
        "x".into(),
        "y".into(),
        &AxisSpec::default(),
        &AxisSpec::default(),
        &theme,
    );
    let narrow = Layout::new_with_axis_specs_in_panel_box(
        (0.0, 100.0),
        (0.0, 100.0),
        crate::layout::PanelBox::new(0.0, 40.0, 0.0, 400.0),
        "x".into(),
        "y".into(),
        &AxisSpec::default(),
        &AxisSpec::default(),
        &theme,
    );

    assert_eq!(wide.projection.x_range(), narrow.projection.x_range());
    assert!(narrow.x_axis.ticks.len() < wide.x_axis.ticks.len());
}

#[test]
fn shorter_y_panel_uses_fewer_automatic_ticks_without_changing_resolved_range() {
    let theme = crate::Theme::Publication.resolve();
    let tall = Layout::new_with_axis_specs_in_panel_box(
        (0.0, 100.0),
        (0.0, 100.0),
        crate::layout::PanelBox::new(0.0, 400.0, 0.0, 400.0),
        "x".into(),
        "y".into(),
        &AxisSpec::default(),
        &AxisSpec::default(),
        &theme,
    );
    let short = Layout::new_with_axis_specs_in_panel_box(
        (0.0, 100.0),
        (0.0, 100.0),
        crate::layout::PanelBox::new(0.0, 400.0, 0.0, 40.0),
        "x".into(),
        "y".into(),
        &AxisSpec::default(),
        &AxisSpec::default(),
        &theme,
    );

    assert_eq!(tall.projection.y_range(), short.projection.y_range());
    assert!(short.y_axis.ticks.len() < tall.y_axis.ticks.len());
}

#[test]
fn wider_numeric_labels_reduce_automatic_x_tick_density() {
    let theme = crate::Theme::Publication.resolve();
    let panel_box = crate::layout::PanelBox::new(0.0, 100.0, 0.0, 100.0);
    let wider_label_spec = AxisSpec::default().limits(0.0, 100.0);
    let shorter_label_spec = AxisSpec::default().limits(0.0, 1.0);
    let wider_labels = Layout::new_with_axis_specs_in_panel_box(
        (0.0, 100.0),
        (0.0, 1.0),
        panel_box,
        "x".into(),
        "y".into(),
        &wider_label_spec,
        &shorter_label_spec,
        &theme,
    );
    let shorter_labels = Layout::new_with_axis_specs_in_panel_box(
        (0.0, 1.0),
        (0.0, 1.0),
        panel_box,
        "x".into(),
        "y".into(),
        &shorter_label_spec,
        &shorter_label_spec,
        &theme,
    );

    assert!(
        wider_labels.x_axis.ticks.len() < shorter_labels.x_axis.ticks.len(),
        "wider ticks: {:?}, shorter ticks: {:?}",
        wider_labels
            .x_axis
            .ticks
            .iter()
            .map(|tick| tick.label.as_str())
            .collect::<Vec<_>>(),
        shorter_labels
            .x_axis
            .ticks
            .iter()
            .map(|tick| tick.label.as_str())
            .collect::<Vec<_>>()
    );
}

#[test]
fn automatic_ticks_keep_target_five_candidate_when_labels_fit() {
    let theme = crate::Theme::Publication.resolve();
    let spec = AxisSpec::default().limits(0.0, 100.0);
    let layout = Layout::new_with_axis_specs_in_panel_box(
        (0.0, 100.0),
        (0.0, 100.0),
        crate::layout::PanelBox::new(0.0, 400.0, 0.0, 400.0),
        "x".into(),
        "y".into(),
        &spec,
        &spec,
        &theme,
    );

    assert_eq!(
        layout
            .x_axis
            .ticks
            .iter()
            .map(|tick| tick.position)
            .collect::<Vec<_>>(),
        [0.0, 25.0, 50.0, 75.0, 100.0]
    );
}

#[test]
fn resolved_tick_label_size_controls_automatic_tick_measurement_and_drawing() {
    let spec = AxisSpec::default().limits(0.0, 100.0);
    let panel = crate::layout::PanelBox::new(0.0, 400.0, 0.0, 400.0);
    let default_theme = crate::Theme::Publication.resolve();
    let default_layout = Layout::new_with_axis_specs_in_panel_box(
        (0.0, 100.0),
        (0.0, 100.0),
        panel,
        String::new(),
        String::new(),
        &spec,
        &spec,
        &default_theme,
    );
    let mut large_tick_theme = default_theme.clone();
    large_tick_theme.typography.tick_label.font_size = 96;
    let large_tick_layout = Layout::new_with_axis_specs_in_panel_box(
        (0.0, 100.0),
        (0.0, 100.0),
        panel,
        String::new(),
        String::new(),
        &spec,
        &spec,
        &large_tick_theme,
    );
    let empty_tick_layout = Layout::new_with_ticks_and_axis_specs_in_panel_box(
        (0.0, 100.0),
        (0.0, 100.0),
        panel,
        String::new(),
        String::new(),
        Vec::new(),
        Vec::new(),
        &spec,
        &spec,
        &large_tick_theme,
    );

    assert!(large_tick_layout.x_axis.ticks.len() < default_layout.x_axis.ticks.len());
    assert_eq!(
        empty_tick_layout.x_axis.ticks.len(),
        large_tick_layout.x_axis.ticks.len()
    );

    let mut backend = crate::render::svg_backend::SvgBackend::in_memory(400, 400);
    backend.begin(400, 400).expect("begin SVG");
    large_tick_layout
        .draw_decorations(&large_tick_theme, &mut backend)
        .expect("draw layout decorations");
    backend.end().expect("end SVG");
    let svg = String::from_utf8(backend.into_bytes().expect("finished SVG")).expect("UTF-8 SVG");
    assert!(svg.contains("font-size=\"96\""));
}

#[test]
fn frame_and_ticks_draw_with_their_independent_resolved_strokes() {
    let mut theme = crate::Theme::Science.resolve();
    theme.axes.grid = None;
    theme.axes.frame.as_mut().unwrap().stroke.paint = crate::render::backend::Rgba::rgb(1, 2, 3);
    theme.axes.ticks.as_mut().unwrap().stroke.paint =
        crate::render::backend::Rgba::rgb(161, 178, 195);
    let layout = Layout::new_with_axis_specs_in_panel_box(
        (0.0, 10.0),
        (0.0, 10.0),
        crate::layout::PanelBox::new(40.0, 360.0, 40.0, 360.0),
        String::new(),
        String::new(),
        &AxisSpec::default(),
        &AxisSpec::default(),
        &theme,
    );
    let expected_ticks = layout.x_axis.ticks.len() + layout.y_axis.ticks.len();
    let mut backend = crate::render::svg_backend::SvgBackend::in_memory(400, 400);
    backend.begin(400, 400).unwrap();

    layout.draw_decorations(&theme, &mut backend).unwrap();
    backend.end().unwrap();

    let svg = String::from_utf8(backend.into_bytes().unwrap()).unwrap();
    assert_eq!(svg.matches("stroke=\"#010203\"").count(), 6);
    assert_eq!(svg.matches("stroke=\"#A1B2C3\"").count(), expected_ticks);
}

#[test]
fn absent_frame_and_ticks_leave_axis_text_without_panicking() {
    let mut theme = crate::Theme::Publication.resolve();
    theme.axes.frame = None;
    theme.axes.ticks = None;
    theme.axes.grid = None;
    let layout = Layout::new_with_axis_specs_in_panel_box(
        (0.0, 10.0),
        (0.0, 10.0),
        crate::layout::PanelBox::new(40.0, 360.0, 40.0, 360.0),
        "horizontal".into(),
        "vertical".into(),
        &AxisSpec::default(),
        &AxisSpec::default(),
        &theme,
    );
    let mut backend = crate::render::svg_backend::SvgBackend::in_memory(400, 400);
    backend.begin(400, 400).unwrap();

    layout.draw_decorations(&theme, &mut backend).unwrap();
    backend.end().unwrap();

    let svg = String::from_utf8(backend.into_bytes().unwrap()).unwrap();
    assert!(svg.contains(">horizontal</text>") && svg.contains(">vertical</text>"));
    assert!(!svg.contains(" stroke="));
}

#[test]
fn automatic_ticks_fall_back_to_one_finite_midpoint_when_no_labels_fit() {
    let theme = crate::Theme::Publication.resolve();
    let projection = Projection::new((0.0, 10.0), (0.0, 10.0), (0.0, 0.0, 0.0, 0.0));

    let ticks = automatic_numeric_ticks((0.0, 10.0), &projection, AxisSide::Bottom, &theme);

    assert_eq!(
        ticks
            .iter()
            .map(|tick| (tick.position, tick.label.as_str()))
            .collect::<Vec<_>>(),
        [(5.0, "5")]
    );
}

#[test]
fn fitted_automatic_candidates_never_exceed_the_internal_maximum() {
    let theme = crate::Theme::Publication.resolve();
    let projection = Projection::new((0.0, 9.0), (0.0, 1.0), (0.0, 1_000.0, 0.0, 100.0));
    let ticks = (0..10)
        .map(|position| AxisTick::new(f64::from(position), position.to_string()))
        .collect();

    let fitted = fit_automatic_tick_candidates(
        ticks,
        &projection,
        AxisSide::Bottom,
        &theme,
        AxisTick::new(4.5, "4.5"),
    );

    assert_eq!(fitted.len(), AUTOMATIC_TICK_TARGET);
}

#[test]
fn automatic_ticks_never_exceed_the_internal_maximum() {
    let theme = crate::Theme::Publication.resolve();
    let spec = AxisSpec::default().limits(5.0, 30.0);
    let layout = Layout::new_with_axis_specs_in_panel_box(
        (5.0, 30.0),
        (0.0, 1.0),
        crate::layout::PanelBox::new(0.0, 1_000.0, 0.0, 100.0),
        "x".into(),
        "y".into(),
        &spec,
        &AxisSpec::default(),
        &theme,
    );

    assert!(layout.x_axis.ticks.len() <= AUTOMATIC_TICK_TARGET);
}

#[test]
fn panel_lane_filter_removes_automatic_labels_that_cross_its_edges() {
    let theme = crate::Theme::Publication.resolve();
    let projection = Projection::new((0.0, 100.0), (0.0, 1.0), (0.0, 100.0, 0.0, 100.0));
    let ticks = vec![
        AxisTick::new(0.0, "0"),
        AxisTick::new(50.0, "50"),
        AxisTick::new(100.0, "100"),
    ];

    let retained =
        retain_automatic_ticks_inside_panel(ticks, &projection, AxisSide::Bottom, &theme);

    assert_eq!(
        retained
            .iter()
            .map(|tick| tick.position)
            .collect::<Vec<_>>(),
        [50.0]
    );
}

#[test]
fn y_axis_inset_preserves_measured_tick_width_and_label_gap() {
    let mut theme = crate::Theme::Publication.resolve();
    let tick_width = theme
        .measure_text(
            "locus_0 aa",
            &theme.typography.tick_label,
            crate::layout::PhysicalTextAlignment::End,
            "y-axis tick label",
            false,
        )
        .unwrap()
        .width();
    theme.axes.spacing.y_title_edge_padding = 9.0;
    theme.axes.spacing.y_title_tick_label_gap = 13.0;
    theme.axes.spacing.y_tick_label_gap = 4.0;
    let inset = y_axis_left_inset_for_tick_labels(tick_width, 5, 17, &theme.axes.spacing);

    let expected = 9.0 + 17.0 + 13.0 + tick_width + 5.0 + 4.0;
    assert!((inset - expected).abs() < f64::EPSILON);
}

#[test]
fn categorical_y_ticks_preserve_the_supplied_vector_and_row_range() {
    let theme = crate::Theme::Publication.resolve();
    let y_ticks = (0..7)
        .map(|index| AxisTick::new(f64::from(index) + 0.5, format!("row {index}")))
        .collect::<Vec<_>>();
    let expected_ticks = y_ticks
        .iter()
        .map(|tick| (tick.position, tick.label.clone()))
        .collect::<Vec<_>>();
    let layout = Layout::new_with_ticks_and_axis_specs_in_panel_box(
        (0.0, 10.0),
        (0.0, 7.0),
        crate::layout::PanelBox::new(0.0, 640.0, 0.0, 480.0),
        "x".into(),
        "y".into(),
        Vec::new(),
        y_ticks,
        &AxisSpec::default(),
        &AxisSpec::default(),
        &theme,
    );

    assert_eq!(
        (
            layout.projection.y_range(),
            layout
                .y_axis
                .ticks
                .iter()
                .map(|tick| (tick.position, tick.label.clone()))
                .collect::<Vec<_>>()
        ),
        ((0.0, 7.0), expected_ticks)
    );
}

#[test]
fn projection_preserves_out_of_range_canvas_coordinates() {
    let projection = Projection::new((0.0, 1.0), (0.0, 1.0), (10.0, 20.0, 30.0, 50.0));

    assert_eq!(projection.project(-0.5, 1.5), (5.0, 60.0));
}

#[test]
fn projection_scales_opposite_sign_extreme_axis_boundaries_without_overflow() {
    let projection = Projection::new(
        (-f64::MAX, f64::MAX),
        (-1.0, 1.0),
        (10.0, 110.0, 20.0, 220.0),
    );

    assert_eq!(projection.project(-f64::MAX, -1.0), (10.0, 20.0));
    assert_eq!(projection.project(f64::MAX, 1.0), (110.0, 220.0));
}

#[test]
fn nice_ticks_return_adjacent_subnormal_endpoints_without_zero_step() {
    let upper = f64::from_bits(1);

    assert_eq!(nice_ticks(0.0, upper, 5), [0.0, upper]);
    assert_eq!(nice_range(0.0, upper, 5), (0.0, upper));
    assert!(labels_are_unique(&distinct_tick_labels(&[0.0, upper])));
}

#[test]
fn adjacent_ordinary_and_large_ticks_keep_both_distinguishable_endpoints() {
    for lower in [1.0_f64, 1e300, f64::MAX.next_down()] {
        let upper = lower.next_up();
        let ticks = nice_ticks(lower, upper, 5);
        let labels = distinct_tick_labels(&ticks);

        assert_eq!(ticks, [lower, upper], "{lower}..{upper}");
        assert_ne!(labels[0], labels[1], "{lower}..{upper}: {labels:?}");
    }
}

#[test]
fn opposite_extreme_ticks_use_finite_endpoints_and_distinct_labels() {
    let ticks = nice_ticks(-f64::MAX, f64::MAX, 5);
    let labels = distinct_tick_labels(&ticks);

    assert_eq!(ticks, [-f64::MAX, f64::MAX]);
    assert_ne!(labels[0], labels[1]);
}

#[test]
fn numeric_layout_preserves_explicit_adjacent_range_and_ticks() {
    let theme = crate::Theme::Publication.resolve();
    let upper = f64::from_bits(1);
    let spec = AxisSpec::default().limits(0.0, upper);
    let layout = Layout::new_with_axis_specs(
        (0.0, upper),
        (0.0, upper),
        640,
        480,
        "x".into(),
        "y".into(),
        &spec,
        &spec,
        &theme,
    );

    assert_eq!(layout.projection.x_range(), (0.0, upper));
    assert_eq!(
        layout
            .x_axis
            .ticks
            .iter()
            .map(|tick| tick.position)
            .collect::<Vec<_>>(),
        [0.0, upper]
    );
    assert!(layout.projection.project(upper, upper).0.is_finite());
}
