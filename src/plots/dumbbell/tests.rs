use super::*;
use crate::{CategoryValues, Color, DumbbellEndpoints, NumericValues, PointShape};

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one endpoint paint contract checks marks, both literal compositions and canonical outline capture"
)]
fn exact_endpoint_rgba_reaches_marks_literal_keys_and_canonical_capture_without_outline_changes() {
    let colors = [Color::rgba(13, 29, 47, 0), Color::rgba(71, 83, 97, 131)];
    let source = Plot::dumbbell(
        CategoryValues::new("category", ["first", "second"]).unwrap(),
        NumericValues::new("left", [1.0, 2.0]),
        NumericValues::new("right", [3.0, 4.0]),
        DumbbellEndpoints::new("Left", colors[0], "Right", colors[1]).unwrap(),
    )
    .erase();
    let mut theme = crate::Theme::publication().resolve().unwrap();
    let outline = crate::theme::PointOutlineTheme::new(
        Color::rgba(101, 113, 127, 61),
        crate::theme::RelativeSpacing::ratio(1, 4).unwrap(),
    )
    .unwrap();
    let point = crate::theme::PointTheme::new(17, PointShape::TriangleUp, Some(outline)).unwrap();
    let radii = theme
        .marks
        .get_points()
        .get_radii()
        .clone()
        .dumbbell(6)
        .unwrap();
    let points = theme
        .marks
        .get_points()
        .clone()
        .primary(point)
        .unwrap()
        .radii(radii)
        .unwrap();
    theme.marks = theme.marks.clone().points(points).unwrap();
    let chart = crate::plots::shared_x::prepare_panel_for_test(&source, &theme).unwrap();
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
        let style = &payload.points[0][index];
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
        expected.push((
            crate::decorations::legend::GuideEntryIdentity::endpoint(label).unwrap(),
            crate::decorations::legend::ResolvedGuideKey::point(style.clone()),
        ));
    }
    let guide = measured.discrete_guide.as_ref().unwrap();
    assert_eq!(guide.counts.entries, 2);
    let separate = crate::decorations::legend::discrete::PreparedGuide::prepare_for_test(
        &chart,
        &theme,
        crate::LegendComposition::Separate,
    )
    .unwrap()
    .unwrap();
    assert!(matches!(
        separate.sections[0].identity,
        crate::decorations::legend::discrete::GuideSectionIdentity::Literal(
            crate::decorations::legend::source::LiteralDomain::DumbbellEndpoints
        )
    ));
    for (complete, separate) in guide.sections[0]
        .entries()
        .iter()
        .zip(separate.sections[0].entries().iter())
    {
        assert_eq!(complete.key, separate.key);
        assert_eq!(complete.display_label, separate.display_label);
    }
    for (actual, (identity, key)) in guide.sections[0].entries().iter().zip(expected) {
        assert_eq!(actual.identity, identity);
        assert_eq!(actual.key, key);
    }
    assert_eq!(
        payload.connectors[0],
        theme
            .marks
            .get_strokes()
            .get_connectors()
            .get_dumbbell_connector()
            .backend_style()
    );
    let mapped = source
        .with_alpha(
            crate::alpha::AlphaTarget::Point,
            crate::alpha::AlphaEncoding::observations(
                crate::alpha::AlphaValues::new("confidence", [64, 230]).unwrap(),
            ),
        )
        .with_alpha(
            crate::alpha::AlphaTarget::Line,
            crate::alpha::AlphaEncoding::observations(
                crate::alpha::AlphaValues::new("pair confidence", [0, 191]).unwrap(),
            ),
        );
    let chart = crate::plots::shared_x::prepare_panel_for_test(&mapped, &theme).unwrap();
    let selected = chart
        .measure_selected_interior(PlotRenderContext::new(
            chart.data(),
            crate::layout::PanelBox::new(0.0, 600.0, 0.0, 400.0),
            &theme,
        ))
        .unwrap();
    let crate::layout::interior::MarkPayload::Dumbbell(payload) =
        &selected.marks.as_ref().unwrap().payload
    else {
        panic!("Dumbbell payload")
    };
    for (styles, alpha) in payload.points.iter().zip([64, 230]) {
        for (style, color) in styles.iter().zip(colors) {
            assert_eq!(style.color, color.alpha(alpha));
            assert_eq!(
                style.outline.as_ref().unwrap().color,
                Color::rgba(101, 113, 127, 61)
            );
        }
    }
    assert_eq!(
        payload
            .connectors
            .iter()
            .map(|style| style.color.get_alpha())
            .collect::<Vec<_>>(),
        [0, 191]
    );
}
