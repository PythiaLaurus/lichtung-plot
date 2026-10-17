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
        CategoryValues::new("condition", ["before", "before"]).unwrap(),
        NumericValues::new("estimate", [1.0, 2.0]),
        CategoryValues::new("group", ["A B", "A  B"]).unwrap(),
        Bounds::new(
            NumericValues::new("lower", [0.5, 1.5]),
            NumericValues::new("upper", [1.5, 2.5]),
        ),
    )
    .styles([spaced, single])
    .erase();
    let prepared = prepare(&chart, chart.data(), None).unwrap();
    assert_eq!(prepared.traces.len(), 2);
    assert_eq!(prepared.traces[0].identity.group.as_deref(), Some("A B"));
    assert_eq!(prepared.traces[0].style.point_shape, PointShape::Circle);
    assert_eq!(prepared.traces[0].style.line_pattern, LinePattern::Solid);
    assert_eq!(prepared.traces[1].identity.group.as_deref(), Some("A  B"));
    assert_eq!(prepared.traces[1].style.point_shape, PointShape::TriangleUp);
    assert_eq!(prepared.traces[1].style.line_pattern, LinePattern::Dashed);
    assert_eq!(prepared.checked_primary_mark_count().unwrap(), 2);
}
