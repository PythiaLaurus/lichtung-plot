use super::*;
use crate::theme::{Color, LocalMetric, RelativeSpacing};
use crate::TextWeight;

fn text(identity: u8) -> TextTheme {
    TextTheme::new(
        crate::font::canonical::named_stack(&[&format!("Role {identity}")]),
        Color::rgb(identity, identity + 1, identity + 2),
        u32::from(identity),
        TextWeight::Semibold,
        1.75,
    )
    .unwrap()
}

fn complete() -> TypographyTheme {
    TypographyTheme::new(
        RootTextRoles::new(text(1), text(2), text(3)).unwrap(),
        AxisTextRoles::new(text(4), text(5)).unwrap(),
        GuideTextRoles::new(text(13), text(14)).unwrap(),
        CompositionTextRoles::new(text(15), text(16), text(17), text(18)).unwrap(),
        AnnotationTextRoles::new(
            text(6),
            text(7),
            text(8),
            text(9),
            text(10),
            ContrastTextRoles::new(text(11), text(12)).unwrap(),
        )
        .unwrap(),
    )
    .unwrap()
}

fn assert_one_change(source: &TypographyTheme, edited: &TypographyTheme, index: usize) {
    for (position, (before, after)) in source.roles().into_iter().zip(edited.roles()).enumerate() {
        if position == index {
            assert_eq!(after, &text(91));
            assert_ne!(before, after);
        } else {
            assert_eq!(before, after);
            assert_eq!(
                before.stack().families().as_ptr(),
                after.stack().families().as_ptr()
            );
        }
    }
}

#[test]
fn complete_construction_retains_all_eighteen_inputs_once_without_font_discovery() {
    let before = crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get);
    let source = complete();
    for (index, role) in (1_u8..=18).zip(source.roles()) {
        assert_eq!(role, &text(index));
    }
    let cloned = source.clone();
    assert_eq!(cloned, source);
    for (before, after) in source.roles().into_iter().zip(cloned.roles()) {
        assert_eq!(
            before.stack().families().as_ptr(),
            after.stack().families().as_ptr()
        );
    }
    assert_eq!(
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        before
    );
}

#[test]
fn every_text_role_edit_is_independent_and_repeated_edits_replace_the_same_field() {
    let source = complete();
    macro_rules! edit {
        ($($owner:ident).+; $field:ident => $method:ident; $index:expr) => {
            let group = source.$($owner).+.clone().$method(text(90)).unwrap().$method(text(91)).unwrap();
            assert_eq!(group.$field(), &text(91));
            let mut edited = source.clone();
            edited.$($owner).+ = group;
            assert_one_change(&source, &edited, $index);
        };
    }
    edit!(root; title => with_title; 0);
    edit!(root; subtitle => with_subtitle; 1);
    edit!(root; caption => with_caption; 2);
    edit!(axes; axis_title => with_axis_title; 3);
    edit!(axes; tick_label => with_tick_label; 4);
    edit!(annotations; significance_label => with_significance_label; 5);
    edit!(annotations; estimate_detail => with_estimate_detail; 6);
    edit!(annotations; waterfall_total => with_waterfall_total; 7);
    edit!(annotations; point_layer_label => with_point_layer_label; 8);
    edit!(annotations; panel_text => with_panel_text; 9);
    edit!(annotations.contrast; light => with_light; 10);
    edit!(annotations.contrast; dark => with_dark; 11);
    edit!(guides; legend_title => with_legend_title; 12);
    edit!(guides; legend_label => with_legend_label; 13);
    edit!(composition; facet_header => with_facet_header; 14);
    edit!(composition; composition_header => with_composition_header; 15);
    edit!(composition; text_panel_title => with_text_panel_title; 16);
    edit!(composition; body => with_body; 17);
    assert_eq!(source, complete());
}

#[test]
fn complete_group_replacements_retain_siblings_and_do_not_couple_equal_initial_roles() {
    let source = complete();
    let root = source.root().clone().with_title(text(91)).unwrap();
    assert_one_change(&source, &source.clone().with_root(root).unwrap(), 0);
    let axes = source.axes().clone().with_tick_label(text(91)).unwrap();
    assert_one_change(&source, &source.clone().with_axes(axes).unwrap(), 4);
    let guides = source.guides().clone().with_legend_label(text(91)).unwrap();
    assert_one_change(&source, &source.clone().with_guides(guides).unwrap(), 13);
    let composition = source.composition().clone().with_body(text(91)).unwrap();
    assert_one_change(
        &source,
        &source.clone().with_composition(composition).unwrap(),
        17,
    );
    let contrast = source
        .annotations()
        .contrast()
        .clone()
        .with_dark(text(91))
        .unwrap();
    let annotations = source
        .annotations()
        .clone()
        .with_contrast(contrast)
        .unwrap();
    assert_one_change(
        &source,
        &source.clone().with_annotations(annotations).unwrap(),
        11,
    );

    let preset = crate::Theme::Publication
        .resolve(&OperationGenerationResources::new())
        .unwrap();
    assert_eq!(
        preset.typography.annotations().estimate_detail(),
        preset.typography.annotations().waterfall_total()
    );
    let changed = preset
        .typography
        .annotations()
        .clone()
        .with_estimate_detail(text(91))
        .unwrap();
    assert_eq!(
        changed.waterfall_total(),
        preset.typography.annotations().waterfall_total()
    );
    assert_eq!(
        changed.panel_text(),
        preset.typography.annotations().panel_text()
    );
}

#[test]
fn constructors_and_edits_reject_invalid_siblings_through_every_nested_group() {
    let mut source = complete();
    source.root.subtitle.font_size = 0;
    assert!(RootTextRoles::new(text(1), source.root.subtitle.clone(), text(3)).is_err());
    assert!(source.root.clone().with_title(text(91)).is_err());
    source = complete();
    source.axes.tick_label.line_height_multiplier = f64::NAN;
    assert!(AxisTextRoles::new(text(4), source.axes.tick_label.clone()).is_err());
    assert!(source.axes.clone().with_axis_title(text(91)).is_err());
    source = complete();
    source.guides.legend_label.font_size = 0;
    assert!(GuideTextRoles::new(text(13), source.guides.legend_label.clone()).is_err());
    assert!(source.guides.clone().with_legend_title(text(91)).is_err());
    source = complete();
    source.composition.body.line_height_multiplier = f64::INFINITY;
    assert!(CompositionTextRoles::new(
        text(15),
        text(16),
        text(17),
        source.composition.body.clone()
    )
    .is_err());
    assert!(source
        .composition
        .clone()
        .with_facet_header(text(91))
        .is_err());
    assert!(source.clone().with_root(complete().root).is_err());
    assert!(source.clone().with_axes(complete().axes).is_err());
    assert!(source.clone().with_guides(complete().guides).is_err());
    assert!(source
        .clone()
        .with_annotations(complete().annotations)
        .is_err());
    assert!(TypographyTheme::new(
        source.root,
        source.axes,
        source.guides,
        source.composition,
        source.annotations
    )
    .is_err());
    source = complete();
    source.annotations.panel_text.font_size = 0;
    assert!(AnnotationTextRoles::new(
        source.annotations.significance_label.clone(),
        source.annotations.estimate_detail.clone(),
        source.annotations.waterfall_total.clone(),
        source.annotations.point_layer_label.clone(),
        source.annotations.panel_text.clone(),
        source.annotations.contrast.clone(),
    )
    .is_err());
    assert!(source
        .annotations
        .clone()
        .with_estimate_detail(text(91))
        .is_err());
    assert!(source
        .annotations
        .clone()
        .with_contrast(complete().annotations.contrast)
        .is_err());
    assert!(source.with_composition(complete().composition).is_err());
}

#[test]
fn contrast_construction_and_edits_require_both_complete_candidates_to_be_opaque() {
    let source = complete();
    for alpha in [0, 1, 254] {
        let transparent = text(91).with_color(Color::rgba(3, 5, 7, alpha)).unwrap();
        let contrast = source.annotations().contrast();
        assert!(ContrastTextRoles::new(transparent.clone(), contrast.dark().clone()).is_err());
        assert!(ContrastTextRoles::new(contrast.light().clone(), transparent.clone()).is_err());
        assert!(contrast.clone().with_light(transparent.clone()).is_err());
        assert!(contrast.clone().with_dark(transparent.clone()).is_err());
        let mut poisoned = contrast.clone();
        poisoned.light = transparent;
        assert!(poisoned.clone().with_dark(text(91)).is_err());
        let mut poisoned_theme = source.clone();
        poisoned_theme.annotations.contrast = poisoned.clone();
        assert!(AnnotationTextRoles::new(
            text(6),
            text(7),
            text(8),
            text(9),
            text(10),
            poisoned.clone(),
        )
        .is_err());
        assert!(source
            .clone()
            .with_annotations(poisoned_theme.annotations.clone())
            .is_err());
        assert!(poisoned_theme
            .clone()
            .with_root(source.root().clone())
            .is_err());
        assert!(TypographyTheme::new(
            poisoned_theme.root,
            poisoned_theme.axes,
            poisoned_theme.guides,
            poisoned_theme.composition,
            poisoned_theme.annotations,
        )
        .is_err());
        assert!(source
            .annotations()
            .clone()
            .with_contrast(poisoned)
            .is_err());
    }
    assert_eq!(source, complete());
}

#[test]
fn retained_theme_matching_visits_each_nested_role_once_and_refuses_before_skipping_work() {
    let source = crate::Theme::Publication
        .resolve(&OperationGenerationResources::new())
        .unwrap();
    macro_rules! mismatch {
        ($($path:ident).+; $index:expr) => {
            let mut retained = source.clone();
            retained.typography.$($path).+ = retained.typography.$($path).+.clone()
                .with_line_height(1.75).unwrap();
            assert_ne!(retained, source);
            // Four scale/accent, seven axis and two point-ratio witnesses,
            // then one font witness per unchanged
            // preceding role. The differing scalar refuses equality before
            // examining that role's font payload.
            let work = 45 + $index;
            for remaining in [work - 1, work] {
                let resources = OperationGenerationResources::with_test_limits(remaining, 0);
                let result = retained.same(&source, &resources);
                if remaining == work {
                    assert!(!result.unwrap());
                } else {
                    assert!(result.is_err());
                }
                assert_eq!(resources.usage().unwrap(), (0, remaining));
                assert_eq!(resources.byte_usage().unwrap(), 0);
            }
        };
    }
    mismatch!(root.title; 0);
    mismatch!(root.subtitle; 1);
    mismatch!(root.caption; 2);
    mismatch!(axes.axis_title; 3);
    mismatch!(axes.tick_label; 4);
    mismatch!(annotations.significance_label; 5);
    mismatch!(annotations.estimate_detail; 6);
    mismatch!(annotations.waterfall_total; 7);
    mismatch!(annotations.point_layer_label; 8);
    mismatch!(annotations.panel_text; 9);
    mismatch!(annotations.contrast.light; 10);
    mismatch!(annotations.contrast.dark; 11);
    mismatch!(guides.legend_title; 12);
    mismatch!(guides.legend_label; 13);
    mismatch!(composition.facet_header; 14);
    mismatch!(composition.composition_header; 15);
    mismatch!(composition.text_panel_title; 16);
    mismatch!(composition.body; 17);
}

#[test]
fn publication_title_keeps_the_exact_binary64_leading_and_upward_occupancy() {
    let resources = OperationGenerationResources::new();
    let theme = crate::Theme::Publication.resolve(&resources).unwrap();
    let title = theme.typography.root().title();
    assert_eq!(title.line_height().to_bits(), 0x3ff4_cccc_cccc_cccd);
    let exact = RelativeSpacing::new_in(title.line_height(), &resources)
        .unwrap()
        .product(
            LocalMetric::css_pixels(title.font_size()).unwrap(),
            &resources,
        )
        .unwrap();
    let occupancy = exact
        .round(crate::layout::kernel::SpatialRounding::Ceil, &resources)
        .unwrap();
    assert_eq!(occupancy.quanta(), 26_625);
}
