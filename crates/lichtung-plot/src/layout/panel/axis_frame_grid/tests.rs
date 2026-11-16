use super::*;
use crate::layout::PixelRect;

fn layout(panel: PixelRect, x: Vec<AxisTick>, y: Vec<AxisTick>, source: TickSource) -> Layout {
    let mut layout = Layout::envelope(PixelRect::new(80.0, 770.0, 60.0, 570.0));
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
fn numeric_marks_ticks_and_grid_share_odd_quantum_spans_and_fractional_origins() {
    let theme = crate::Theme::publication().resolve().unwrap();
    let text = crate::text::TextService::default();
    let theme = crate::text::TextRequestContext::new(&theme, &text);
    for origin in [0.0, 1.0 / 1024.0, 0.1, -0.1] {
        let end = origin + 81_923.0 / 1024.0;
        let mut layout = layout(
            PixelRect::new(origin, end, origin, end),
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
