use super::*;
use crate::alpha::{AlphaEncoding, AlphaValues};

#[test]
fn every_off_diagonal_replays_original_observation_alpha_without_changing_diagonal_sources() {
    let source = Plot::matrix(vec![
        crate::NumericValues::new("a", [1.0, 2.0, 3.0]),
        crate::NumericValues::new("b", [2.0, 5.0, 7.0]),
    ])
    .hue(crate::CategoryValues::new("cohort", ["seen"; 3]).unwrap())
    .source_rows(crate::SourceRows::new([100, 200, 300]).unwrap())
    .with_alpha(
        AlphaTarget::Point,
        AlphaEncoding::observations(AlphaValues::new("confidence", [0, 128, 0]).unwrap()),
    )
    .with_alpha(
        AlphaTarget::Fill,
        AlphaEncoding::keyed(
            [crate::CategoryValues::new("cohort", ["\u{10ffff}", "seen"]).unwrap()],
            AlphaValues::new("density confidence", [0, 64]).unwrap(),
        )
        .unwrap(),
    )
    .hide_point_outline()
    .erase();
    let prepared = prepare_axes(
        &source,
        crate::data::PlotData::with_rows(&source, &[2, 0, 1], &[300, 100, 200]),
    )
    .unwrap();
    assert_eq!(
        prepared.groups[0]
            .alphas
            .iter()
            .map(|alpha| (alpha.slot, alpha.source_row, alpha.value.unwrap().alpha))
            .collect::<Vec<_>>(),
        [(2, 300, 0), (0, 100, 0), (1, 200, 128)]
    );
    let data = source
        .data()
        .map_source_rows(source.source_row_map().unwrap());
    let before = prepare_axes(&source, data).unwrap();
    let control = source
        .clone()
        .with_alpha(
            AlphaTarget::Point,
            AlphaEncoding::single(AlphaValues::new("confidence", [0]).unwrap()).unwrap(),
        )
        .with_alpha(
            AlphaTarget::Fill,
            AlphaEncoding::single(AlphaValues::new("density confidence", [0]).unwrap()).unwrap(),
        );
    let control_data = control
        .data()
        .map_source_rows(control.source_row_map().unwrap());
    let after = prepare_axes(&control, control_data).unwrap();
    assert_eq!(before.ranges, after.ranges);
    for (before, after) in before.diagonals.iter().zip(&after.diagonals) {
        assert_eq!(before.positions, after.positions);
        assert_eq!(before.densities, after.densities);
        assert_eq!(before.maximum.to_bits(), after.maximum.to_bits());
    }
    let mut theme = crate::Theme::publication().resolve().unwrap();
    let primary = theme
        .marks
        .get_points()
        .get_primary()
        .clone()
        .fill_alpha(0)
        .unwrap();
    let points = theme.marks.get_points().clone().primary(primary).unwrap();
    theme.marks = theme.marks.clone().points(points).unwrap();
    let chart = crate::plots::shared_x::prepare_panel_for_test(&source, &theme).unwrap();
    let selected = chart
        .measure_selected_interior(PlotRenderContext::new(
            chart.data(),
            PanelBox::new(20.0, 800.0, 20.0, 620.0),
            &theme,
        ))
        .unwrap();
    let layout::interior::MarkPayload::Matrix(payload) = &selected.marks.as_ref().unwrap().payload
    else {
        panic!("Matrix payload")
    };
    assert_eq!(payload.cells.len(), 4);
    for index in [1, 2] {
        let points = payload.cells[index]
            .marks
            .iter()
            .map(|mark| match mark {
                MatrixPrimitive::Scatter(points, members, style) => {
                    assert_eq!(points.len(), members.len());
                    assert!(style.outline.is_none());
                    (
                        members[0].slot,
                        members[0].source_row,
                        style.color.get_alpha(),
                    )
                }
                MatrixPrimitive::Polygon(..) => panic!("off-diagonal polygon"),
            })
            .collect::<Vec<_>>();
        assert_eq!(points, [(0, 100, 0), (1, 200, 128), (2, 300, 0)]);
    }
    for index in [0, 3] {
        assert!(
            matches!(&payload.cells[index].marks[0], MatrixPrimitive::Polygon(_, style) if style.color.get_alpha() == 64 && style.stroke.get_alpha() == 255)
        );
    }
}
