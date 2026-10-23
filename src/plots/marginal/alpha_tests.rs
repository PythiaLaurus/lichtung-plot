use super::*;
use crate::alpha::{AlphaEncoding, AlphaValues};

#[test]
fn central_alpha_retains_filtered_original_members_and_does_not_change_densities() {
    let source = Plot::marginal(
        crate::NumericValues::new("x", [1.0, f64::NAN, 3.0, 2.0, 10.0, 12.0, 11.0]),
        crate::NumericValues::new("y", [1.0, 2.0, 5.0, 4.0, 10.0, 15.0, 14.0]),
    )
    .hue(crate::CategoryValues::new("cohort", ["a", "filtered", "a", "a", "b", "b", "b"]).unwrap())
    .source_rows(crate::SourceRows::new([100, 200, 300, 400, 500, 600, 700]).unwrap())
    .with_alpha(
        AlphaTarget::Point,
        AlphaEncoding::observations(
            AlphaValues::new("confidence", [0, 17, 128, 0, 0, 64, 230]).unwrap(),
        ),
    )
    .with_alpha(
        AlphaTarget::Fill,
        AlphaEncoding::keyed(
            [crate::CategoryValues::new("cohort", ["\u{10ffff}", "b", "a"]).unwrap()],
            AlphaValues::new("density confidence", [99, 0, 230]).unwrap(),
        )
        .unwrap(),
    )
    .x_limits(-20.0, 20.0)
    .y_limits(-20.0, 20.0)
    .hide_point_outline()
    .erase();
    let data = source
        .data()
        .map_source_rows(source.source_row_map().unwrap());
    let prepared = finish(prepare(&source, data).unwrap(), None).unwrap();
    assert_eq!(prepared.checked_primary_mark_count().unwrap(), 6);
    assert_eq!(
        prepared
            .groups
            .iter()
            .map(|group| group.fill_alpha.unwrap().alpha)
            .collect::<Vec<_>>(),
        [230, 0]
    );
    assert_eq!(
        prepared.groups[0]
            .alphas
            .iter()
            .map(|alpha| (alpha.slot, alpha.source_row))
            .collect::<Vec<_>>(),
        [(0, 100), (2, 300), (3, 400)]
    );
    let control = source
        .clone()
        .with_alpha(
            AlphaTarget::Point,
            AlphaEncoding::observations(AlphaValues::new("confidence", [0; 7]).unwrap()),
        )
        .with_alpha(
            AlphaTarget::Fill,
            AlphaEncoding::single(AlphaValues::new("density confidence", [0]).unwrap()).unwrap(),
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
        .fill_alpha(0)
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
    assert_eq!(payload.groups.len(), 2);
    assert_eq!(payload.groups[0].source_rows, [100, 300, 400]);
    assert_eq!(
        payload.groups[0]
            .point_batches
            .iter()
            .map(|(points, style)| (points.len(), style.color.get_alpha()))
            .collect::<Vec<_>>(),
        [(1, 0), (1, 128), (1, 0)]
    );
    assert!(payload.groups[0]
        .point_batches
        .iter()
        .all(|(_, style)| style.outline.is_none()));
    assert_eq!(
        payload
            .groups
            .iter()
            .map(|group| group.fill.color.get_alpha())
            .collect::<Vec<_>>(),
        [230, 0]
    );
    assert_eq!(payload.groups[1].source_rows, [500, 600, 700]);
    assert_eq!(
        payload.groups[1]
            .point_batches
            .iter()
            .map(|(points, style)| (points.len(), style.color.get_alpha()))
            .collect::<Vec<_>>(),
        [(1, 0), (1, 64), (1, 230)]
    );
    assert!(payload
        .groups
        .iter()
        .all(|group| group.fill.stroke.get_alpha() == 255));
}
