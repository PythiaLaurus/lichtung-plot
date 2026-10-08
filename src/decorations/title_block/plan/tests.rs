#![allow(clippy::float_cmp, reason = "frozen geometry is compared exactly")]

use super::*;
use crate::theme::TitleAlignment::{Center, End, Start};

const PRESETS: [crate::Theme; 4] = [
    crate::Theme::Publication,
    crate::Theme::Nature,
    crate::Theme::Science,
    crate::Theme::CHI,
];

fn no_padding() -> crate::theme::composition::RootPaddingLengths {
    crate::theme::composition::RootPaddingLengths::zero().unwrap()
}

fn heights(plan: &RootTitleBlockPlan) -> (f64, f64) {
    (
        crate::theme::composition::hard(&plan.reserves.top).unwrap(),
        crate::theme::composition::hard(&plan.reserves.bottom).unwrap(),
    )
}

fn measure(
    theme: &ResolvedTheme,
    sources: [Option<&str>; 3],
    align: RootAlignments,
) -> RootTitleBlockPlan {
    RootTitleBlockPlan::measure(
        theme,
        sources,
        PanelBox::new(16.0, 317.0, 8.0, 468.0),
        &no_padding(),
        301.0,
        align,
        60.0,
        &FitScope::root(),
    )
    .unwrap()
}

#[test]
fn root_alignment_precedence_preserves_independent_optional_instructions() {
    for preset in PRESETS {
        let mut theme = preset.resolve().unwrap();
        let defaults = RootAlignments::resolve(&theme, [None; 2], [None; 2]);
        assert_eq!(
            defaults,
            RootAlignments {
                title: Center,
                subtitle: Center,
                caption: Start
            }
        );
        assert_eq!(theme.composition.root_subtitle_alignment(), None);
        let panel = [Some(Start), None];
        assert_eq!(
            RootAlignments::resolve(&theme, [None; 2], panel).title,
            Start
        );
        let plot = [Some(End), None];
        assert_eq!(RootAlignments::resolve(&theme, plot, panel).subtitle, End);
        let panel = [Some(Start), Some(Center)];
        assert_eq!(
            RootAlignments::resolve(&theme, plot, panel).subtitle,
            Center
        );
        assert_eq!(
            RootAlignments::resolve(&theme, [Some(End), Some(Start)], panel).subtitle,
            Start
        );
        theme.composition = theme
            .composition
            .clone()
            .with_root_subtitle_alignment(Some(End))
            .unwrap();
        assert_eq!(
            RootAlignments::resolve(&theme, [Some(Start), None], [None; 2]).subtitle,
            End
        );
        assert_eq!(
            RootAlignments::resolve(&theme, plot, panel).subtitle,
            Center
        );
        theme.composition = theme
            .composition
            .clone()
            .with_root_caption_alignment(End)
            .unwrap();
        assert_eq!(
            RootAlignments::resolve(&theme, plot, panel),
            RootAlignments {
                title: End,
                subtitle: Center,
                caption: End
            }
        );
        // A Canvas root consumes only its own theme, never its panel instructions.
        assert_eq!(
            RootAlignments::resolve(&theme, [None; 2], [None; 2]).title,
            Center
        );
    }
}

#[test]
#[expect(
    clippy::cast_precision_loss,
    reason = "fixture baselines are bounded by the small assigned root"
)]
fn fractional_title_gap_and_center_use_final_parent_parity_without_rounding_offsets_separately() {
    let mut theme = crate::Theme::Publication.resolve().unwrap();
    let spacing = theme
        .composition
        .root_title_block()
        .clone()
        .with_title_subtitle_gap(crate::theme::RelativeSpacing::ratio(1, 20 * 2048).unwrap())
        .unwrap();
    theme.composition = theme
        .composition
        .clone()
        .with_root_title_block(spacing)
        .unwrap();
    for parity in [0, 1] {
        let top = 468.0 + f64::from(parity) / 1024.0;
        let span = PanelBox::new(1.0 / 1024.0, 301.0 + 1.0 / 1024.0, 0.0, top);
        let plan = RootTitleBlockPlan::measure(
            &theme,
            [Some("Title"), Some("Subtitle"), None],
            span,
            &no_padding(),
            301.0,
            RootAlignments::resolve(&theme, [None; 2], [None; 2]),
            60.0,
            &FitScope::root(),
        )
        .unwrap();
        let title = plan.roles[0].as_ref().unwrap();
        let subtitle = plan.roles[1].as_ref().unwrap();
        let expected_y = (top * 1024.0
            - title.text.height() * 1024.0
            - 0.5
            - subtitle.text.first_baseline_quanta.unwrap() as f64)
            .round_ties_even()
            / 1024.0;
        assert_eq!(subtitle.placement.position.y, expected_y);
        let expected_left = (span.x0 * 1024.0 + (span.width() - subtitle.text.width()) * 512.0)
            .round_ties_even()
            / 1024.0;
        assert_eq!(subtitle.bounds.x0, expected_left);
        assert_eq!(title.text.source, "Title");
        assert_eq!(subtitle.text.source, "Subtitle");
    }
}

#[test]
fn private_plot_inputs_survive_erasure_theme_order_and_replacement() {
    let make = || {
        crate::Plot::scatter(
            crate::NumericValues::new("x", [1.0]),
            crate::NumericValues::new("y", [2.0]),
        )
    };
    let mut before = make();
    before.title_alignment = Some(Start);
    before.subtitle_alignment = Some(Center);
    before.title_alignment = Some(End);
    let before = before.theme(crate::Theme::Nature).erase();
    let mut after = make().theme(crate::Theme::Nature);
    after.title_alignment = Some(End);
    after.subtitle_alignment = Some(Center);
    let after = after.erase();
    assert_eq!(before.title_alignment, after.title_alignment);
    assert_eq!(before.subtitle_alignment, after.subtitle_alignment);
    let mut canvas = crate::Canvas::grid(1, 1).unwrap();
    assert_eq!(canvas.panel_title_alignment, None);
    assert_eq!(canvas.panel_subtitle_alignment, None);
    canvas.panel_title_alignment = Some(Start);
    canvas.panel_subtitle_alignment = Some(End);
    let effective = RootAlignments::resolve(
        &crate::Theme::Nature.resolve().unwrap(),
        [before.title_alignment, before.subtitle_alignment],
        [
            canvas.panel_title_alignment,
            canvas.panel_subtitle_alignment,
        ],
    );
    assert_eq!((effective.title, effective.subtitle), (End, Center));
}

#[test]
fn root_role_presence_keeps_blank_lines_whitespace_and_exact_baselines() {
    for preset in PRESETS {
        let theme = preset.resolve().unwrap();
        for mask in 0..8 {
            let sources = [Some("Title\n\n"), Some(" \nSubtitle"), Some("Caption\n ")];
            let sources = std::array::from_fn(|index| {
                if mask & (1 << index) != 0 {
                    sources[index]
                } else {
                    None
                }
            });
            let plan = measure(
                &theme,
                sources,
                RootAlignments::resolve(&theme, [None; 2], [None; 2]),
            );
            for (index, role) in plan.roles.iter().enumerate() {
                assert_eq!(role.is_some(), sources[index].is_some());
                if let Some(role) = role {
                    assert_eq!(role.text.source, sources[index].unwrap());
                    assert_eq!(role.text.lines.len(), if index == 0 { 3 } else { 2 });
                    crate::render::backend::validate_planned_text(role.placement, &role.text)
                        .unwrap();
                    assert_eq!(
                        role.text.style.font_size,
                        [
                            &theme.typography.root.title,
                            &theme.typography.root.subtitle,
                            &theme.typography.root.caption
                        ][index]
                            .font_size
                    );
                    assert_eq!(
                        role.text.style.paint,
                        [
                            &theme.typography.root.title,
                            &theme.typography.root.subtitle,
                            &theme.typography.root.caption
                        ][index]
                            .paint
                    );
                }
            }
            assert_eq!(
                heights(&plan).0 == 0.0,
                sources[0].is_none() && sources[1].is_none()
            );
            assert_eq!(heights(&plan).1 == 0.0, mask & 4 == 0);
        }
        let empty = measure(
            &theme,
            [Some(""); 3],
            RootAlignments::resolve(&theme, [None; 2], [None; 2]),
        );
        assert!(empty.roles.iter().all(Option::is_none));
        let whitespace = measure(
            &theme,
            [Some(" "); 3],
            RootAlignments::resolve(&theme, [None; 2], [None; 2]),
        );
        assert!(whitespace.roles.iter().all(Option::is_some));
    }
}

#[test]
fn independent_physical_alignment_does_not_change_wrapping_or_fit() {
    let theme = crate::Theme::Publication.resolve().unwrap();
    for source in [
        "Unequal lines\nx",
        "Deterministic wrapping keeps every word in this deliberately long root heading without changing its alignment-independent fit",
        "\u{5e9}\u{5dc}\u{5d5}\u{5dd}\nx",
        "Latin \u{5e9}\u{5dc}\u{5d5}\u{5dd}\nsmall",
    ] {
        let sources = [Some(source); 3];
        let widths = RootTitleBlockPlan::intrinsic_widths(&theme, sources, None).unwrap().widths();
        assert!(widths.1 >= widths.0 && widths.0 > 0.0);
        let base = measure(
            &theme,
            sources,
            RootAlignments {
                title: Start,
                subtitle: Start,
                caption: Start,
            },
        );
        for title in [Start, Center, End] {
            for subtitle in [Start, Center, End] {
                for caption in [Start, Center, End] {
                    let plan = measure(
                        &theme,
                        sources,
                        RootAlignments {
                            title,
                            subtitle,
                            caption,
                        },
                    );
                    assert_eq!(
                        heights(&plan),
                        heights(&base)
                    );
                    for (index, alignment) in [title, subtitle, caption].into_iter().enumerate() {
                        let role = plan.roles[index].as_ref().unwrap();
                        let original = &base.roles[index].as_ref().unwrap().text;
                        assert_eq!(role.text.lines.len(), original.lines.len());
                        assert_eq!(role.text.width_quanta, original.width_quanta);
                        assert_eq!(role.placement.rotation, 0.0);
                        match alignment {
                            Start => assert_eq!(role.bounds.x0, 16.0),
                            Center => assert!(
                                (f64::midpoint(role.bounds.x0, role.bounds.x1) - 166.5).abs()
                                    <= 1.0 / 2048.0
                            ),
                            End => assert_eq!(role.bounds.x1, 317.0),
                        }
                        let short = role.text.lines.last().unwrap();
                        let (left, width) = short
                            .ink_quanta
                            .map_or((0, 0), |ink| (ink.left, ink.right - ink.left));
                        let residual = role.text.width_quanta - width;
                        let offset = short.translation_x_quanta + left;
                        match alignment {
                            Start => assert_eq!(offset, 0),
                            Center => {
                                assert!((2 * offset - residual).abs() <= 1);
                            }
                            End => assert_eq!(offset, residual),
                        }
                        crate::render::backend::validate_planned_text(role.placement, &role.text)
                            .unwrap();
                    }
                }
            }
        }
    }
}

#[test]
fn root_intrinsic_width_miss_identifies_the_subtitle() {
    let theme = crate::Theme::Publication.resolve().unwrap();
    let sources = [None, Some("Unbreakable"), None];
    let align = RootAlignments::resolve(&theme, [None; 2], [None; 2]);
    let (hard, _) = RootTitleBlockPlan::intrinsic_widths(&theme, sources, None)
        .unwrap()
        .widths();
    let failure = RootTitleBlockPlan::measure(
        &theme,
        sources,
        PanelBox::new(0.0, hard - 1.0 / 1024.0, 0.0, 480.0),
        &no_padding(),
        hard - 1.0 / 1024.0,
        align,
        60.0,
        &FitScope::root(),
    )
    .unwrap_err();
    let PlanError::Fit(failure) = failure else {
        panic!("expected a typed width miss");
    };
    assert_eq!(
        failure.reason.address,
        FitScope::root()
            .child(AddressRole::RootSubtitle, 0)
            .unwrap()
            .address
    );
}

#[test]
fn intrinsic_summary_and_constrained_title_share_glyph_analysis() {
    let theme = crate::Theme::Publication.resolve().unwrap();
    let sources = [
        Some("Retained glyph analysis"),
        Some("Subtitle"),
        Some("Caption"),
    ];
    let intrinsics = RootTitleBlockPlan::intrinsic_widths(&theme, sources, None).unwrap();
    let shapes = crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get);

    let alignments = RootAlignments::resolve(&theme, [None; 2], [None; 2]);
    for width in [640.0, 800.0] {
        intrinsics.check_width(width, &FitScope::root()).unwrap();
        let plan = RootTitleBlockPlan::measure(
            &theme,
            sources,
            PanelBox::new(0.0, width, 0.0, 480.0),
            &no_padding(),
            width,
            alignments,
            60.0,
            &FitScope::root(),
        )
        .unwrap();
        assert!(plan
            .roles
            .iter()
            .flatten()
            .all(|role| role.text.lines.len() == 1));
        assert_eq!(
            shapes,
            crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get)
        );
    }

    assert_eq!(
        shapes,
        crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get)
    );
}

#[test]
fn root_exact_height_refusal_is_typed_and_source_checks_remain_strict() {
    let theme = crate::Theme::Publication.resolve().unwrap();
    let align = RootAlignments::resolve(&theme, [Some(End), None], [None; 2]);
    let sources = [None, Some("Subtitle only\nsecond line"), Some("Caption")];
    let plan = measure(&theme, sources, align);
    assert_eq!(plan.alignments.subtitle, End);
    let height = heights(&plan).0 + heights(&plan).1 + 60.0;
    RootTitleBlockPlan::measure(
        &theme,
        sources,
        PanelBox::new(16.0, 317.0, 0.0, height),
        &no_padding(),
        301.0,
        align,
        60.0,
        &FitScope::root(),
    )
    .unwrap();
    let error = RootTitleBlockPlan::measure(
        &theme,
        sources,
        PanelBox::new(16.0, 317.0, 0.0, height - 1.0 / 1024.0),
        &no_padding(),
        301.0,
        align,
        60.0,
        &FitScope::root(),
    )
    .unwrap_err();
    assert!(matches!(error, PlanError::Fit(_)));
    let mut backend = crate::render::geometry::GeometryBackend::new(&theme).unwrap();
    backend.begin(333, 480).unwrap();
    plan.draw(sources, &mut backend).unwrap();
    assert!(plan.draw([None; 3], &mut backend).is_err());
    assert!(plan
        .draw([Some("unexpected"), sources[1], sources[2]], &mut backend)
        .is_err());
    assert!(plan
        .draw([None, Some("different"), sources[2]], &mut backend)
        .is_err());
    backend.end().unwrap();
    let mut malformed = plan.clone();
    let role = malformed.roles[1].as_mut().unwrap();
    role.text.edit_for_test().lines.pop();
    assert!(crate::render::backend::validate_planned_text(role.placement, &role.text).is_err());
    let mut malformed = plan.clone();
    let role = malformed.roles[1].as_mut().unwrap();
    role.text.edit_for_test().lines[1].translation_y_quanta += 1;
    assert!(crate::render::backend::validate_planned_text(role.placement, &role.text).is_err());
    let absent_title_start = measure(
        &theme,
        [None; 3],
        RootAlignments {
            title: Start,
            subtitle: End,
            caption: Start,
        },
    );
    let absent_title_end = measure(
        &theme,
        [None; 3],
        RootAlignments {
            title: End,
            subtitle: End,
            caption: Start,
        },
    );
    assert_ne!(absent_title_start, absent_title_end);
}
