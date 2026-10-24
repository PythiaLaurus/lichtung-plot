use super::*;
use crate::theme::{AxisSpacing, FrameSides, FrameTheme, RelativeSpacing};

#[test]
fn authored_axis_titles_keep_centered_blocks_and_use_independent_selected_metrics() {
    let mut theme = crate::Theme::publication().resolve().unwrap();
    let one = RelativeSpacing::ratio(1, 1).unwrap();
    let zero = RelativeSpacing::ratio(0, 1).unwrap();
    theme.axes = theme
        .axes
        .clone()
        .frame(None)
        .unwrap()
        .ticks(None)
        .unwrap()
        .grid(None)
        .unwrap()
        .spacing(
            AxisSpacing::new(
                zero.clone(),
                zero.clone(),
                one.clone(),
                one.clone(),
                one,
                zero,
            )
            .unwrap(),
        )
        .unwrap();
    let panel = crate::layout::PanelBox::new(150.0, 650.0, 150.0, 650.0);
    let mut layout = Layout::envelope(panel);
    layout.x_axis.label = Some(
        crate::Text::new("Long horizontal title\nshort")
            .style(
                theme
                    .typography
                    .axes
                    .axis_title
                    .clone()
                    .font_size(23.0)
                    .unwrap(),
            )
            .alignment(crate::TextAlignment::Start),
    );
    layout.y_axis.label = Some(
        crate::Text::new("Long vertical title\nshort")
            .style(
                theme
                    .typography
                    .axes
                    .axis_title
                    .clone()
                    .font_size(19.0)
                    .unwrap(),
            )
            .alignment(crate::TextAlignment::End),
    );
    let aligned = layout.plan_axes(&theme).unwrap();
    let (x, xp) = aligned.x_axis.title.as_ref().unwrap();
    let (y, yp) = aligned.y_axis.title.as_ref().unwrap();
    assert_eq!(x.style.font_size.to_bits(), 23.0_f64.to_bits());
    assert_eq!(y.style.font_size.to_bits(), 19.0_f64.to_bits());
    assert!(x.style.exact_override && y.style.exact_override);
    assert_eq!(
        x.lines[0].translation_x_quanta + x.lines[0].ink_quanta.unwrap().left,
        x.lines[1].translation_x_quanta + x.lines[1].ink_quanta.unwrap().left
    );
    assert_eq!(
        y.lines[0].translation_x_quanta + y.lines[0].ink_quanta.unwrap().right,
        y.lines[1].translation_x_quanta + y.lines[1].ink_quanta.unwrap().right
    );
    assert!((aligned.requirements().bottom - x.height() - 23.0).abs() <= 1.0 / 1024.0);
    assert!((aligned.requirements().left - y.height() - 19.0).abs() <= 1.0 / 1024.0);
    assert_eq!(yp.rotation.to_bits(), (-90.0_f64).to_bits());
    // Resetting line alignment keeps the block's semantic center and measured gutter.
    layout.x_axis.label = Some(
        layout
            .x_axis
            .label
            .take()
            .unwrap()
            .layout(crate::TextLayout::default()),
    );
    layout.y_axis.label = Some(
        layout
            .y_axis
            .label
            .take()
            .unwrap()
            .layout(crate::TextLayout::default()),
    );
    let centered = layout.plan_axes(&theme).unwrap();
    let (cx, cxp) = centered.x_axis.title.as_ref().unwrap();
    let (cy, cyp) = centered.y_axis.title.as_ref().unwrap();
    assert!((xp.position.x + x.width() / 2.0 - cxp.position.x).abs() <= 1.0 / 1024.0);
    assert!((yp.position.y - y.width() / 2.0 - cyp.position.y).abs() <= 1.0 / 1024.0);
    assert_eq!(x.height_quanta, cx.height_quanta);
    assert_eq!(y.height_quanta, cy.height_quanta);
    let mut backend = crate::render::svg_backend::SvgBackend::in_memory(800, 800);
    backend.begin(800, 800).unwrap();
    aligned.draw(&mut backend).unwrap();
    backend.end().unwrap();
}

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
        let mut theme = crate::Theme::publication().resolve().unwrap();
        let stroke = theme.axes.get_frame().unwrap().get_stroke().clone();
        theme.axes = theme
            .axes
            .clone()
            .frame(sides.map(|sides| FrameTheme::new(sides, stroke).unwrap()))
            .unwrap()
            .ticks(None)
            .unwrap()
            .grid(None)
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
    let mut theme = crate::Theme::publication().resolve().unwrap();
    let cell = theme
        .axes
        .get_cell_frame()
        .unwrap()
        .clone()
        .sides(FrameSides::new(true, true, true, true).unwrap())
        .unwrap();
    theme.axes = theme
        .axes
        .clone()
        .frame(None)
        .unwrap()
        .cell_frame(Some(cell))
        .unwrap()
        .ticks(None)
        .unwrap()
        .grid(None)
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
    let mut theme = crate::Theme::publication().resolve().unwrap();
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
        .clone()
        .frame(None)
        .unwrap()
        .ticks(None)
        .unwrap()
        .grid(None)
        .unwrap()
        .spacing(spacing)
        .unwrap();
    let mut layout = Layout::envelope(crate::layout::PanelBox::new(80.0, 480.0, 80.0, 480.0));
    layout.x_axis.source = TickSource::EXACT;
    layout.x_axis.ticks = vec![AxisTick::new(0.5, "I")];
    layout.x_axis.label = Some("I".into());
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
