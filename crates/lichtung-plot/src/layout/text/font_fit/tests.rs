use super::*;
use crate::{TextBounds, TextFit, TextLayout, TextWrapping};

fn selected(
    theme: &ResolvedTheme,
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
fn fractional_floor_and_equal_ranges_retain_preferred_appearance_separately() {
    assert!(TextFit::shrink_to(0.0).is_err());
    assert!(TextFit::range(14.0, 10.0).is_err());
    assert!(TextFit::range(10.0, f64::INFINITY).is_err());
    let theme = crate::Theme::publication().resolve().unwrap();
    let appearance = theme
        .typography
        .root
        .caption
        .clone()
        .font_size(14.0)
        .unwrap();
    let request = TextLayout::default().fit(TextFit::range(10.25, 10.25).unwrap());
    let FitOutcome::Measured(plan) = measure(
        &theme,
        "Complete title",
        TextAppearance::from(&appearance).layout(Some(&request)),
        140 * 1024,
        PhysicalTextAlignment::Start,
        "fit witness",
        &FitScope::root(),
    )
    .unwrap() else {
        panic!("equal-size fit");
    };
    assert_eq!(plan.style.font_size.to_bits(), 10.25_f64.to_bits());
    assert_eq!(
        plan.fit_input
            .as_ref()
            .unwrap()
            .appearance
            .get_font_size()
            .to_bits(),
        14.0_f64.to_bits()
    );
    assert_eq!(plan.source, "Complete title");
    let request = TextLayout::default()
        .fit(TextFit::shrink_to(10.25).unwrap())
        .wrapping(TextWrapping::NoSoftWrap)
        .bounds(TextBounds::new(Some(100.0), None, None).unwrap());
    let FitOutcome::Measured(plan) = measure(
        &theme,
        "Complete title",
        TextAppearance::from(&appearance).layout(Some(&request)),
        140 * 1024,
        PhysicalTextAlignment::Start,
        "fit witness",
        &FitScope::root(),
    )
    .unwrap() else {
        panic!("floor fit");
    };
    assert!((10.25..=14.0).contains(&plan.style.font_size));
    assert!(plan.width() <= 100.0);
}

#[test]
fn metric_proposals_find_fractional_sizes_and_large_intervals_do_not_sample_integer_sizes() {
    let theme = crate::Theme::publication().resolve().unwrap();
    let style = theme
        .typography
        .root
        .caption
        .clone()
        .font_size(14.0)
        .unwrap();
    let request = TextLayout::default()
        .wrapping(TextWrapping::NoSoftWrap)
        .fit(TextFit::range(10.0, 1_000_000.0).unwrap());
    let FitOutcome::Measured(plan) = selected(
        &theme,
        "Complete scientific title",
        &style,
        &request,
        125 * 1024,
    ) else {
        panic!("fractional metric proposal");
    };
    assert!((10.0..14.0).contains(&plan.style.font_size));
    assert_ne!(plan.style.font_size.fract().to_bits(), 0.0_f64.to_bits());
    assert!(plan.width() <= 125.0);
    assert!(theme.text_query_count()<=2,"only the source basis and an optional selected result may be retained, never failed size scenes");
    let fixed = style.clone().font_size(plan.style.font_size).unwrap();
    let expected = crate::layout::fit::measured(super::super::request::measure_unfitted(
        &theme,
        "Complete scientific title",
        TextAppearance::from(&fixed).layout(Some(
            &TextLayout::default().wrapping(TextWrapping::NoSoftWrap),
        )),
        125 * 1024,
        PhysicalTextAlignment::Start,
        "independent fixed witness",
        &FitScope::root(),
    ))
    .unwrap();
    assert_eq!(plan.lines, expected.lines);
    assert_eq!(plan.height_quanta, expected.height_quanta);
}

#[test]
fn priorities_keep_complete_content_or_preferred_size_and_floor_actions_remain_explicit() {
    use crate::{TextElision, TextEllipsis};
    let theme = crate::Theme::publication().resolve().unwrap();
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
fn uniform_roles_use_the_common_interval_without_coupling_independent_or_contrast_members() {
    let theme = crate::Theme::publication().resolve().unwrap();
    let style = theme
        .typography
        .root
        .caption
        .clone()
        .font_size(14.0)
        .unwrap();
    let request = TextLayout::default()
        .wrapping(TextWrapping::NoSoftWrap)
        .fit(
            TextFit::shrink_to(8.0)
                .unwrap()
                .grouping(crate::TextFitGrouping::Uniform),
        );
    let mut plans = Vec::new();
    for (source, style, layout) in [
        ("Short", style.clone(), request.clone()),
        ("Scientific complete title", style.clone(), request.clone()),
        (
            "Independent",
            style.clone(),
            request.clone().fit(TextFit::fixed()),
        ),
        (
            "Contrast",
            style
                .clone()
                .color(crate::Color::rgb(255, 255, 255))
                .unwrap(),
            request.clone(),
        ),
    ] {
        let FitOutcome::Measured(plan) = selected(&theme, source, &style, &layout, 125 * 1024)
        else {
            panic!("group member");
        };
        plans.push(plan);
    }
    assert_ne!(
        plans[0].style.font_size.to_bits(),
        plans[1].style.font_size.to_bits()
    );
    uniform(&theme, &mut plans, &FitScope::root()).unwrap();
    assert_eq!(
        plans[0].style.font_size.to_bits(),
        plans[1].style.font_size.to_bits()
    );
    assert_eq!(plans[2].style.font_size.to_bits(), 14.0_f64.to_bits());
    assert_eq!(plans[3].style.font_size.to_bits(), 14.0_f64.to_bits());
    assert_eq!(plans[0].fit_group, plans[1].fit_group);
    assert_ne!(plans[0].fit_group, plans[3].fit_group);
}

#[test]
fn exact_metric_cells_cover_rounding_boundaries_and_leave_trial_queries_uncached() {
    let theme = crate::Theme::publication().resolve().unwrap();
    let style = theme
        .typography
        .root
        .caption
        .clone()
        .font_size(12.25)
        .unwrap();
    let request = TextLayout::default()
        .wrapping(TextWrapping::Legal)
        .bounds(TextBounds::new(None, Some(100.0), Some(3)).unwrap());
    let trace = Arc::new(stability::Stability::new(12.25, 12.0));
    let mut trial = theme.clone();
    trial.text_size_trial = Some(Arc::clone(&trace));
    let source = "A contextual line with legal wrapping";
    let cache_before = theme.text_query_count();
    let FitOutcome::Measured(plan) = super::super::request::measure_unfitted(
        &trial,
        source,
        TextAppearance::from(&style).layout(Some(&request)),
        100 * 1024,
        PhysicalTextAlignment::Start,
        "metric cell",
        &FitScope::root(),
    )
    .unwrap() else {
        panic!("metric cell");
    };
    assert_eq!(theme.text_query_count(), cache_before);
    let lower = trace.lower().unwrap();
    assert!((12.0..=12.25).contains(&lower));
    for size in [lower, f64::midpoint(lower, 12.25), 12.25] {
        let style = style.clone().font_size(size).unwrap();
        let FitOutcome::Measured(other) = super::super::request::measure_unfitted(
            &theme,
            source,
            TextAppearance::from(&style).layout(Some(&request)),
            100 * 1024,
            PhysicalTextAlignment::Start,
            "independent cell replay",
            &FitScope::root(),
        )
        .unwrap() else {
            panic!("stable metric cell");
        };
        assert_eq!(plan.lines, other.lines);
        assert_eq!(plan.height_quanta, other.height_quanta);
    }
}

#[test]
fn quantized_ink_can_fit_between_two_nonfitting_endpoints_without_monotone_wrapping() {
    // The two independent rectangle edges are at s and 2s quanta. At s=1
    // outward width is 1; immediately either side it is 2. A one-quantum line
    // cell therefore changes from two wrapped lines to one and back to two.
    let minimum = 0.99_f64;
    let maximum = 1.01_f64;
    let width = |size: f64| {
        ExactRational::design(2, size, 1024)
            .unwrap()
            .ceil_i64("right")
            .unwrap()
            - ExactRational::design(1, size, 1024)
                .unwrap()
                .floor_i64("left")
                .unwrap()
    };
    assert_eq!(width(minimum), 2);
    assert_eq!(width(maximum), 2);
    assert_eq!(width(1.0), 1);
    let mut size = maximum;
    loop {
        let trace = Arc::new(stability::Stability::new(size, minimum));
        let right = ExactRational::tracked_design(2, size, 1024, &trace)
            .unwrap()
            .ceil_i64("right")
            .unwrap();
        let left = ExactRational::tracked_design(1, size, 1024, &trace)
            .unwrap()
            .floor_i64("left")
            .unwrap();
        if right - left <= 1 {
            assert_eq!(size.to_bits(), 1.0_f64.to_bits());
            break;
        }
        let lower = trace.lower().unwrap();
        assert!(lower > minimum);
        size = f64::from_bits(lower.to_bits() - 1);
    }
}
