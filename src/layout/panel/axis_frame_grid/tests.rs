use super::*;
use crate::TickGeneration;

#[test]
fn empty_axis_measurement_does_not_shape_text() {
    {
        let theme = crate::Theme::publication().resolve().unwrap();

        let layout = Layout::envelope(PanelBox::new(80.0, 770.0, 60.0, 570.0));
        let standard = super::super::StandardPanelInputs::new(&theme).unwrap();

        crate::font::canonical::FONT_SHAPE_CALLS.with(|count| count.set(0));
        let result = layout.x_axis.measure(
            &layout.projection,
            AxisSide::Bottom,
            &theme,
            &standard,
            &FitScope::root(),
        );
        result.unwrap();

        crate::font::canonical::FONT_SHAPE_CALLS.with(|count| assert_eq!(count.get(), 0));
    }
}

#[test]
fn present_empty_axis_title_keeps_line_metrics_while_empty_exact_ticks_stay_hidden() {
    let theme = crate::Theme::publication().resolve().unwrap();
    let mut layout = layout(
        PanelBox::new(80.0, 680.0, 80.0, 480.0),
        vec![AxisTick::new(1.0, "")],
        Vec::new(),
        TickSource::EXACT,
    );
    let absent = layout.plan_axes(&theme).unwrap();
    assert!(absent.x_axis.title.is_none());
    layout.x_axis.label = Some("".into());
    let present = layout.plan_axes(&theme).unwrap();
    let text = &present.x_axis.title.as_ref().unwrap().0;
    assert_eq!(text.line_texts(), [""]);
    assert!(text.height_quanta > 0);
    assert!(present.x_axis.gutter >= absent.x_axis.gutter + text.height());
    assert!(present.x_axis.ticks[0].label.is_none());
    assert_eq!(
        present.x_axis.ticks[0].position.to_bits(),
        absent.x_axis.ticks[0].position.to_bits()
    );
    assert_eq!(
        present.x_axis.ticks[0].segment,
        absent.x_axis.ticks[0].segment
    );
}

#[test]
fn empty_category_alias_keeps_its_line_until_the_owner_hides_tick_text() {
    let theme = crate::Theme::publication().resolve().unwrap();
    let mut layout = layout(
        PanelBox::new(80.0, 680.0, 80.0, 480.0),
        Vec::new(),
        vec![AxisTick::new(1.5, "")],
        TickSource::SUPPLIED,
    );
    let visible = layout.plan_axes(&theme).unwrap();
    let text = &visible.y_axis.ticks[0].label.as_ref().unwrap().0;
    assert_eq!(text.line_texts(), [""]);
    assert!(text.height_quanta > 0);
    layout.y_axis.tick_labels_visible = false;
    let hidden = layout.plan_axes(&theme).unwrap();
    assert!(hidden.y_axis.ticks[0].label.is_none());
    assert!(visible.y_axis.gutter > hidden.y_axis.gutter);
    assert_eq!(
        visible.y_axis.ticks[0].segment,
        hidden.y_axis.ticks[0].segment
    );
    assert_eq!(
        visible.y_axis.ticks[0].position.to_bits(),
        hidden.y_axis.ticks[0].position.to_bits()
    );
}

#[test]
fn selected_axis_plan_retains_semantics_beside_projection_and_sources() {
    let theme = crate::Theme::publication().resolve().unwrap();
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
    .x_tick_generation(TickGeneration::target(5).unwrap())
    .y_tick_generation(TickGeneration::maximum(5).unwrap())
    .erase()
    .with_resolved_facet_ranges(crate::spec::PlotKind::Scatter, Some((0.0, 3.0)), None);
    let inputs = plot.axis_semantic_inputs([crate::spec::AxisDomain::Continuous; 2]);
    let plan = layout
        .measure_axes(&theme, &FitScope::root(), inputs.clone())
        .unwrap();
    let retained = plan.clone();
    assert_eq!(retained.semantics, inputs);
    assert_eq!(retained.semantics[0].generation.target_count(), Some(5));
    assert_eq!(
        retained.semantics[1].generation.get_maximum_count(),
        Some(5)
    );
    assert_eq!(retained.standard, plan.standard);
    assert_eq!(retained.x_axis.source, TickSource::EXACT);
    assert_eq!(
        retained.range(AxisSide::Bottom),
        layout.projection.x_range()
    );
}
use crate::layout::PanelBox;

fn layout(panel: PanelBox, x: Vec<AxisTick>, y: Vec<AxisTick>, source: TickSource) -> Layout {
    let mut layout = Layout::envelope(PanelBox::new(80.0, 770.0, 60.0, 570.0));
    layout.projection = Projection::new((0.0, 3.0), (0.0, 3.0), panel.as_tuple()).with_categories(
        [x.len(), y.len()]
            .map(|count| (source == TickSource::SUPPLIED && count > 0).then_some(count)),
    );
    layout.x_axis.ticks = x;
    layout.y_axis.ticks = y;
    layout.x_axis.source = source;
    layout.y_axis.source = source;
    layout
}

#[test]
fn categorical_marks_ticks_and_grid_use_exact_band_ratios_at_odd_origins() {
    let theme = crate::Theme::publication().resolve().unwrap();
    for origin in [80.0, 80.0 + 1.0 / 1024.0, 80.1, 80.2] {
        let end = origin + 300.0 + 1.0 / 1024.0;
        let ticks = || {
            [0.5, 1.5, 2.5]
                .map(|value| AxisTick::new(value, ""))
                .to_vec()
        };
        let layout = Layout::new_with_ticks_and_axis_specs_in_panel_box(
            (0.0, 3.0),
            (0.0, 3.0),
            PanelBox::new(origin, end, origin, end),
            None,
            None,
            ticks(),
            ticks(),
            &AxisSpec::default(),
            &AxisSpec::default(),
            &theme,
            &FitScope::root(),
        )
        .unwrap();
        assert_eq!(layout.projection.category_counts(), [Some(3); 2]);
        let plan = layout.plan_axes(&theme).unwrap();
        let grid = plan.grid.as_ref().unwrap();
        for (index, value) in [0.5, 1.5, 2.5].into_iter().enumerate() {
            let (x, y) = layout.projection.project(value, value);
            assert_eq!(x.to_bits(), plan.x_axis.ticks[index].segment[0].x.to_bits());
            assert_eq!(y.to_bits(), plan.y_axis.ticks[index].segment[0].y.to_bits());
            assert_eq!(x.to_bits(), grid.lines[index][0].x.to_bits());
            assert_eq!(y.to_bits(), grid.lines[index + 3][0].y.to_bits());
            assert!(layout.projection.project(value + 0.1, value - 0.1).0 > x);
        }
        if origin.to_bits() == 80.0_f64.to_bits() {
            assert_eq!(
                layout.projection.project(1.5, 1.5),
                (235_520.0 / 1024.0, 235_520.0 / 1024.0)
            );
        }
    }
}

#[test]
fn native_categorical_trajectory_installs_band_projection_before_mark_capture() {
    let source = crate::Plot::trajectory(
        crate::CategoryValues::new("condition", ["a", "b", "c"]).unwrap(),
        crate::NumericValues::new("y", [1.0, 2.0, 3.0]),
        crate::CategoryValues::new("group", ["g", "g", "g"]).unwrap(),
    )
    .hue(crate::CategoryValues::new("cohort", ["seen"; 3]).unwrap())
    .erase()
    .with_global_hue_levels(vec!["unobserved".into(), "seen".into()]);
    let theme = crate::Theme::publication().resolve().unwrap();
    let plot = crate::plots::shared_x::prepare_panel_for_test(&source, &theme).unwrap();
    let selected = plot
        .measure_selected_interior(crate::spec::PlotRenderContext::new(
            plot.data(),
            PanelBox::new(80.0, 380.0 + 1.0 / 1024.0, 80.0, 380.0),
            &theme,
        ))
        .unwrap();
    assert_eq!(
        selected.layout.projection.category_counts(),
        [Some(3), None]
    );
    let axes = selected.axes.as_ref().unwrap();
    let x = axes.x_axis.ticks[1].segment[0].x;
    assert_eq!(x.to_bits(), 230.0_f64.to_bits());
    let guide = selected.discrete_guide.as_ref().unwrap();
    assert_eq!(guide.counts.entries, 1);
    let separate = crate::decorations::legend::discrete::PreparedGuide::prepare_for_test(
        &plot,
        &theme,
        crate::LegendComposition::Separate,
    )
    .unwrap()
    .unwrap();
    assert_eq!(
        (
            separate.counts.entries,
            separate.counts.points,
            separate.counts.lines
        ),
        (1, 1, 0)
    );
    assert!(matches!(
        separate.sections[0].identity,
        crate::decorations::legend::discrete::GuideSectionIdentity::HueColor(_)
    ));
    assert_eq!(
        guide.sections[0].entries()[0].identity,
        crate::decorations::legend::GuideEntryIdentity::category("cohort", "seen").unwrap()
    );
    let crate::layout::interior::MarkPayload::Line(payload) =
        &selected.marks.as_ref().unwrap().payload
    else {
        panic!("selected Trajectory marks")
    };
    for mark in &payload.marks {
        if let crate::plots::line::LineMark::Points(_, style) = mark {
            assert_eq!(
                (style.color.red(), style.color.green(), style.color.blue()),
                (213, 94, 0)
            );
            assert_eq!(
                guide.sections[0].entries()[0].key,
                crate::decorations::legend::ResolvedGuideKey::point(style.clone())
            );
        }
    }
    let mut svg = crate::render::svg_backend::SvgBackend::in_memory(500, 500);
    svg.begin(500, 500).unwrap();
    selected.marks.as_ref().unwrap().draw(&mut svg).unwrap();
    svg.end().unwrap();
    let svg = String::from_utf8(svg.into_bytes().unwrap()).unwrap();
    assert!(
        svg.lines()
            .any(|line| line.contains("<circle ") && line.contains("cx=\"230\"")),
        "{svg}"
    );
}

#[test]
fn assigned_category_tracks_drive_frozen_ticks_grid_and_label_cells() {
    let theme = crate::Theme::publication().resolve().unwrap();
    let mut layout = layout(
        PanelBox::new(80.0, 380.0, 80.0, 380.0),
        vec![AxisTick::new(0.25, "first"), AxisTick::new(1.75, "second")],
        vec![AxisTick::new(1.75, "top"), AxisTick::new(0.25, "bottom")],
        TickSource::SUPPLIED,
    );
    layout.x_axis.assigned_cells = Some(vec![(80.0, 160.0), (160.0, 380.0)]);
    layout.y_axis.assigned_cells = Some(vec![(160.0, 380.0), (80.0, 160.0)]);
    let plan = layout.plan_axes(&theme).unwrap();
    let grid = plan.grid.as_ref().unwrap();
    for (index, center) in [120.0_f64, 270.0].into_iter().enumerate() {
        assert_eq!(
            plan.x_axis.ticks[index].segment[0].x.to_bits(),
            center.to_bits()
        );
        assert_eq!(grid.lines[index][0].x.to_bits(), center.to_bits());
        assert_eq!(
            plan.x_axis.ticks[index]
                .label
                .as_ref()
                .unwrap()
                .1
                .position
                .x
                .to_bits(),
            center.to_bits()
        );
    }
    for (index, center) in [270.0_f64, 120.0].into_iter().enumerate() {
        assert_eq!(
            plan.y_axis.ticks[index].segment[0].y.to_bits(),
            center.to_bits()
        );
        assert_eq!(grid.lines[index + 2][0].y.to_bits(), center.to_bits());
        assert_eq!(
            plan.y_axis.ticks[index]
                .label
                .as_ref()
                .unwrap()
                .1
                .position
                .y
                .to_bits(),
            center.to_bits()
        );
    }
}

#[test]
fn frozen_axis_retains_projection_source_and_unlabeled_positions() {
    let theme = crate::Theme::publication().resolve().unwrap();
    let source = TickSource {
        kind: SourceKind::LogInterval(3),
        ordinal: 2,
        cap: 81,
        precision: None,
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
    let theme = crate::Theme::publication().resolve().unwrap();
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
    let theme = crate::Theme::publication().resolve().unwrap();
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
    let theme = crate::Theme::publication().resolve().unwrap();
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
fn visibility_retains_complete_axis_geometry_and_never_consumes_an_unselected_glyph() {
    let mut theme = crate::Theme::publication().resolve().unwrap();
    let layout = layout(
        PanelBox::new(80.0, 680.0, 80.0, 280.0),
        vec![
            AxisTick::new(0.5, "First"),
            AxisTick::new(1.5, "\u{10ffff}"),
            AxisTick::new(2.5, "Last"),
        ],
        Vec::new(),
        TickSource::SUPPLIED,
    );
    assert!(layout.plan_axes(&theme).is_err());
    theme.text_layouts.ticks[0] = Some(
        crate::TextLayout::default()
            .visibility(crate::TextVisibility::every(2, 0, crate::LabelEndpoints::Both).unwrap()),
    );
    let plan = layout.plan_axes(&theme).unwrap();
    assert_eq!(plan.x_axis.eligible_labels, [0, 1, 2]);
    assert_eq!(
        plan.x_axis
            .ticks
            .iter()
            .enumerate()
            .filter_map(|(index, tick)| tick.label.as_ref().map(|_| index))
            .collect::<Vec<_>>(),
        [0, 2]
    );
    let mut hidden = layout.clone();
    hidden.x_axis.tick_labels_visible = false;
    let geometry = hidden.plan_axes(&theme).unwrap();
    assert_eq!(plan.x_axis.ticks.len(), 3);
    assert_eq!(
        plan.grid.as_ref().unwrap().lines,
        geometry.grid.as_ref().unwrap().lines
    );
    for (selected, unlabelled) in plan.x_axis.ticks.iter().zip(&geometry.x_axis.ticks) {
        assert_eq!(selected.position.to_bits(), unlabelled.position.to_bits());
        assert_eq!(selected.segment, unlabelled.segment);
    }
}

#[test]
fn ordered_tick_angles_fit_complete_exact_labels_without_changing_ticks_or_grids() {
    use crate::{TextLayout, TextOrientation, TextRotation};
    let mut theme = crate::Theme::publication().resolve().unwrap();
    let layout = layout(
        PanelBox::new(80.0, 180.0, 80.0, 380.0),
        vec![
            AxisTick::new(0.5, "LightGBM"),
            AxisTick::new(1.5, "LightGBM"),
            AxisTick::new(2.5, "LightGBM"),
        ],
        Vec::new(),
        TickSource::EXACT,
    );
    assert!(layout.plan_axes(&theme).is_err());
    let policy = TextLayout::default().orientation(TextOrientation::automatic_ticks(vec![
        TextRotation::horizontal(),
        TextRotation::vertical_up(),
    ]));
    theme.text_layouts.ticks[0] = Some(policy.clone());
    let selected = layout.plan_axes(&theme).unwrap();
    for tick in &selected.x_axis.ticks {
        let (plan, placement) = tick.label.as_ref().unwrap();
        assert_eq!(plan.source, "LightGBM");
        assert_eq!(plan.request.as_ref(), Some(&policy));
        assert_eq!(placement.rotation.to_bits(), (-90.0_f64).to_bits());
        let bounds = crate::layout::text::placement::footprint(*placement, plan).unwrap();
        assert!(bounds.x0 >= tick.cell.start && bounds.x1 <= tick.cell.end && bounds.y1 < 80.0);
    }
    theme.text_layouts.ticks[0] = Some(TextLayout::default().rotation(TextRotation::vertical_up()));
    let fixed = layout.plan_axes(&theme).unwrap();
    assert_eq!(
        selected.grid.as_ref().unwrap().lines,
        fixed.grid.as_ref().unwrap().lines
    );
    assert_eq!(selected.x_axis.ticks.len(), 3);
    let mut titled = layout.clone();
    titled.y_axis.label = Some(
        crate::Text::new("Horizontal\nY title")
            .layout(TextLayout::default().rotation(TextRotation::horizontal())),
    );
    let plan = titled.plan_axes(&theme).unwrap();
    let (_, placement) = plan.y_axis.title.as_ref().unwrap();
    assert_eq!(placement.rotation.to_bits(), 0.0_f64.to_bits());
    assert!(!placement.reverse_lines);
}

#[test]
fn vertical_category_cells_reject_insufficient_height_without_dropping() {
    let theme = crate::Theme::publication().resolve().unwrap();
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
    let theme = crate::Theme::publication().resolve().unwrap();
    let mut layout = layout(
        PanelBox::new(200.0, 700.0, 80.0, 480.0),
        Vec::new(),
        vec![AxisTick::new(1.5, "Control group")],
        TickSource::SUPPLIED,
    );
    layout.y_axis.assigned_tick_width = Some(500.0);
    let wide = layout.plan_axes(&theme).unwrap();
    assert_eq!(
        wide.y_axis.ticks[0].label.as_ref().unwrap().0.lines.len(),
        1
    );
    let plan = &wide.y_axis.ticks[0].label.as_ref().unwrap().0;
    let natural = plan.width().max(plan.fit_width());
    layout.y_axis.assigned_tick_width = Some(natural - 1.0);
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
    let theme = crate::Theme::publication().resolve().unwrap();
    let mut layout = layout(
        PanelBox::new(80.0, 680.0, 80.0, 480.0),
        Vec::new(),
        vec![AxisTick::new(1.5, "W".repeat(20))],
        TickSource::SUPPLIED,
    );
    layout.y_axis.label = Some("vertical title".into());
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
        let mut theme = crate::Theme::science().resolve().unwrap();
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
        layout.x_axis.label = Some("first line\nsecond line".into());
        layout.y_axis.label = Some("first line\nsecond line".into());
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
                    AxisSide::Bottom => (tick.segment[1].y - half_stroke - ink.top, 8.0),
                    AxisSide::Left => (tick.segment[1].x - half_stroke - ink.right, 6.0),
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
    let theme = crate::Theme::science().resolve().unwrap();
    let mut layout = layout(
        PanelBox::new(80.0, 480.0, 80.0, 480.0),
        Vec::new(),
        vec![AxisTick::new(0.0, "-0.5"), AxisTick::new(3.0, "0.5")],
        TickSource::EXACT,
    );
    layout.y_axis.label = Some("Y".into());
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

#[test]
fn numeric_marks_ticks_and_grid_share_odd_quantum_spans_and_fractional_origins() {
    let theme = crate::Theme::publication().resolve().unwrap();
    for origin in [0.0, 1.0 / 1024.0, 0.1, -0.1] {
        let end = origin + 81_923.0 / 1024.0;
        let mut layout = layout(
            PanelBox::new(origin, end, origin, end),
            [1.0, 2.0, 3.0].map(|value| AxisTick::new(value, "")).into(),
            [1.0, 2.0, 3.0].map(|value| AxisTick::new(value, "")).into(),
            TickSource::EXACT,
        );
        layout.projection = Projection::new((1.0, 3.0), (1.0, 3.0), (origin, end, origin, end));
        let plan = layout.plan_axes(&theme).unwrap();
        let origin = crate::layout::kernel::nearest_spatial_quanta(origin).unwrap();
        for (index, offset) in [0, 40_962, 81_923].into_iter().enumerate() {
            let expected = (origin + offset) as f64 / 1024.0;
            let value = index as f64 + 1.0;
            let mark = layout.projection.project(value, value);
            assert_eq!(mark, (expected, expected));
            assert_eq!(
                plan.x_axis.ticks[index].segment[0].x.to_bits(),
                expected.to_bits()
            );
            assert_eq!(
                plan.y_axis.ticks[index].segment[0].y.to_bits(),
                expected.to_bits()
            );
            let grid = plan.grid.as_ref().unwrap();
            assert_eq!(grid.lines[index][0].x.to_bits(), expected.to_bits());
            assert_eq!(grid.lines[index + 3][0].y.to_bits(), expected.to_bits());
        }
    }
}

#[test]
fn canonical_label_fit_accepts_touching_ink_and_rejects_one_quantum_beyond() {
    let theme = crate::Theme::publication().resolve().unwrap();
    for side in [AxisSide::Bottom, AxisSide::Left] {
        let text = theme
            .measure_text(
                "1",
                &theme.typography.axes.tick_label,
                PhysicalTextAlignment::Center,
                "boundary label",
                false,
            )
            .unwrap();
        let generous = cells::LabelCell {
            center: 0.0,
            start: -100.0,
            end: 100.0,
            inward: false,
        };
        let placement = generous.placement(&text, side, 0.1).unwrap().unwrap();
        let ink = crate::render::geometry::planned_text_ink_bounds(
            &text,
            placement.position,
            placement.anchor,
            placement.rotation,
            placement.origin,
            placement.reverse_lines,
        )
        .unwrap();
        // A taller/wider ink edge exercises the actual ink predicate independently
        // of logical width and line height.
        let mut text = text;
        let line_ink = text.edit_for_test().lines[0].ink_quanta.as_mut().unwrap();
        match side {
            AxisSide::Bottom => line_ink.right += 102_400,
            AxisSide::Left => line_ink.top += 102_400,
        }
        let edge = match side {
            AxisSide::Bottom => ink.right,
            AxisSide::Left => ink.top,
        } + 100.0;
        let cell = cells::LabelCell {
            end: edge,
            ..generous
        };
        assert!(cell.placement(&text, side, 0.1).unwrap().is_some());
        assert!(cells::LabelCell {
            end: edge - 1.0 / 1024.0,
            ..cell
        }
        .placement(&text, side, 0.1)
        .unwrap()
        .is_none());
        let inward = cells::LabelCell {
            inward: true,
            ..cell
        };
        assert!(cells::LabelCell {
            end: edge - 1.0 / 1024.0,
            ..inward
        }
        .placement(&text, side, 0.1)
        .unwrap()
        .is_some());
    }
}

#[test]
fn automatic_selection_and_frozen_axis_agree_at_quantum_fit_boundary() {
    use super::super::sources::{labels_fit, numeric_tick_plan};
    let theme = crate::Theme::publication().resolve().unwrap();
    let ticks = [
        AxisTick::new(0.0, "0"),
        AxisTick::new(1.0, "1"),
        AxisTick::new(2.0, "2"),
    ];
    for side in [AxisSide::Bottom, AxisSide::Left] {
        let projection = |span: i64| {
            let origin = 0.1;
            let end = origin + span as f64 / 1024.0;
            match side {
                AxisSide::Bottom => {
                    Projection::new((0.0, 2.0), (0.0, 2.0), (origin, end, 80.0, 480.0))
                }
                AxisSide::Left => {
                    Projection::new((0.0, 2.0), (0.0, 2.0), (80.0, 480.0, origin, end))
                }
            }
        };
        let mut low = 1;
        let mut high = 200 * 1024;
        while low < high {
            let middle = i64::midpoint(low, high);
            if labels_fit(&ticks, &projection(middle), side, &theme).unwrap() {
                high = middle;
            } else {
                low = middle + 1;
            }
        }
        for span in [low - 1, low, low + 1] {
            let projection = projection(span);
            let selected = numeric_tick_plan(
                (0.0, 2.0),
                TickGeneration::additive(1.0).unwrap(),
                false,
                &projection,
                side,
                &theme,
                None,
                &FitScope::root(),
            )
            .unwrap();
            assert_eq!(selected.source.ordinal == 0, span >= low);
            let mut layout = Layout::envelope(PanelBox::new(80.0, 770.0, 60.0, 570.0));
            layout.projection = projection;
            let axis = match side {
                AxisSide::Bottom => &mut layout.x_axis,
                AxisSide::Left => &mut layout.y_axis,
            };
            axis.source = selected.source;
            axis.ticks = selected.ticks;
            let plan = layout.plan_axes(&theme).unwrap();
            let axis = match side {
                AxisSide::Bottom => &plan.x_axis,
                AxisSide::Left => &plan.y_axis,
            };
            for tick in &axis.ticks {
                let (text, placement) = tick.label.as_ref().unwrap();
                let candidate = theme
                    .measure_text(
                        &text.source,
                        &theme.typography.axes.tick_label,
                        match side {
                            AxisSide::Bottom => PhysicalTextAlignment::Center,
                            AxisSide::Left => PhysicalTextAlignment::End,
                        },
                        "automatic tick label",
                        false,
                    )
                    .unwrap();
                assert_eq!(text.lines, candidate.lines);
                let perpendicular = match side {
                    AxisSide::Bottom => placement.position.y,
                    AxisSide::Left => placement.position.x,
                };
                assert_eq!(
                    tick.cell
                        .placement(&candidate, side, perpendicular)
                        .unwrap()
                        .unwrap(),
                    *placement
                );
                let ink = crate::render::geometry::planned_text_ink_bounds(
                    text,
                    placement.position,
                    placement.anchor,
                    placement.rotation,
                    placement.origin,
                    placement.reverse_lines,
                )
                .unwrap();
                let (start, end) = match side {
                    AxisSide::Bottom => (ink.left, ink.right),
                    AxisSide::Left => (ink.bottom, ink.top),
                };
                assert!(start >= tick.cell.start && end <= tick.cell.end);
            }
        }
    }
}
