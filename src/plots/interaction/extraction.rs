use super::*;
use crate::{Bounds, CategoryValues, NumericValues};

pub(crate) fn source() -> Plot<crate::kind::Interaction> {
    Plot::interaction(
        CategoryValues::new("condition", ["b", "a", "c", "b", "c"]).unwrap(),
        NumericValues::new("estimate", [2.0, 3.0, 4.0, 5.0, 6.0]),
        CategoryValues::new("group", ["one", "one", "one", "two", "two"]).unwrap(),
        Bounds::new(
            NumericValues::new("lower", [1.0, 2.0, 3.0, 4.0, 5.0]),
            NumericValues::new("upper", [3.0, 4.0, 5.0, 6.0, 7.0]),
        ),
    )
}

#[test]
fn lexical_style_selectors_preserve_raw_identity_through_prepared_group_matching() {
    use crate::InteractionStyle;

    for invalid in ["", " A", "A ", "A\nB", "A\u{7f}B"] {
        assert!(InteractionStyle::new(invalid).is_err(), "{invalid:?}");
    }
    let spaced = InteractionStyle::new("A  B")
        .unwrap()
        .with_point_shape(PointShape::TriangleUp)
        .unwrap()
        .with_line_pattern(LinePattern::Dashed)
        .unwrap();
    let single = InteractionStyle::new("A B").unwrap();
    let chart = Plot::interaction(
        CategoryValues::new("condition", ["before", "after", "before", "after"]).unwrap(),
        NumericValues::new("estimate", [1.0, 2.0, 3.0, 4.0]),
        CategoryValues::new("group", ["A B", "A B", "A  B", "A  B"]).unwrap(),
        Bounds::new(
            NumericValues::new("lower", [0.5, 1.5, 2.5, 3.5]),
            NumericValues::new("upper", [1.5, 2.5, 3.5, 4.5]),
        ),
    )
    .styles([spaced, single])
    .connectors(true)
    .labels(crate::Labels::raw())
    .with_alpha(
        crate::alpha::AlphaTarget::Point,
        crate::alpha::AlphaEncoding::observations(
            crate::alpha::AlphaValues::new("confidence", [0, 90, 230, 128]).unwrap(),
        ),
    )
    .with_alpha(
        crate::alpha::AlphaTarget::Line,
        crate::alpha::AlphaEncoding::keyed(
            [CategoryValues::new("group", ["A  B", "A B"]).unwrap()],
            crate::alpha::AlphaValues::new("trace confidence", [64, 0]).unwrap(),
        )
        .unwrap(),
    )
    .erase();
    let prepared = prepare(&chart, chart.data(), None).unwrap();
    assert_eq!(prepared.traces.len(), 2);
    assert_eq!(prepared.traces[0].identity.group.as_deref(), Some("A B"));
    assert_eq!(prepared.traces[0].style.point_shape, PointShape::Circle);
    assert_eq!(prepared.traces[0].style.line_pattern, LinePattern::Solid);
    assert_eq!(prepared.traces[1].identity.group.as_deref(), Some("A  B"));
    assert_eq!(prepared.traces[1].style.point_shape, PointShape::TriangleUp);
    assert_eq!(prepared.traces[1].style.line_pattern, LinePattern::Dashed);
    assert_eq!(prepared.checked_primary_mark_count().unwrap(), 4);
    assert_eq!(prepared.traces[0].cells[0].alpha.value.unwrap().alpha, 0);
    assert_eq!(prepared.traces[1].cells[0].alpha.value.unwrap().alpha, 230);
    let theme = crate::Theme::publication().resolve().unwrap();
    let chart = crate::plots::shared_x::prepare_panel_for_test(&chart, &theme).unwrap();
    let selected = chart
        .measure_selected_interior(PlotRenderContext::new(
            chart.data(),
            crate::layout::PanelBox::new(40.0, 600.0, 40.0, 420.0),
            &theme,
        ))
        .unwrap();
    let crate::layout::interior::MarkPayload::Interaction(payload) =
        &selected.marks.as_ref().unwrap().payload
    else {
        panic!("Interaction payload")
    };
    assert_eq!(payload.traces[0].markers[0].1.color.alpha(), 0);
    assert_eq!(payload.traces[1].markers[0].1.color.alpha(), 230);
    assert_eq!(payload.traces[0].interval.color.alpha(), 0);
    assert_eq!(
        payload.traces[0].connector.as_ref().unwrap().color.alpha(),
        0
    );
    assert_eq!(payload.traces[1].interval.color.alpha(), 64);
    assert_eq!(
        payload.traces[1].connector.as_ref().unwrap().color.alpha(),
        64
    );
    assert_eq!(
        payload.traces[1].connector.as_ref().unwrap().pattern,
        LinePattern::Dashed.stroke_pattern()
    );
}
