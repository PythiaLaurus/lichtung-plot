use super::*;

pub(super) fn capture_counters() -> [usize; 6] {
    [
        crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get),
        crate::layout::Projection::test_projection_calls(),
        crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get),
        crate::plots::shared_x::AXIS_PREPARATIONS.with(std::cell::Cell::get),
        crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(std::cell::Cell::get),
        crate::orchestrator::SELECTED_INTERIOR_MEASUREMENTS.with(std::cell::Cell::get),
    ]
}

/// Parent interaction uses representative coordinate, categorical, grid and
/// distribution children. The caller separately exercises every family as a root.
pub(crate) fn mixed_family_parent_audit(plots: &[(PlotKind, Plot)]) {
    assert_eq!(plots.len(), 25);
    let mut canvas = Canvas::grid(2, 2)
        .unwrap()
        .size(1601, 1201)
        .title("Mixed Canvas families")
        .subtitle("Retained child scopes")
        .caption("Independent root caption")
        .labels(Labels::raw().alias("x", "Shared sample"));
    let mut index = 0;
    for (kind, plot) in plots {
        if *kind == PlotKind::Matrix {
            let mut rejected = Canvas::grid(1, 1).unwrap();
            rejected.panels[0] = Some(plot.clone());
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
            assert!(rejected.preflight_plan().is_err());
            assert_eq!(
                crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
                0
            );
            continue;
        }
        if !matches!(
            kind,
            PlotKind::Scatter | PlotKind::HorizontalBar | PlotKind::Heatmap | PlotKind::Raincloud
        ) {
            continue;
        }
        canvas.panels[index] = Some(plot.clone().title(format!("{kind:?}")));
        index += 1;
    }
    assert_eq!(index, 4);
    crate::render::geometry::GEOMETRY_BACKEND_CONSTRUCTIONS.with(|calls| calls.set(0));
    crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(|calls| calls.set(0));
    crate::orchestrator::measurements::start();
    let cached = canvas.render_scene().unwrap();
    let work = crate::orchestrator::measurements::finish();
    assert_eq!(work.calls[2], 1);
    assert_eq!(work.calls[3], 1);
    assert_eq!(
        work.capture_counters,
        [0; crate::orchestrator::measurements::Counter::ALL.len()]
    );
    crate::decorations::plan::children::DISABLE_CHILD_REUSE.with(|disabled| disabled.set(true));
    let uncached = canvas.render_scene();
    crate::decorations::plan::children::DISABLE_CHILD_REUSE.with(|disabled| disabled.set(false));
    assert_eq!(cached.svg_bytes(), uncached.unwrap().svg_bytes());
    crate::render::geometry::GEOMETRY_BACKEND_CONSTRUCTIONS
        .with(|calls| assert_eq!(calls.get(), 0));
    crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(|calls| assert_eq!(calls.get(), 0));
}
