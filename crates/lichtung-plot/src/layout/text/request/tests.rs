use super::*;
use crate::layout::fit::{self, FitScope};
use crate::{Text, TextBounds, TextLayout, TextOverflow, TextVisibility};

#[test]
fn explicit_zero_bounds_preserve_source_and_consume_only_the_selected_terminal_action() {
    let theme = crate::Theme::publication().resolve().unwrap();
    let scope = FitScope::root();
    let source = "First line\nSecond line";
    let bounds = TextBounds::new(Some(0.0), Some(0.0), Some(0)).unwrap();
    let complete = Text::new(source).layout(TextLayout::default().bounds(bounds));
    assert!(matches!(
        measure(
            &theme,
            source,
            complete.appearance(&theme.typography.root.title),
            200.0,
            PhysicalTextAlignment::Start,
            "bounded text",
            &scope
        )
        .unwrap(),
        FitOutcome::Miss(_)
    ));
    let clipped = Text::new(source).layout(
        TextLayout::default()
            .bounds(bounds)
            .overflow(TextOverflow::clip()),
    );
    let plan = fit::measured(measure(
        &theme,
        source,
        clipped.appearance(&theme.typography.root.title),
        200.0,
        PhysicalTextAlignment::Start,
        "bounded text",
        &scope,
    ))
    .unwrap();
    assert_eq!(plan.source, source);
    assert!(!plan.lines.is_empty());
    assert_eq!(plan.occupied_size(), (0.0, 0.0));
    let placement = crate::render::backend::PlannedTextPlacement::middle(
        crate::render::backend::Point2D::new(40.0, 50.0),
        crate::render::backend::TextAnchor::Start,
        0.0,
    );
    let mut geometry = crate::render::geometry::GeometrySnapshot::new();
    geometry.text(placement, &plan, None).unwrap();
    assert!(geometry.bounds().is_empty());
    let hidden = Text::new(source).layout(
        TextLayout::default()
            .bounds(bounds)
            .overflow(TextOverflow::hide()),
    );
    let plan = fit::measured(measure(
        &theme,
        source,
        hidden.appearance(&theme.typography.root.title),
        200.0,
        PhysicalTextAlignment::Start,
        "bounded text",
        &scope,
    ))
    .unwrap();
    assert_eq!(plan.source, source);
    assert!(plan.lines.is_empty());
}

#[test]
fn explicit_hidden_text_does_not_consume_its_unavailable_exact_font() {
    let theme = crate::Theme::publication().resolve().unwrap();
    let style = theme
        .typography
        .root
        .title
        .clone()
        .stack(crate::font::canonical::named_stack(&[
            "Unused hidden font that does not exist",
        ]))
        .unwrap();
    let text = Text::new("Hidden Ω source")
        .style(style)
        .layout(TextLayout::default().visibility(TextVisibility::hidden()));
    let plan = fit::measured(fit::text(
        &theme,
        text.as_str(),
        text.appearance(&theme.typography.root.title),
        0.0,
        PhysicalTextAlignment::Start,
        "hidden exact text",
        &FitScope::root(),
    ))
    .unwrap();
    assert_eq!(plan.source, text.as_str());
    assert!(plan.lines.is_empty() && plan.style.exact_override);
}
