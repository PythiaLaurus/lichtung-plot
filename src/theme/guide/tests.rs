use super::*;
use crate::layout::kernel::SpatialRounding;

fn source() -> LegendTheme {
    LegendTheme::builtin().unwrap()
}

#[test]
fn complete_guide_fields_and_consuming_edits_are_independent() {
    let source = source();
    let spacing = source.spacing();
    let replacement = RelativeSpacing::ratio(3, 2).unwrap();
    macro_rules! edit {
        ($field:ident, $method:ident, $index:expr) => {
            let changed = spacing
                .clone()
                .$method(RelativeSpacing::ratio(7, 3).unwrap())
                .unwrap()
                .$method(replacement.clone())
                .unwrap();
            assert_eq!(changed.$field(), &replacement);
            for (index, (left, right)) in spacing
                .relationships()
                .into_iter()
                .zip(changed.relationships())
                .enumerate()
            {
                if index != $index {
                    assert_eq!(left, right);
                }
            }
        };
    }
    edit!(padding, with_padding, 0);
    edit!(key_width, with_key_width, 1);
    edit!(key_label_gap, with_key_label_gap, 2);
    edit!(outside_gap, with_outside_gap, 3);
    edit!(shared_gap, with_shared_gap, 4);
    edit!(row_gap, with_row_gap, 5);
    edit!(column_gap, with_column_gap, 6);
    edit!(title_gap, with_title_gap, 7);
    edit!(glyph_edge_padding, with_glyph_edge_padding, 8);
    let changed = source.clone().with_shared_gap(replacement.clone()).unwrap();
    assert_eq!(changed.shared_gap(), &replacement);
    assert_eq!(changed.gradient_lengths(), source.gradient_lengths());
    assert_eq!(changed.frame(), source.frame());
    let changed = source
        .clone()
        .with_spacing(spacing.clone().with_padding(replacement.clone()).unwrap())
        .unwrap();
    assert_eq!(changed.spacing().padding(), &replacement);
    let minimum = RelativeSpacing::ratio(1, 1).unwrap();
    let preferred = RelativeSpacing::ratio(2, 1).unwrap();
    let lengths = GradientLengths::new(minimum.clone(), preferred.clone()).unwrap();
    assert_eq!(
        lengths
            .clone()
            .with_minimum(preferred.clone())
            .unwrap()
            .minimum(),
        &preferred
    );
    assert_eq!(
        lengths
            .clone()
            .with_preferred(minimum.clone())
            .unwrap()
            .preferred(),
        &minimum
    );
    assert_eq!(
        source
            .clone()
            .with_gradient_lengths(lengths.clone())
            .unwrap()
            .gradient_lengths(),
        &lengths
    );
    let frame = source.frame().unwrap();
    let fill = Color::rgba(1, 2, 3, 0);
    let changed = frame.clone().with_fill(fill).unwrap();
    assert_eq!(changed.fill(), fill);
    assert_eq!(changed.outline(), frame.outline());
    assert_eq!(frame.clone().with_outline(None).unwrap().outline(), None);
    assert_eq!(
        frame
            .clone()
            .with_radius(replacement.clone())
            .unwrap()
            .radius(),
        &replacement
    );
    let absent = source.clone().with_frame(None).unwrap();
    assert!(absent.frame().is_none());
    assert_eq!(absent.spacing(), source.spacing());
}

#[test]
fn pinned_binary_sources_and_complete_edge_rounding_are_exact() {
    let source = source();

    let lengths = source.lengths(11, 12).unwrap();
    let edge = lengths.padding.add_length(&lengths.glyph_edge_padding);
    assert_eq!(
        edge.round(SpatialRounding::NearestEven).unwrap().quanta(),
        11264
    );
    assert_eq!(edge.round(SpatialRounding::Ceil).unwrap().quanta(), 11265);
    assert_eq!(
        edge.times(2).round(SpatialRounding::Ceil).unwrap().quanta(),
        22529
    );
    assert_ne!(
        source.spacing().padding(),
        &RelativeSpacing::ratio(4, 5).unwrap()
    );
    assert_ne!(
        source.spacing().glyph_edge_padding(),
        &RelativeSpacing::ratio(1, 5).unwrap()
    );
    let exact = GradientLengths::new(
        RelativeSpacing::ratio(6, 10).unwrap(),
        RelativeSpacing::ratio(3, 5).unwrap(),
    )
    .unwrap();
    assert_eq!(exact.minimum(), exact.preferred());
}
