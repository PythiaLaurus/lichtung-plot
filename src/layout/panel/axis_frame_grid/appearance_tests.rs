use super::*;
use crate::theme::{AxisSpacing, FrameSides, FrameTheme, RelativeSpacing};

#[test]
fn selected_sides_are_frozen_drawn_and_published_once_with_no_inactive_edge_reserve() {
    let panel = crate::layout::PanelBox::new(80.0, 480.0, 80.0, 480.0);
    for (sides, count, right) in [
        (
            Some(FrameSides::new(true, true, true, true).unwrap()),
            4,
            true,
        ),
        (
            Some(FrameSides::new(false, false, false, true).unwrap()),
            1,
            false,
        ),
        (None, 0, false),
    ] {
        let mut theme = crate::Theme::Publication.resolve().unwrap();
        let stroke = theme.axes.frame().unwrap().stroke().clone();
        theme.axes = theme
            .axes
            .with_frame(sides.map(|sides| FrameTheme::new(sides, stroke).unwrap()))
            .unwrap()
            .with_ticks(None)
            .unwrap()
            .with_grid(None)
            .unwrap();
        let layout = Layout::envelope(panel);
        let plan = layout.plan_axes(&theme).unwrap();
        let expected_right = if right { 410.0_f64 / 1024.0 } else { 0.0 };
        assert_eq!(
            plan.requirements().right.to_bits(),
            expected_right.to_bits()
        );
        assert_eq!(
            plan.frame
                .as_ref()
                .map_or(0, |frame| frame.sides.iter().flatten().count()),
            count
        );
        let mut backend = crate::render::svg_backend::SvgBackend::in_memory(600, 600);
        backend.begin(600, 600).unwrap();
        plan.draw(&mut backend).unwrap();
        backend.end().unwrap();
        let svg = String::from_utf8(backend.into_bytes().unwrap()).unwrap();
        assert_eq!(svg.matches("<polyline ").count(), count);
        if count == 1 {
            assert!(svg.contains("points=\"80,120 80,520\""));
        }
        let geometry = plan.geometry_snapshot().unwrap();
        assert_eq!(geometry.bounds().len(), count);
    }
}

#[test]
fn ordinary_panel_does_not_paint_or_reserve_the_independent_cell_frame() {
    let mut theme = crate::Theme::Publication.resolve().unwrap();
    let cell = theme
        .axes
        .cell_frame()
        .unwrap()
        .clone()
        .with_sides(FrameSides::new(true, true, true, true).unwrap())
        .unwrap();
    theme.axes = theme
        .axes
        .with_frame(None)
        .unwrap()
        .with_cell_frame(Some(cell))
        .unwrap()
        .with_ticks(None)
        .unwrap()
        .with_grid(None)
        .unwrap();
    let layout = Layout::envelope(crate::layout::PanelBox::new(80.0, 480.0, 80.0, 480.0));
    let plan = layout.plan_axes(&theme).unwrap();
    assert!(plan.frame.is_none());
    assert_eq!(
        [
            plan.requirements().left,
            plan.requirements().right,
            plan.requirements().bottom,
            plan.requirements().top
        ]
        .map(f64::to_bits),
        [0; 4]
    );
    let mut backend = crate::render::svg_backend::SvgBackend::in_memory(600, 600);
    backend.begin(600, 600).unwrap();
    plan.draw(&mut backend).unwrap();
    backend.end().unwrap();
    let svg = String::from_utf8(backend.into_bytes().unwrap()).unwrap();
    assert!(!svg.contains("<polyline "));
}

#[test]
fn zero_chrome_keeps_required_text_and_combines_fractional_gaps_before_hard_rounding() {
    let mut theme = crate::Theme::Publication.resolve().unwrap();
    let zero = RelativeSpacing::ratio(0, 1).unwrap();
    // Against the actual selected font metrics, each gap is exactly one third
    // of a CSS quantum. Independent upward rounding would over-reserve a quantum.
    let spacing = AxisSpacing::new(
        RelativeSpacing::ratio(1, 49_152).unwrap(),
        zero.clone(),
        RelativeSpacing::ratio(1, 52_224).unwrap(),
        zero.clone(),
        zero.clone(),
        zero,
    )
    .unwrap();
    theme.axes = theme
        .axes
        .with_frame(None)
        .unwrap()
        .with_ticks(None)
        .unwrap()
        .with_grid(None)
        .unwrap()
        .with_spacing(spacing)
        .unwrap();
    let mut layout = Layout::envelope(crate::layout::PanelBox::new(80.0, 480.0, 80.0, 480.0));
    layout.x_axis.source = TickSource::EXACT;
    layout.x_axis.ticks = vec![AxisTick::new(0.5, "I")];
    layout.x_axis.label = "I".into();
    let plan = layout.plan_axes(&theme).unwrap();
    let label = &plan.x_axis.ticks[0].label.as_ref().unwrap().0;
    let title = &plan.x_axis.title.as_ref().unwrap().0;
    assert!(plan.frame.is_none() && plan.grid.is_none() && plan.x_axis.tick_style.is_none());
    let expected = label.height_quanta + title.height_quanta + 1;
    assert_eq!(
        crate::layout::kernel::LayoutScalar::from_css_pixels(
            plan.requirements().bottom,
            crate::layout::kernel::SpatialRounding::NearestEven
        )
        .unwrap()
        .quanta(),
        expected
    );
    assert_eq!(label.source, "I");
    assert_eq!(title.source, "I");
}
