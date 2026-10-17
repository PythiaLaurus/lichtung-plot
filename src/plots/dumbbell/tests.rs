use super::*;
use crate::{CategoryValues, Color, DumbbellEndpoints, NumericValues, PointShape};

#[test]
fn exact_endpoint_rgba_reaches_marks_literal_keys_and_canonical_capture_without_outline_changes() {
    let colors = [Color::rgba(13, 29, 47, 0), Color::rgba(71, 83, 97, 131)];
    let source = Plot::dumbbell(
        CategoryValues::new("category", ["first", "second"]).unwrap(),
        NumericValues::new("left", [1.0, 2.0]),
        NumericValues::new("right", [3.0, 4.0]),
        DumbbellEndpoints::new("Left", colors[0], "Right", colors[1]).unwrap(),
    )
    .erase();
    let operation = crate::font::canonical::OperationContext::new();
    let inputs = crate::plots::shared_x::prepare_plot(&source, &operation, None).unwrap();
    let chart = inputs.attach(source).unwrap();
    let mut theme = crate::Theme::publication().resolve().unwrap();
    let outline = crate::theme::PointOutlineTheme::new(
        Color::rgba(101, 113, 127, 61),
        crate::theme::RelativeSpacing::ratio(1, 4).unwrap(),
    )
    .unwrap();
    let point = crate::theme::PointTheme::new(17, PointShape::TriangleUp, Some(outline)).unwrap();
    let radii = theme
        .marks
        .points()
        .radii()
        .clone()
        .with_dumbbell(6)
        .unwrap();
    let points = theme
        .marks
        .points()
        .clone()
        .with_primary(point)
        .unwrap()
        .with_radii(radii)
        .unwrap();
    theme.marks = theme.marks.clone().with_points(points).unwrap();
    let measured = chart
        .measure_selected_interior(PlotRenderContext::new(
            chart.data(),
            crate::layout::PanelBox::new(0.0, 600.0, 0.0, 400.0),
            &theme,
        ))
        .unwrap();
    let crate::layout::interior::MarkPayload::Dumbbell(payload) =
        &measured.marks.as_ref().unwrap().payload
    else {
        panic!("Dumbbell payload")
    };
    assert_eq!(payload.pairs.len(), 2);
    let mut expected = Vec::new();
    for (index, label) in ["Left", "Right"].into_iter().enumerate() {
        let style = &payload.points[index];
        assert_eq!(style.color, colors[index]);
        assert_eq!(style.radius, 6);
        assert_eq!(style.shape, PointShape::TriangleUp);
        assert_eq!(
            style.outline.as_ref().unwrap().color,
            Color::rgba(101, 113, 127, 61)
        );
        let captured = crate::render::primitives::CanonicalScatter::new(
            &[payload.pairs[0][index]],
            style,
            400,
            None,
        )
        .unwrap();
        assert_eq!(captured.color, colors[index]);
        let (stroke, width) = captured.outline.unwrap();
        assert_eq!(stroke, Color::rgba(101, 113, 127, 61));
        assert_eq!(width.units(), 1536);
        expected.push(LegendEntry::point(
            chart.point_legend_identity(label).unwrap(),
            style.clone(),
        ));
    }
    crate::decorations::legend::resolve_entries(&chart, &mut expected).unwrap();
    assert_eq!(measured.legend_entries.len(), 2);
    for (actual, expected) in measured.legend_entries.iter().zip(expected) {
        assert!(actual.same_child_input(&expected));
    }
    assert_eq!(
        payload.connector,
        theme
            .marks
            .strokes()
            .connectors()
            .dumbbell_connector()
            .backend_style()
    );
}
