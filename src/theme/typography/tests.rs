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
        }
    }
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
fn publication_title_keeps_the_exact_binary64_leading_and_upward_occupancy() {
    let theme = crate::Theme::Publication.resolve().unwrap();
    let title = theme.typography.root().title();
    assert_eq!(title.line_height().to_bits(), 0x3ff4_cccc_cccc_cccd);
    let exact = RelativeSpacing::new(title.line_height())
        .unwrap()
        .product(LocalMetric::css_pixels(title.font_size()).unwrap());
    let occupancy = exact
        .round(crate::layout::kernel::SpatialRounding::Ceil)
        .unwrap();
    assert_eq!(occupancy.quanta(), 26_625);
}
