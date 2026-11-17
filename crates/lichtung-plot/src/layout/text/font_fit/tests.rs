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

#[test]
fn actual_source_metrics_not_unused_primary_or_candidates_bound_fit() {
    use crate::font::canonical::{named_stack, PreparedFontEnvironment};
    use crate::layout::text::metric_tests::font;
    let appearance = crate::Theme::publication().resolve().unwrap();
    let text = crate::text::TextService::default();
    let environment = PreparedFontEnvironment::synthetic_shaping_fonts(vec![
        font("Primary", &[' ', 'X'], Some((0, 0)), (0, 0)),
        font("Used", &[' ', 'A'], Some((800, 0)), (900, -300)),
        font("Unused", &['B'], Some((0, 0)), (0, 0)),
    ]);
    text.fonts.set(Ok(environment.clone())).unwrap();
    let theme = crate::text::TextRequestContext::new(&appearance, &text);
    let style = theme
        .typography
        .root
        .caption
        .clone()
        .stack(named_stack(&["Primary", "Used", "Unused"]))
        .unwrap()
        .font_size(10.0)
        .unwrap();
    let request = TextLayout::default()
        .wrapping(TextWrapping::NoSoftWrap)
        .fit(TextFit::shrink_to(2.0).unwrap());
    crate::font::canonical::FONT_METRIC_QUERIES.with(|queries| queries.borrow_mut().clear());
    let FitOutcome::Measured(measured) = selected(&theme, "AA", &style, &request, 7 * 1024) else {
        panic!("source fits after shrinking")
    };
    assert_eq!(measured.source, "AA");
    assert!(measured.style.font_size < 10.0 && measured.style.font_size >= 2.0);
    assert_eq!(measured.pair_provenance.candidates.len(), 3);
    crate::font::canonical::FONT_METRIC_QUERIES.with(|queries| {
        assert!(!queries.borrow().is_empty());
        assert!(queries
            .borrow()
            .iter()
            .all(|identity| *identity == measured.pair_provenance.candidates[1]));
    });
    assert!(measured
        .lines
        .iter()
        .flat_map(|line| &line.runs)
        .all(|run| run.face == measured.pair_provenance.candidates[1]));
    let lower = minimum_height(
        &theme,
        "AA",
        TextAppearance::from(&style).layout(Some(&request)),
    )
    .unwrap()
    .unwrap();
    assert_eq!(lower, (800 * 2 * 1024 + 999) / 1000);
    assert!(lower <= measured.height_quanta);
    // Actually selecting the no-ink space in Primary is fatal. Neither the
    // valid fallback nor another fit size/view may conceal this failure.
    assert!(measure(
        &theme,
        " ",
        TextAppearance::from(&style).layout(Some(&request)),
        7 * 1024,
        PhysicalTextAlignment::Start,
        "selected blank",
        &FitScope::root()
    )
    .is_err());
    let small = style.font_size(0.5).unwrap();
    let small_request = request.fit(TextFit::shrink_to(0.125).unwrap());
    let FitOutcome::Measured(small_plan) = selected(&theme, "AA", &small, &small_request, 1024)
    else {
        panic!("fractional fit")
    };
    let lower = minimum_height(
        &theme,
        "AA",
        TextAppearance::from(&small).layout(Some(&small_request)),
    )
    .unwrap()
    .unwrap();
    assert!(lower > 0 && lower <= small_plan.height_quanta);
}

#[test]
fn hyphen_and_reached_marker_do_not_inherit_an_unproven_source_floor() {
    use crate::font::canonical::{named_stack, PreparedFontEnvironment};
    use crate::layout::text::metric_tests::font;
    let appearance = crate::Theme::publication().resolve().unwrap();
    let text = crate::text::TextService::default();
    let environment = PreparedFontEnvironment::synthetic_shaping_fonts(vec![
        font("Used", &['A'], Some((800, 0)), (900, -300)),
        font("Hyphen", &['-'], Some((0, 0)), (0, 0)),
    ]);
    text.fonts.set(Ok(environment.clone())).unwrap();
    let theme = crate::text::TextRequestContext::new(&appearance, &text);
    let style = theme
        .typography
        .root
        .caption
        .clone()
        .stack(named_stack(&["Used", "Hyphen"]))
        .unwrap()
        .font_size(10.0)
        .unwrap();
    let request = TextLayout::default().fit(TextFit::shrink_to(10.0).unwrap());
    let FitOutcome::Measured(plan) = selected(&theme, "A\u{ad}AA", &style, &request, 12 * 1024)
    else {
        panic!("hyphen fits")
    };
    assert!(plan.lines[0].selected_soft_hyphen.is_some());
    assert_eq!(
        minimum_height(
            &theme,
            "A\u{ad}AA",
            TextAppearance::from(&style).layout(Some(&request))
        )
        .unwrap(),
        Some(0)
    );
    assert!(plan.lines[0]
        .runs
        .iter()
        .any(|run| run.metric_source == crate::font::canonical::MetricSource::InkOnly));

    let text = crate::text::TextService::default();
    let environment = PreparedFontEnvironment::synthetic_shaping_fonts(vec![
        font("Tall", &['A'], Some((3000, -1000)), (900, -300)),
        font("Marker", &['\u{2026}'], Some((0, 0)), (0, 0)),
    ]);
    text.fonts.set(Ok(environment.clone())).unwrap();
    let theme = crate::text::TextRequestContext::new(&appearance, &text);
    let style = style.stack(named_stack(&["Tall", "Marker"])).unwrap();
    let request = request
        .wrapping(TextWrapping::NoSoftWrap)
        .bounds(TextBounds::new(Some(6.0), Some(8.0), None).unwrap())
        .overflow(TextOverflow::ellipsis(
            crate::TextEllipsis::new(crate::TextElision::End),
            TextFailure::Error,
        ));
    environment
        .admit_text(style.get_stack(), style.get_weight().into(), "AAAA", None)
        .unwrap();
    assert_eq!(
        minimum_height(
            &theme,
            "AAAA",
            TextAppearance::from(&style).layout(Some(&request))
        )
        .unwrap(),
        Some(0)
    );
    let pair = environment
        .prepared_pair(style.get_stack(), style.get_weight().into())
        .unwrap()
        .unwrap();
    assert!(
        pair.selection_for_unit("\u{2026}", '\u{2026}').is_err(),
        "unused marker must not be admitted for evidence"
    );
    let FitOutcome::Measured(marked) = selected(&theme, "AAAA", &style, &request, 6 * 1024) else {
        panic!("marker-only view fits below source nominal height")
    };
    assert_eq!(marked.source, "\u{2026}");
    assert_eq!(marked.original_source(), "AAAA");
    assert_eq!(marked.height_quanta, 7 * 1024);
}
