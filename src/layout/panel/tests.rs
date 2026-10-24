use super::*;
use crate::render::backend::Backend;

#[test]
fn projected_span_validation_preserves_source_intent_without_reprojection_or_requantization() {
    let projection = Projection::new((0.0, 1.0), (0.0, 1.0), (0.0, 100.0, 0.0, 100.0));
    for axis in [AxisSide::Bottom, AxisSide::Left] {
        for (source, nonzero, collapse) in [
            ((0.0, 1e-9), true, true),
            ((1e-9, 0.0), true, true),
            ((0.5, 0.5), true, true),
            ((0.0, -0.0), false, false),
            ((0.5, 0.5), false, false),
            ((0.0, 1.0 / (100.0 * 1024.0)), true, false),
            ((0.5, 1.5), true, false),
            ((2.0, 3.0), true, false),
            ((2.0, 2.0 + 1e-9), true, false),
            ((2.0, 2.0), true, false),
        ] {
            let project = |value| match axis {
                AxisSide::Bottom => projection.project(value, 0.0).0,
                AxisSide::Left => projection.project(0.0, value).1,
            };
            let projected = (project(source.0), project(source.1));
            let calls = Projection::test_projection_calls();
            let conversions =
                crate::render::coordinates::SPATIAL_CONVERSIONS.with(std::cell::Cell::get);
            let result = projection.validate_span(axis, source, projected, nonzero, "test polygon");
            assert_eq!(result.is_err(), collapse, "{source:?}: {result:?}");
            if let Err(error) = result {
                assert!(matches!(error, crate::Error::Render(_)), "{error}");
                assert!(error.to_string().contains("test polygon"), "{error}");
            }
            assert_eq!(calls, Projection::test_projection_calls());
            assert_eq!(
                conversions,
                crate::render::coordinates::SPATIAL_CONVERSIONS.with(std::cell::Cell::get)
            );
        }
    }
    let outside = (2.0, 2.0 + 1e-9);
    let projected = (
        projection.project(outside.0, 0.5).0,
        projection.project(outside.1, 0.5).0,
    );
    projection
        .validate_span(
            AxisSide::Bottom,
            outside,
            projected,
            true,
            "outside interval",
        )
        .unwrap();
    let error = crate::render::backend::validate_line_points(&[
        crate::render::backend::Point2D::new(projected.0, 50.0),
        crate::render::backend::Point2D::new(projected.1, 50.0),
    ])
    .unwrap_err();
    assert!(error.to_string().contains("collapse"), "{error}");
}

#[test]
fn continuous_auto_range_preserves_finite_extreme_endpoints_when_padding_overflows() {
    let range = continuous_auto_range((-f64::MAX, f64::MAX), &AxisSpec::default()).unwrap();

    assert_eq!(range, (-f64::MAX, f64::MAX));
    let projection = Projection::new(range, range, (0.0, 100.0, 0.0, 100.0));
    assert_eq!(projection.project(0.0, 0.0), (50.0, 50.0));
}

#[test]
fn anchored_auto_range_keeps_zero_side_exact_for_positive_data() {
    let range = anchored_auto_range((0.0, 10.0), &AxisSpec::default(), 0.0).unwrap();

    assert_eq!(range, (0.0, 10.5));
}

#[test]
fn anchored_auto_range_uses_relative_padding_for_tiny_positive_density() {
    let maximum = 2.4e-294;

    let range = anchored_auto_range((0.0, maximum), &AxisSpec::default(), 0.0).unwrap();

    assert!(range.0.total_cmp(&0.0).is_eq());
    assert!(range.1 > maximum && range.1 < 1e-292, "{range:?}");
}

#[test]
fn anchored_auto_range_advances_adjacent_subnormal_density_endpoint() {
    let maximum = f64::from_bits(1);

    let range = anchored_auto_range((0.0, maximum), &AxisSpec::default(), 0.0).unwrap();

    assert_eq!(range, (0.0, maximum.next_up()));
}

#[test]
fn continuous_auto_range_keeps_nice_ticks_inside_limits() {
    let range = continuous_auto_range((0.0, 10.0), &AxisSpec::default()).unwrap();
    let ticks = nice_ticks(range.0, range.1, 5);

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
fn exact_tick_configuration_validates_count_before_fonts() {
    for count in [0_u32, 1, 10_000, 10_001] {
        let spec = AxisSpec {
            provenance: RangeProvenance::Automatic,
            limits: None,
            generation: crate::TickGeneration::automatic(),
            category_labels: crate::CategoryLabelSelection::all(),
            format: None,
            numeric_source: None,
            exact_ticks: Some(
                (0..count)
                    .map(|index| AxisTick::new(f64::from(index), ""))
                    .collect(),
            ),
        };
        assert_eq!(spec.validate("x-axis").is_ok(), count > 0, "count={count}");
    }
}

#[test]
fn exact_tick_configuration_validates_positions_without_a_display_domain() {
    for positions in [
        vec![f64::NAN],
        vec![f64::INFINITY],
        vec![f64::NEG_INFINITY],
        vec![1.0, 1.0],
        vec![1.0, 0.0],
        vec![-0.0, 0.0],
    ] {
        let spec = AxisSpec {
            provenance: RangeProvenance::Automatic,
            limits: None,
            generation: crate::TickGeneration::automatic(),
            category_labels: crate::CategoryLabelSelection::all(),
            format: None,
            numeric_source: None,
            exact_ticks: Some(
                positions
                    .into_iter()
                    .map(|position| AxisTick::new(position, "verbatim"))
                    .collect(),
            ),
        };
        assert!(matches!(
            spec.validate("y-axis"),
            Err(crate::Error::Data(_))
        ));
    }
}

#[test]
fn exact_tick_configuration_preserves_empty_and_duplicate_labels_and_checks_known_limits() {
    let mut spec = AxisSpec {
        provenance: RangeProvenance::Automatic,
        limits: None,
        generation: crate::TickGeneration::automatic(),
        category_labels: crate::CategoryLabelSelection::all(),
        format: None,
        numeric_source: None,
        exact_ticks: Some(vec![
            AxisTick::new(-f64::MAX, "same"),
            AxisTick::new(0.0, ""),
            AxisTick::new(f64::MAX, "same"),
        ]),
    };
    spec.validate("x-axis").unwrap();
    spec.limits = Some((-f64::MAX, f64::MAX));
    spec.validate("x-axis").unwrap();
    spec.limits = Some((0.0, f64::MAX));
    assert!(matches!(
        spec.validate("x-axis"),
        Err(crate::Error::Data(message)) if message.contains("outside resolved range")
    ));
    // An automatic domain cannot be checked at configuration time, but still
    // must reject an out-of-domain exact position before text measurement.
    assert!(
        Axis::validate_ticks(spec.exact_ticks.as_ref().unwrap(), (0.0, 1.0), "x-axis").is_err()
    );
}

#[test]
fn exact_axis_sources_bypass_automatic_measurement_in_both_constructors() {
    let mut theme = crate::Theme::publication().resolve().unwrap();
    // Automatic measurement would fail immediately with this role. Exact source
    // selection must not measure any generated label before the axis text pass.
    theme.typography.axes.tick_label.font_size = 0;
    let mut x = AxisSpec::default().limits(0.0, 1.0);
    x.exact_ticks = Some(vec![
        AxisTick::new(0.25, "left exact"),
        AxisTick::new(0.75, "right exact"),
    ]);
    let mut y = AxisSpec::default().limits(0.0, 1.0);
    y.exact_ticks = Some(vec![AxisTick::new(0.5, "")]);
    let panel = crate::layout::PanelBox::new(0.0, 400.0, 0.0, 300.0);
    let numeric = Layout::new_with_axis_specs_in_panel_box(
        (0.0, 1.0),
        (0.0, 1.0),
        panel,
        None,
        None,
        &x,
        &y,
        &theme,
    )
    .unwrap();
    let supplied = Layout::new_with_ticks_and_axis_specs_in_panel_box(
        (0.0, 1.0),
        (0.0, 1.0),
        panel,
        None,
        None,
        vec![AxisTick::new(0.0, "unused")],
        Vec::new(),
        &x,
        &y,
        &theme,
        &crate::layout::fit::ROOT_SCOPE,
    )
    .unwrap();
    for layout in [numeric, supplied] {
        assert_eq!(layout.x_axis.ticks, *x.exact_ticks.as_ref().unwrap());
        assert_eq!(layout.y_axis.ticks, *y.exact_ticks.as_ref().unwrap());
    }
    x.exact_ticks = Some(Vec::new());
    let error = Layout::new_with_axis_specs_in_panel_box(
        (0.0, 1.0),
        (0.0, 1.0),
        panel,
        None,
        None,
        &x,
        &y,
        &theme,
    )
    .unwrap_err();
    assert!(matches!(error, crate::Error::Data(message) if message.contains("must not be empty")));
}

#[test]
fn automatic_tick_fit_never_swallows_fatal_text_errors() {
    let mut theme = crate::Theme::publication().resolve().unwrap();
    theme.typography.axes.tick_label.font_size = 0;
    let projection = Projection::new((0.0, 10.0), (0.0, 1.0), (0.0, 400.0, 0.0, 200.0));
    let error =
        automatic_numeric_ticks((0.0, 10.0), &projection, AxisSide::Bottom, &theme).unwrap_err();
    assert!(error.to_string().contains("font size must be positive"));
}

#[test]
fn later_invalid_axis_source_wins_over_terminal_tick_fit_miss() {
    use crate::layout::fit::{FitScope, PlanError};
    let theme = crate::Theme::publication().resolve().unwrap();
    let scope = FitScope::root()
        .child(crate::layout::AddressRole::FacetCell, 2)
        .unwrap();
    let x = AxisSpec::default().limits(1.1, 1.2);
    let y = AxisSpec {
        exact_ticks: Some(Vec::new()),
        ..AxisSpec::default().limits(0.0, 1.0)
    };
    let result = Layout::new_with_ticks_and_axis_specs_in_panel_box(
        (1.1, 1.2),
        (0.0, 1.0),
        crate::layout::PanelBox::new(0.0, 1.0, 0.0, 100.0),
        None,
        None,
        Vec::new(),
        Vec::new(),
        &x,
        &y,
        &theme,
        &scope,
    );
    assert!(matches!(
        result,
        Err(PlanError::Fatal(crate::Error::Data(_)))
    ));
}

#[test]
fn narrower_x_panel_uses_fewer_automatic_ticks_without_changing_resolved_range() {
    let theme = crate::Theme::publication().resolve().unwrap();
    let wide = Layout::new_with_axis_specs_in_panel_box(
        (0.0, 100.0),
        (0.0, 100.0),
        crate::layout::PanelBox::new(0.0, 400.0, 0.0, 400.0),
        Some("x".into()),
        Some("y".into()),
        &AxisSpec::default(),
        &AxisSpec::default(),
        &theme,
    )
    .unwrap();
    let narrow = Layout::new_with_axis_specs_in_panel_box(
        (0.0, 100.0),
        (0.0, 100.0),
        crate::layout::PanelBox::new(0.0, 40.0, 0.0, 400.0),
        Some("x".into()),
        Some("y".into()),
        &AxisSpec::default(),
        &AxisSpec::default(),
        &theme,
    )
    .unwrap();

    assert_eq!(wide.projection.x_range(), narrow.projection.x_range());
    assert!(narrow.x_axis.ticks.len() < wide.x_axis.ticks.len());
}

#[test]
fn shorter_y_panel_uses_fewer_automatic_ticks_without_changing_resolved_range() {
    let theme = crate::Theme::publication().resolve().unwrap();
    let tall = Layout::new_with_axis_specs_in_panel_box(
        (0.0, 100.0),
        (0.0, 100.0),
        crate::layout::PanelBox::new(0.0, 400.0, 0.0, 400.0),
        Some("x".into()),
        Some("y".into()),
        &AxisSpec::default(),
        &AxisSpec::default(),
        &theme,
    )
    .unwrap();
    let short = Layout::new_with_axis_specs_in_panel_box(
        (0.0, 100.0),
        (0.0, 100.0),
        crate::layout::PanelBox::new(0.0, 400.0, 0.0, 40.0),
        Some("x".into()),
        Some("y".into()),
        &AxisSpec::default(),
        &AxisSpec::default(),
        &theme,
    )
    .unwrap();

    assert_eq!(tall.projection.y_range(), short.projection.y_range());
    assert!(short.y_axis.ticks.len() < tall.y_axis.ticks.len());
}

#[test]
fn wider_numeric_labels_reduce_automatic_x_tick_density() {
    let theme = crate::Theme::publication().resolve().unwrap();
    let panel_box = crate::layout::PanelBox::new(0.0, 400.0, 0.0, 100.0);
    let wider_label_spec = AxisSpec::default().limits(0.0, 1e12);
    let shorter_label_spec = AxisSpec::default().limits(0.0, 1.0);
    let wider_labels = Layout::new_with_axis_specs_in_panel_box(
        (0.0, 100.0),
        (0.0, 1.0),
        panel_box,
        Some("x".into()),
        Some("y".into()),
        &wider_label_spec,
        &shorter_label_spec,
        &theme,
    )
    .unwrap();
    let shorter_labels = Layout::new_with_axis_specs_in_panel_box(
        (0.0, 1.0),
        (0.0, 1.0),
        panel_box,
        Some("x".into()),
        Some("y".into()),
        &shorter_label_spec,
        &shorter_label_spec,
        &theme,
    )
    .unwrap();

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
fn automatic_ticks_keep_the_first_span_adaptive_source_when_labels_fit() {
    let theme = crate::Theme::publication().resolve().unwrap();
    let spec = AxisSpec::default().limits(0.0, 100.0);
    let layout = Layout::new_with_axis_specs_in_panel_box(
        (0.0, 100.0),
        (0.0, 100.0),
        crate::layout::PanelBox::new(0.0, 400.0, 0.0, 400.0),
        Some("x".into()),
        Some("y".into()),
        &spec,
        &spec,
        &theme,
    )
    .unwrap();

    assert_eq!(
        layout
            .x_axis
            .ticks
            .iter()
            .map(|tick| tick.position)
            .collect::<Vec<_>>(),
        [0.0, 20.0, 40.0, 60.0, 80.0, 100.0]
    );
}

#[test]
fn resolved_tick_label_size_controls_automatic_tick_measurement_and_drawing() {
    let spec = AxisSpec::default().limits(0.0, 100.0);
    let panel = crate::layout::PanelBox::new(0.0, 400.0, 0.0, 400.0);
    let default_theme = crate::Theme::publication().resolve().unwrap();
    let default_layout = Layout::new_with_axis_specs_in_panel_box(
        (0.0, 100.0),
        (0.0, 100.0),
        panel,
        None,
        None,
        &spec,
        &spec,
        &default_theme,
    )
    .unwrap();
    let mut large_tick_theme = default_theme.clone();
    large_tick_theme.typography.axes.tick_label.font_size = 96;
    let large_tick_layout = Layout::new_with_axis_specs_in_panel_box(
        (0.0, 100.0),
        (0.0, 100.0),
        panel,
        None,
        None,
        &spec,
        &spec,
        &large_tick_theme,
    )
    .unwrap();
    let empty_tick_layout = Layout::new_with_ticks_and_axis_specs_in_panel_box(
        (0.0, 100.0),
        (0.0, 100.0),
        panel,
        None,
        None,
        Vec::new(),
        Vec::new(),
        &spec,
        &spec,
        &large_tick_theme,
        &crate::layout::fit::ROOT_SCOPE,
    )
    .unwrap();

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
    let mut theme = crate::Theme::science().resolve().unwrap();
    theme.axes.grid = None;
    theme.axes.frame.as_mut().unwrap().stroke.paint = crate::theme::Color::rgb(1, 2, 3);
    theme.axes.ticks.as_mut().unwrap().stroke.paint = crate::theme::Color::rgb(161, 178, 195);
    let layout = Layout::new_with_axis_specs_in_panel_box(
        (0.0, 10.0),
        (0.0, 10.0),
        crate::layout::PanelBox::new(40.0, 360.0, 40.0, 360.0),
        None,
        None,
        &AxisSpec::default(),
        &AxisSpec::default(),
        &theme,
    )
    .unwrap();
    let expected_ticks = layout.x_axis.ticks.len() + layout.y_axis.ticks.len();
    let mut backend = crate::render::svg_backend::SvgBackend::in_memory(400, 400);
    backend.begin(400, 400).unwrap();

    layout.draw_decorations(&theme, &mut backend).unwrap();
    backend.end().unwrap();

    let svg = String::from_utf8(backend.into_bytes().unwrap()).unwrap();
    assert_eq!(svg.matches("stroke=\"#010203\"").count(), 4);
    assert_eq!(svg.matches("stroke=\"#A1B2C3\"").count(), expected_ticks);
}

#[test]
fn frozen_axes_and_grid_replay_without_layout_theme_or_shaping() {
    let theme = crate::Theme::publication().resolve().unwrap();
    let spec = AxisSpec {
        provenance: RangeProvenance::Configured,
        limits: Some((0.0, 10.0)),
        exact_ticks: Some(vec![
            AxisTick::new(0.0, ""),
            AxisTick::new(5.0, "middle"),
            AxisTick::new(10.0, ""),
        ]),
        generation: crate::TickGeneration::automatic(),
        category_labels: crate::CategoryLabelSelection::all(),
        format: None,
        numeric_source: None,
    };
    let layout = Layout::new_with_axis_specs_in_panel_box(
        (0.0, 10.0),
        (0.0, 10.0),
        crate::layout::PanelBox::new(80.0, 360.0, 80.0, 360.0),
        Some("horizontal".into()),
        Some("vertical".into()),
        &spec,
        &spec,
        &theme,
    )
    .unwrap();
    let plan = layout.plan_axes(&theme).unwrap();
    drop(layout);
    drop(theme);
    let shape_calls = crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get);
    let render = |plan: &PanelAxisPlan| {
        let mut backend = crate::render::svg_backend::SvgBackend::in_memory(400, 400);
        backend.begin(400, 400).unwrap();
        plan.draw_grid(&mut backend).unwrap();
        plan.draw(&mut backend).unwrap();
        backend.end().unwrap();
        String::from_utf8(backend.into_bytes().unwrap()).unwrap()
    };
    let svg = render(&plan);
    assert_eq!(svg, render(&plan.clone()));
    crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| assert_eq!(calls.get(), shape_calls));
    assert_eq!(svg.matches(">middle</text>").count(), 2);
    assert!(svg.contains(">horizontal</text>") && svg.contains(">vertical</text>"));
    // Empty endpoint labels retain all six ticks and all six grid lines.
    assert_eq!(svg.matches("<polyline ").count(), 4 + 6 + 6);
}

#[test]
fn absent_frame_and_ticks_leave_axis_text_without_panicking() {
    let mut theme = crate::Theme::publication().resolve().unwrap();
    theme.axes.frame = None;
    theme.axes.ticks = None;
    theme.axes.grid = None;
    let layout = Layout::new_with_axis_specs_in_panel_box(
        (0.0, 10.0),
        (0.0, 10.0),
        crate::layout::PanelBox::new(40.0, 360.0, 40.0, 360.0),
        Some("horizontal".into()),
        Some("vertical".into()),
        &AxisSpec::default(),
        &AxisSpec::default(),
        &theme,
    )
    .unwrap();
    let mut backend = crate::render::svg_backend::SvgBackend::in_memory(400, 400);
    backend.begin(400, 400).unwrap();

    layout.draw_decorations(&theme, &mut backend).unwrap();
    backend.end().unwrap();

    let svg = String::from_utf8(backend.into_bytes().unwrap()).unwrap();
    assert!(svg.contains(">horizontal</text>") && svg.contains(">vertical</text>"));
    assert!(!svg.contains(" stroke="));
}

#[test]
fn automatic_ticks_reject_when_even_the_terminal_midpoint_cannot_fit() {
    let theme = crate::Theme::publication().resolve().unwrap();
    let projection = Projection::new((0.0, 10.0), (0.0, 10.0), (0.0, 0.0, 0.0, 0.0));

    assert!(automatic_numeric_ticks((0.0, 10.0), &projection, AxisSide::Bottom, &theme).is_err());
}

#[test]
fn automatic_ticks_use_the_assigned_span_instead_of_a_five_tick_ceiling() {
    let theme = crate::Theme::publication().resolve().unwrap();
    let spec = AxisSpec::default().limits(5.0, 30.0);
    let layout = Layout::new_with_axis_specs_in_panel_box(
        (5.0, 30.0),
        (0.0, 1.0),
        crate::layout::PanelBox::new(0.0, 1_000.0, 0.0, 100.0),
        Some("x".into()),
        Some("y".into()),
        &spec,
        &AxisSpec::default(),
        &theme,
    )
    .unwrap();

    assert!(layout.x_axis.ticks.len() > 5);
}

#[test]
fn complete_sources_fit_endpoint_labels_inward_instead_of_filtering_positions() {
    let theme = crate::Theme::publication().resolve().unwrap();
    let projection = Projection::new((0.0, 100.0), (0.0, 1.0), (0.0, 160.0, 0.0, 100.0));
    let ticks = vec![
        AxisTick::new(0.0, "0"),
        AxisTick::new(50.0, "50"),
        AxisTick::new(100.0, "100"),
    ];

    let narrow = Projection::new((0.0, 100.0), (0.0, 1.0), (0.0, 100.0, 0.0, 100.0));
    assert!(!super::sources::labels_fit(&ticks, &narrow, AxisSide::Bottom, &theme).unwrap());
    assert!(super::sources::labels_fit(&ticks, &projection, AxisSide::Bottom, &theme).unwrap());
    assert_eq!(
        ticks.iter().map(|tick| tick.position).collect::<Vec<_>>(),
        [0.0, 50.0, 100.0]
    );
}

#[test]
fn categorical_y_ticks_preserve_the_supplied_vector_and_row_range() {
    let theme = crate::Theme::publication().resolve().unwrap();
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
        Some("x".into()),
        Some("y".into()),
        Vec::new(),
        y_ticks,
        &AxisSpec::default(),
        &AxisSpec::default(),
        &theme,
        &crate::layout::fit::ROOT_SCOPE,
    )
    .unwrap();

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
fn overflowing_extrapolation_numerator_keeps_a_finite_clipped_mark() {
    let projection = Projection::new((-1e308, -5e307), (0.0, 1.0), (100.0, 500.0, 100.0, 300.0));
    assert_eq!(projection.project(1e308, 0.5), (1700.0, 200.0));
    let source = crate::Plot::line(
        crate::NumericValues::new("x", [-1e308, 1e308]),
        crate::NumericValues::new("y", [0.25, 0.75]),
    )
    .x_limits(-1e308, -5e307)
    .y_limits(0.0, 1.0)
    .erase();
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("clipped.svg");
    source.svg_preserve_text(&path).unwrap();
    let svg = std::fs::read_to_string(path).unwrap();
    assert!(svg.contains("clip-path="));
    assert!(svg
        .lines()
        .filter(|line| line.starts_with("<polyline "))
        .all(|line| {
            !line.contains("NaN") && !line.contains("\"inf") && !line.contains(",inf")
        }));
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
    assert_eq!(nice_range(0.0, upper, 5).unwrap(), (0.0, upper));
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
fn opposite_extreme_ticks_use_scaled_steps_and_distinct_labels() {
    let ticks = nice_ticks(-f64::MAX, f64::MAX, 5);
    let labels = distinct_tick_labels(&ticks);

    assert_eq!(ticks, [-1e308, 0.0, 1e308]);
    assert_ne!(labels[0], labels[1]);
}

#[test]
fn numeric_layout_preserves_explicit_adjacent_range_and_ticks() {
    let theme = crate::Theme::publication().resolve().unwrap();
    let upper = f64::from_bits(1);
    let spec = AxisSpec::default().limits(0.0, upper);
    let layout = Layout::new_with_axis_specs(
        (0.0, upper),
        (0.0, upper),
        640,
        480,
        Some("x".into()),
        Some("y".into()),
        &spec,
        &spec,
        &theme,
    )
    .unwrap();

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
