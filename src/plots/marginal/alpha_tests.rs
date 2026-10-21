use super::*;
use crate::alpha::{AlphaEncoding, AlphaValues};

#[test]
fn central_alpha_retains_filtered_original_members_and_does_not_change_densities() {
    let source = Plot::marginal(
        crate::NumericValues::new("x", [1.0, f64::NAN, 3.0, 2.0]),
        crate::NumericValues::new("y", [1.0, 2.0, 5.0, 4.0]),
    )
    .source_rows(crate::SourceRows::new([100, 200, 300, 400]).unwrap())
    .with_alpha(
        AlphaTarget::Point,
        AlphaEncoding::observations(AlphaValues::new("confidence", [0, 17, 128, 0]).unwrap()),
    )
    .x_limits(-20.0, 20.0)
    .y_limits(-20.0, 20.0)
    .hide_point_outline()
    .erase();
    let data = source
        .data()
        .map_source_rows(source.source_row_map().unwrap());
    let prepared = finish(prepare(&source, data).unwrap(), None).unwrap();
    assert_eq!(prepared.checked_primary_mark_count().unwrap(), 3);
    assert_eq!(
        prepared.groups[0]
            .alphas
            .iter()
            .map(|alpha| (alpha.slot, alpha.source_row))
            .collect::<Vec<_>>(),
        [(0, 100), (2, 300), (3, 400)]
    );
    let control = source.clone().with_alpha(
        AlphaTarget::Point,
        AlphaEncoding::observations(AlphaValues::new("confidence", [0; 4]).unwrap()),
    );
    let control_data = control
        .data()
        .map_source_rows(control.source_row_map().unwrap());
    let control = finish(prepare(&control, control_data).unwrap(), None).unwrap();
    assert_eq!(prepared.ranges, control.ranges);
    assert_eq!(prepared.curves.x_densities, control.curves.x_densities);
    assert_eq!(prepared.curves.y_densities, control.curves.y_densities);
    let mut theme = crate::Theme::publication().resolve().unwrap();
    let primary = theme
        .marks
        .points()
        .primary()
        .clone()
        .with_fill_alpha(0)
        .unwrap();
    let points = theme.marks.points().clone().with_primary(primary).unwrap();
    theme.marks = theme.marks.clone().with_points(points).unwrap();
    let chart = crate::plots::shared_x::prepare_panel_for_test(&source, &theme).unwrap();
    let selected = chart
        .measure_selected_interior(PlotRenderContext::new(
            chart.data(),
            PanelBox::new(20.0, 800.0, 20.0, 620.0),
            &theme,
        ))
        .unwrap();
    let layout::interior::MarkPayload::Marginal(payload) =
        &selected.marks.as_ref().unwrap().payload
    else {
        panic!("Marginal payload")
    };
    assert_eq!(payload.groups.len(), 1);
    assert_eq!(payload.groups[0].source_rows, [100, 300, 400]);
    assert_eq!(
        payload.groups[0]
            .point_batches
            .iter()
            .map(|(points, style)| (points.len(), style.color.alpha()))
            .collect::<Vec<_>>(),
        [(1, 0), (1, 128), (1, 0)]
    );
    assert!(payload.groups[0]
        .point_batches
        .iter()
        .all(|(_, style)| style.outline.is_none()));
    assert_eq!(
        payload.groups[0].fill.color.alpha(),
        theme
            .marks
            .fills()
            .densities()
            .marginal_density()
            .fill_alpha()
    );
}
