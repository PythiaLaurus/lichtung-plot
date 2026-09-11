use super::*;

#[test]
fn selected_axis_plan_retains_semantics_beside_projection_and_sources() {
    let theme = crate::Theme::Publication.resolve();
    let layout = layout(
        PanelBox::new(80.0, 680.0, 80.0, 480.0),
        vec![AxisTick::new(1.0, "one")],
        vec![AxisTick::new(1.0, "one")],
        TickSource::EXACT,
    );
    let plot = crate::Plot::scatter(
        crate::NumericValues::new("x", [0.0, 3.0]),
        crate::NumericValues::new("y", [0.0, 3.0]),
    )
    .erase()
    .with_resolved_facet_ranges(crate::spec::PlotKind::Scatter, Some((0.0, 3.0)), None);
    let inputs = plot.axis_semantic_inputs([crate::spec::AxisDomain::Continuous; 2]);
    let plan = layout
        .measure_axes(&theme, &FitScope::root(), inputs)
        .unwrap();
    let retained = plan.clone();
    assert_eq!(retained.semantics, inputs);
    assert_eq!(retained.standard, plan.standard);
    assert_eq!(retained.x_axis.source, TickSource::EXACT);
    assert_eq!(
        retained.range(AxisSide::Bottom),
        layout.projection.x_range()
    );
}
use crate::layout::PanelBox;

fn layout(panel: PanelBox, x: Vec<AxisTick>, y: Vec<AxisTick>, source: TickSource) -> Layout {
    let mut layout = Layout::self_managed(800, 600);
    layout.projection = Projection::new((0.0, 3.0), (0.0, 3.0), panel.as_tuple());
    layout.x_axis.ticks = x;
    layout.y_axis.ticks = y;
    layout.x_axis.source = source;
    layout.y_axis.source = source;
    layout
}

#[test]
fn frozen_axis_retains_projection_source_and_unlabeled_positions() {
    let theme = crate::Theme::Publication.resolve();
    let source = TickSource {
        kind: SourceKind::LogInterval(3),
        ordinal: 2,
        cap: 81,
    };
    let layout = layout(
        PanelBox::new(80.0, 680.0, 80.0, 480.0),
        vec![AxisTick::new(0.0, "1"), AxisTick::new(3.0, "")],
        Vec::new(),
        source,
    );
    let plan = layout.plan_axes(&theme).unwrap();
    drop(layout);
    drop(theme);
    for plan in [&plan, &plan.clone()] {
        assert_eq!(plan.x_axis.source, source);
        assert_eq!(plan.x_axis.ticks[0].position.to_bits(), 0.0_f64.to_bits());
        assert_eq!(plan.x_axis.ticks[1].position.to_bits(), 3.0_f64.to_bits());
        assert!(plan.x_axis.ticks[1].label.is_none());
        assert_eq!(plan.projection.x_range(), (0.0, 3.0));
        assert_eq!(plan.projection.y_range(), (0.0, 3.0));
        assert_eq!(plan.projection.project(0.0, 0.0), (80.0, 80.0));
        assert_eq!(plan.projection.project(3.0, 3.0), (680.0, 480.0));
    }
}

#[test]
fn exact_cells_use_adjacent_label_midpoints_and_skip_empty_labels() {
    let theme = crate::Theme::Publication.resolve();
    let mut layout = layout(
        PanelBox::new(80.0, 680.0, 80.0, 480.0),
        vec![
            AxisTick::new(0.0, "wide"),
            AxisTick::new(0.01, "near"),
            AxisTick::new(3.0, "last"),
        ],
        Vec::new(),
        TickSource::EXACT,
    );
    assert!(matches!(
        layout.measure_axes(&theme, &FitScope::root(), layout.test_semantics()),
        Err(PlanError::Fit(_))
    ));
    layout.x_axis.ticks[1].label.clear();
    let plan = layout.plan_axes(&theme).unwrap();
    assert_eq!(plan.x_axis.ticks.len(), 3);
    assert!(plan.x_axis.ticks[1].label.is_none());
    let first = &plan.x_axis.ticks[0];
    let last = &plan.x_axis.ticks[2];
    assert_eq!(first.cell.end.to_bits(), 378.0_f64.to_bits());
    assert_eq!(last.cell.start.to_bits(), 382.0_f64.to_bits());
    assert!(first.label.as_ref().unwrap().1.position.x > first.segment[0].x);
    assert!(last.label.as_ref().unwrap().1.position.x < last.segment[0].x);
    let grid = plan.grid.as_ref().unwrap();
    for (tick, line) in plan.x_axis.ticks.iter().zip(&grid.lines) {
        assert_eq!(tick.segment[0].x.to_bits(), line[0].x.to_bits());
    }
}

#[test]
fn category_bands_round_exact_ratios_and_preserve_reverse_semantic_order() {
    let theme = crate::Theme::Publication.resolve();
    let ticks = vec![
        AxisTick::new(2.5, "third"),
        AxisTick::new(1.5, "second"),
        AxisTick::new(0.5, "first"),
    ];
    let layout = layout(
        PanelBox::new(200.0, 700.0, 80.0, 181.0),
        Vec::new(),
        ticks,
        TickSource::SUPPLIED,
    );
    let plan = layout.plan_axes(&theme).unwrap();
    for (tick, (label, center_quanta)) in
        plan.y_axis
            .ticks
            .iter()
            .zip([("third", 86187_i64), ("second", 51712), ("first", 17237)])
    {
        assert_eq!(
            tick.cell.center.to_bits(),
            (80.0 + center_quanta as f64 / 1024.0).to_bits()
        );
        let (text, placement) = tick.label.as_ref().unwrap();
        assert_eq!(text.source, label);
        assert_eq!(placement.rotation.to_bits(), 0.0_f64.to_bits());
        assert!(text.height() <= tick.cell.capacity());
    }
}

#[test]
fn unbreakable_category_fails_intact_and_cannot_rotate_or_drop() {
    let theme = crate::Theme::Publication.resolve();
    let label = "W".repeat(50);
    let layout = layout(
        PanelBox::new(80.0, 280.0, 80.0, 280.0),
        vec![AxisTick::new(1.5, &label)],
        Vec::new(),
        TickSource::SUPPLIED,
    );
    assert!(matches!(
        layout.measure_axes(&theme, &FitScope::root(), layout.test_semantics()),
        Err(PlanError::Fit(_))
    ));
    assert_eq!(layout.x_axis.ticks.len(), 1);
    assert_eq!(layout.x_axis.ticks[0].label, label);
}

#[test]
fn vertical_category_cells_reject_insufficient_height_without_dropping() {
    let theme = crate::Theme::Publication.resolve();
    let layout = layout(
        PanelBox::new(80.0, 680.0, 80.0, 100.0),
        Vec::new(),
        vec![
            AxisTick::new(0.5, "first"),
            AxisTick::new(1.5, "second"),
            AxisTick::new(2.5, "third"),
        ],
        TickSource::SUPPLIED,
    );
    assert!(matches!(
        layout.measure_axes(&theme, &FitScope::root(), layout.test_semantics()),
        Err(PlanError::Fit(_))
    ));
    assert_eq!(layout.y_axis.ticks.len(), 3);
}

#[test]
fn parent_width_keeps_natural_category_text_or_wraps_only_at_legal_boundaries() {
    let theme = crate::Theme::Publication.resolve();
    let mut layout = layout(
        PanelBox::new(200.0, 700.0, 80.0, 480.0),
        Vec::new(),
        vec![AxisTick::new(1.5, "Control group")],
        TickSource::SUPPLIED,
    );
    layout
        .y_axis
        .assign_label_width(500.0, 400.0, &theme)
        .unwrap();
    let wide = layout.plan_axes(&theme).unwrap();
    assert_eq!(
        wide.y_axis.ticks[0].label.as_ref().unwrap().0.lines.len(),
        1
    );
    let natural = layout.y_axis.assigned_tick_width.unwrap();
    layout
        .y_axis
        .assign_label_width(natural - 1.0, 400.0, &theme)
        .unwrap();
    let compact = layout.plan_axes(&theme).unwrap();
    let text = &compact.y_axis.ticks[0].label.as_ref().unwrap().0;
    assert_eq!(text.source, "Control group");
    assert_eq!(text.line_texts(), ["Control ", "group"]);
    assert_eq!(
        compact.y_axis.ticks[0]
            .label
            .as_ref()
            .unwrap()
            .1
            .rotation
            .to_bits(),
        0.0_f64.to_bits()
    );
}

#[test]
fn gutter_is_provisional_below_at_and_above_forty_percent() {
    let theme = crate::Theme::Publication.resolve();
    let mut layout = layout(
        PanelBox::new(80.0, 680.0, 80.0, 480.0),
        Vec::new(),
        vec![AxisTick::new(1.5, "W".repeat(20))],
        TickSource::SUPPLIED,
    );
    layout.y_axis.label = "vertical title".into();
    let gutter = layout.plan_axes(&theme).unwrap().requirements().left;
    for fraction in [0.39, 0.40, 0.41, 1.1] {
        layout.projection = Projection::new(
            (0.0, 3.0),
            (0.0, 3.0),
            (80.0, 80.0 + gutter / fraction, 80.0, 480.0),
        );
        let plan = layout.plan_axes(&theme).unwrap();
        assert_eq!(plan.requirements().left.to_bits(), gutter.to_bits());
        assert_eq!(
            plan.y_axis.ticks[0].label.as_ref().unwrap().0.source,
            "W".repeat(20)
        );
    }
}

#[test]
fn frozen_requirements_contain_drawn_ink_and_tick_strokes_with_or_without_frame() {
    for frame_present in [false, true] {
        let mut theme = crate::Theme::Science.resolve();
        if !frame_present {
            theme.axes.frame = None;
        }
        theme.axes.ticks.as_mut().unwrap().length = 17;
        let panel = PanelBox::new(250.0, 650.0, 150.0, 550.0);
        let mut layout = layout(
            panel,
            vec![AxisTick::new(0.0, "left"), AxisTick::new(3.0, "right")],
            vec![AxisTick::new(0.0, "low"), AxisTick::new(3.0, "high")],
            TickSource::EXACT,
        );
        layout.x_axis.label = "first line\nsecond line".into();
        layout.y_axis.label = "first line\nsecond line".into();
        let plan = layout.plan_axes(&theme).unwrap();
        let required = plan.requirements();
        for (axis, side) in [
            (&plan.x_axis, AxisSide::Bottom),
            (&plan.y_axis, AxisSide::Left),
        ] {
            let half_stroke = f64::from(axis.tick_style.as_ref().unwrap().width) / 2.0;
            for tick in &axis.ticks {
                let (label, placement) = tick.label.as_ref().unwrap();
                let ink = crate::render::geometry::planned_text_ink_bounds(
                    label,
                    placement.position,
                    placement.anchor,
                    placement.rotation,
                    placement.origin,
                    placement.reverse_lines,
                )
                .unwrap();
                let (actual, required) = match side {
                    AxisSide::Bottom => (
                        tick.segment[1].y - half_stroke - ink.top,
                        theme.axes.spacing.x_tick_label_gap,
                    ),
                    AxisSide::Left => (
                        tick.segment[1].x - half_stroke - ink.right,
                        theme.axes.spacing.y_tick_label_gap,
                    ),
                };
                assert!(actual >= required - 1.0 / 1024.0);
            }
        }
        let mut backend = crate::render::geometry::GeometryBackend::new(&theme).unwrap();
        backend.begin(800, 600).unwrap();
        plan.draw(&mut backend).unwrap();
        backend.end().unwrap();
        for ink in backend.bounds() {
            assert!(ink.left >= panel.x0 - required.left - 1.0 / 1024.0);
            assert!(ink.right <= panel.x1 + required.right + 1.0 / 1024.0);
            assert!(ink.bottom >= panel.y0 - required.bottom - 1.0 / 1024.0);
            assert!(ink.top <= panel.y1 + required.top + 1.0 / 1024.0);
        }
        assert_eq!(
            plan.y_axis.title.as_ref().unwrap().1.rotation.to_bits(),
            (-90.0_f64).to_bits()
        );
    }
}

#[test]
fn measured_gutters_are_translation_invariant_at_fractional_parent_origins() {
    let theme = crate::Theme::Science.resolve();
    let mut layout = layout(
        PanelBox::new(80.0, 480.0, 80.0, 480.0),
        Vec::new(),
        vec![AxisTick::new(0.0, "-0.5"), AxisTick::new(3.0, "0.5")],
        TickSource::EXACT,
    );
    layout.y_axis.label = "Y".into();
    let required = layout.plan_axes(&theme).unwrap().requirements();
    for origin in [80.1, 80.2, 300.333, 500.789] {
        layout.projection = Projection::new(
            (0.0, 3.0),
            (0.0, 3.0),
            (origin, origin + 400.0, 80.0, 480.0),
        );
        assert_eq!(layout.plan_axes(&theme).unwrap().requirements(), required);
    }
}
