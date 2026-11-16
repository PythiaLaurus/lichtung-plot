use super::*;
use crate::{TextBounds, TextFit, TextLayout, TextWrapping};

fn selected(
    theme: &crate::text::TextRequestContext<'_, '_>,
    source: &str,
    appearance: &TextTheme,
    request: &TextLayout,
    width: i64,
) -> FitOutcome<ResolvedTextPlan> {
    measure(
        theme,
        source,
        TextAppearance::from(appearance).layout(Some(request)),
        width,
        PhysicalTextAlignment::Start,
        "font fit contract",
        &FitScope::root(),
    )
    .unwrap()
}

#[test]
fn priorities_keep_complete_content_or_preferred_size_and_floor_actions_remain_explicit() {
    use crate::{TextElision, TextEllipsis};
    let theme = crate::Theme::publication().resolve().unwrap();
    let text = crate::text::TextService::default();
    let theme = crate::text::TextRequestContext::new(&theme, &text);
    let style = theme
        .typography
        .root
        .caption
        .clone()
        .font_size(14.0)
        .unwrap();
    let source = "Measured scientific title";
    let request = TextLayout::default()
        .wrapping(TextWrapping::NoSoftWrap)
        .fit(TextFit::shrink_to(8.0).unwrap())
        .overflow(TextOverflow::ellipsis(
            TextEllipsis::new(TextElision::End),
            TextFailure::Error,
        ));
    let FitOutcome::Measured(complete) = selected(&theme, source, &style, &request, 115 * 1024)
    else {
        panic!("content first");
    };
    assert_eq!(complete.source, source);
    assert!(complete.style.font_size < 14.0);
    let preferred = request.fit_priority(TextFitPriority::PreferredSizeFirst);
    let FitOutcome::Measured(marked) = selected(&theme, source, &style, &preferred, 115 * 1024)
    else {
        panic!("preferred size first");
    };
    assert_eq!(marked.original_source(), source);
    assert_ne!(marked.source, source);
    assert_eq!(marked.style.font_size.to_bits(), 14.0_f64.to_bits());
    for failure in [
        TextFailure::Error,
        TextFailure::Hide,
        TextFailure::Clip,
        TextFailure::Visible,
    ] {
        let overflow = match failure {
            TextFailure::Error => TextOverflow::complete(),
            TextFailure::Hide => TextOverflow::hide(),
            TextFailure::Clip => TextOverflow::clip(),
            TextFailure::Visible => TextOverflow::visible(),
        };
        let request = TextLayout::default()
            .fit(TextFit::shrink_to(10.0).unwrap())
            .bounds(TextBounds::new(Some(0.0), Some(0.0), None).unwrap())
            .overflow(overflow);
        match selected(&theme, source, &style, &request, 115 * 1024) {
            FitOutcome::Miss(_) => assert_eq!(failure, TextFailure::Error),
            FitOutcome::Measured(plan) => {
                assert_eq!(plan.style.font_size.to_bits(), 10.0_f64.to_bits());
                assert_eq!(plan.original_source(), source);
                if failure == TextFailure::Hide {
                    assert!(plan.lines.is_empty());
                } else if failure == TextFailure::Clip {
                    assert_eq!(plan.occupied_size(), (0.0, 0.0));
                } else {
                    assert!(!plan.lines.is_empty());
                }
            }
        }
    }
}
