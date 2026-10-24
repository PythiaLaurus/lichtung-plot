use super::*;
use crate::layout::fit;
use crate::{Text, TextBounds, TextFailure, TextLayout};

fn measured(
    source: &str,
    layout: TextLayout,
    width: f64,
) -> Result<ResolvedTextPlan, fit::PlanError> {
    let theme = crate::Theme::publication().resolve().unwrap();
    let text = Text::new(source).layout(layout);
    fit::measured(fit::text(
        &theme,
        source,
        text.appearance(&theme.typography.root.caption),
        width,
        PhysicalTextAlignment::Start,
        "ellipsis witness",
        &FitScope::root(),
    ))
}

#[test]
fn derived_end_start_and_measured_middle_keep_whole_graphemes_and_original_mapping() {
    let source = "A\u{301}lpha_context_very_long_ending";
    let boundaries = source
        .grapheme_indices(true)
        .map(|(index, _)| index)
        .chain(std::iter::once(source.len()))
        .collect::<Vec<_>>();
    for position in [TextElision::End, TextElision::Start, TextElision::Middle] {
        let request = TextLayout::default().overflow(TextOverflow::ellipsis(
            TextEllipsis::new(position).marker("~"),
            TextFailure::Error,
        ));
        let plan = measured(source, request, 70.0).unwrap();
        assert_eq!(plan.original_source(), source);
        assert_ne!(plan.source, source);
        assert!(plan.width() <= 70.0);
        assert_eq!(plan.lines.len(), 1);
        let map = plan.display_map.as_ref().unwrap();
        for fragment in map.iter() {
            if let Some(span) = &fragment.original {
                assert!(boundaries.contains(&span.start) && boundaries.contains(&span.end));
                assert_eq!(
                    &plan.source[fragment.display.clone()],
                    &source[span.clone()]
                );
            }
        }
        assert!(map
            .iter()
            .any(|fragment| fragment.original.is_none()
                && &plan.source[fragment.display.clone()] == "~"));
        match position {
            TextElision::End => {
                assert!(plan.source.starts_with("A\u{301}") && plan.source.ends_with('~'));
            }
            TextElision::Start => {
                assert!(plan.source.starts_with('~') && plan.source.ends_with("ending"));
            }
            TextElision::Middle => {
                assert!(plan.source.starts_with("A\u{301}") && plan.source.ends_with('g'));
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn mandatory_line_caps_bidi_context_and_unused_marker_admission_are_independent() {
    let multiline = "First\nSecond\nThird";
    let plan = measured(
        multiline,
        TextLayout::default().overflow(TextOverflow::ellipsis(
            TextEllipsis::new(TextElision::End).max_lines(Some(2)),
            TextFailure::Error,
        )),
        100.0,
    )
    .unwrap();
    assert_eq!(plan.original_source(), multiline);
    assert_eq!(plan.lines.len(), 2);
    assert!(plan.source.starts_with("First\n"));
    let source = "אבגדהוזחטיכלמנסעפצקרשת";
    let request = TextLayout::default()
        .direction(TextDirection::RightToLeft)
        .overflow(TextOverflow::ellipsis(
            TextEllipsis::new(TextElision::Left),
            TextFailure::Error,
        ));
    let plan = measured(source, request, 60.0).unwrap();
    assert!(plan.source.ends_with('…'));
    assert_eq!(
        plan.lines[0].base_direction,
        crate::font::canonical::ShapingDirection::RightToLeft
    );
    let request = TextLayout::default().overflow(TextOverflow::ellipsis(
        TextEllipsis::new(TextElision::End).marker("\u{10ffff}"),
        TextFailure::Error,
    ));
    assert!(measured("A", request.clone(), 100.0).is_ok());
    assert!(measured("A long source", request, 1.0)
        .unwrap_err()
        .into_error()
        .to_string()
        .contains("U+10FFFF"));
}

#[test]
fn no_marker_room_consumes_only_the_declared_terminal_action_and_empty_markers_work() {
    let source = "Original source";
    for failure in [
        TextFailure::Error,
        TextFailure::Hide,
        TextFailure::Clip,
        TextFailure::Visible,
    ] {
        let request = TextLayout::default()
            .bounds(TextBounds::new(Some(0.0), Some(0.0), Some(0)).unwrap())
            .overflow(TextOverflow::ellipsis(
                TextEllipsis::new(TextElision::End),
                failure,
            ));
        let result = measured(source, request, 20.0);
        if failure == TextFailure::Error {
            assert!(matches!(result, Err(fit::PlanError::Fit(_))));
            continue;
        }
        let plan = result.unwrap();
        assert_eq!(plan.original_source(), source);
        if failure == TextFailure::Hide {
            assert!(plan.lines.is_empty());
        } else if failure == TextFailure::Clip {
            assert_eq!(plan.occupied_size(), (0.0, 0.0));
        } else {
            assert!(!plan.lines.is_empty());
        }
    }
    let plan = measured(
        source,
        TextLayout::default().overflow(TextOverflow::ellipsis(
            TextEllipsis::new(TextElision::End).marker(""),
            TextFailure::Error,
        )),
        30.0,
    )
    .unwrap();
    assert!(source.starts_with(&plan.source));
    assert_eq!(plan.original_source(), source);
    assert!(plan.width() <= 30.0);
}
