use super::*;
use crate::alpha::{AlphaEncoding, AlphaValues};

#[test]
fn alpha_preserves_original_filtered_slots_coincident_points_paint_order_and_outlines() {
    let source = Plot::scatter(
        crate::NumericValues::new("x", [1.0, f64::NAN, 1.0, 1.0]),
        crate::NumericValues::new("y", [1.0, 2.0, 1.0, 1.0]),
    )
    .source_rows(crate::SourceRows::new([100, 200, 300, 400]).unwrap())
    .with_alpha(
        AlphaTarget::Point,
        AlphaEncoding::observations(AlphaValues::new("confidence", [0, 42, 128, 0]).unwrap()),
    )
    .x_limits(0.0, 2.0)
    .y_limits(0.0, 2.0)
    .legend(crate::LegendPolicy::Hidden)
    .erase();
    let data = source
        .data()
        .map_source_rows(source.source_row_map().unwrap());
    let prepared = PreparedScatter::new(&source, data).unwrap();
    assert_eq!(prepared.checked_primary_mark_count().unwrap(), 3);
    assert_eq!(
        prepared.groups[0]
            .points
            .iter()
            .map(|point| (point.alpha.slot, point.alpha.source_row))
            .collect::<Vec<_>>(),
        [(0, 100), (2, 300), (3, 400)]
    );
    assert_eq!(prepared.ranges, [(1.0, 1.0); 2]);
    let theme = crate::Theme::publication().resolve().unwrap();
    let chart = crate::plots::shared_x::prepare_panel_for_test(&source, &theme).unwrap();
    let selected = chart
        .measure_selected_interior(PlotRenderContext::new(
            chart.data(),
            crate::layout::PanelBox::new(10.0, 600.0, 20.0, 420.0),
            &theme,
        ))
        .unwrap();
    let crate::layout::interior::MarkPayload::Scatter(payload) =
        &selected.marks.as_ref().unwrap().payload
    else {
        panic!("Scatter payload")
    };
    assert_eq!(
        payload
            .groups
            .iter()
            .map(|(points, style)| (points.len(), style.color.alpha()))
            .collect::<Vec<_>>(),
        [(1, 0), (1, 128), (1, 0)]
    );
    let outline = chart
        .point_style(&theme, (0, 0, 0), theme.marks.points().radii().scatter())
        .outline;
    assert!(payload
        .groups
        .iter()
        .all(|(_, style)| style.outline == outline));
    let first = payload.groups[0].0[0];
    assert!(payload.groups.iter().all(|(points, _)| points[0] == first));
    let calls = crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get);
    let error = source
        .with_alpha(
            AlphaTarget::Point,
            AlphaEncoding::observations(AlphaValues::new("confidence", [0; 3]).unwrap()),
        )
        .preflight()
        .unwrap_err();
    assert!(matches!(error, Error::Data(_)));
    assert_eq!(
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        calls
    );
}
