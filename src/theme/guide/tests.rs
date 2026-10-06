use super::*;
use crate::layout::kernel::SpatialRounding;

fn source() -> LegendTheme {
    LegendTheme::builtin(&OperationGenerationResources::new()).unwrap()
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
                    assert!(left.shares_payload_with(right));
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
fn construction_rejects_zero_positive_extents_and_revalidates_cross_siblings() {
    let source = source();
    let zero = RelativeSpacing::new(-0.0).unwrap();
    assert!(source
        .spacing()
        .clone()
        .with_key_width(zero.clone())
        .is_err());
    assert!(GradientLengths::new(zero.clone(), RelativeSpacing::ratio(1, 1).unwrap()).is_err());
    assert!(GradientLengths::new(RelativeSpacing::ratio(1, 1).unwrap(), zero.clone()).is_err());
    assert!(source
        .gradient_lengths()
        .clone()
        .with_minimum(RelativeSpacing::ratio(1000, 1).unwrap())
        .is_err());
    assert!(source
        .gradient_lengths()
        .clone()
        .with_preferred(RelativeSpacing::ratio(1, 1).unwrap())
        .is_err());
    let mut poisoned = source.clone();
    poisoned.gradient_lengths.preferred = RelativeSpacing::ratio(1, 1).unwrap();
    assert!(poisoned.clone().with_frame(None).is_err());
    assert!(poisoned.clone().with_shared_gap(zero.clone()).is_err());
    assert!(poisoned.with_spacing(source.spacing().clone()).is_err());
    let mut poisoned = source.clone();
    poisoned
        .frame
        .as_mut()
        .unwrap()
        .outline
        .as_mut()
        .unwrap()
        .width = f32::NAN;
    assert!(poisoned
        .clone()
        .with_gradient_lengths(source.gradient_lengths().clone())
        .is_err());
    assert!(poisoned.with_spacing(source.spacing().clone()).is_err());
    assert!(source.spacing().clone().with_padding(zero.clone()).is_ok());
    assert!(source.frame().unwrap().clone().with_radius(zero).is_ok());
}

#[test]
fn pinned_binary_sources_and_complete_edge_rounding_are_exact() {
    let source = source();
    let resources = OperationGenerationResources::new();
    let lengths = source.lengths(11, 12, &resources).unwrap();
    let edge = lengths
        .padding
        .add(&lengths.glyph_edge_padding, &resources)
        .unwrap();
    assert_eq!(
        edge.round(SpatialRounding::NearestEven, &resources)
            .unwrap()
            .quanta(),
        11264
    );
    assert_eq!(
        edge.round(SpatialRounding::Ceil, &resources)
            .unwrap()
            .quanta(),
        11265
    );
    assert_eq!(
        edge.times(2, &resources)
            .unwrap()
            .round(SpatialRounding::Ceil, &resources)
            .unwrap()
            .quanta(),
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

#[test]
fn builtin_admission_matches_independent_dyadic_and_euclidean_certificate() {
    let (work, bytes) = builtin_resource_oracle();
    assert_eq!(work, 355); // thirty-six Euclidean steps across the seven pinned P inputs
    for (visits, limit) in [
        (0, bytes),
        (work - 1, bytes),
        (work, bytes - 1),
        (work, bytes),
    ] {
        let resources = OperationGenerationResources::with_test_limits(visits, limit);
        let result = LegendTheme::builtin(&resources);
        assert_eq!(result.is_ok(), visits == work && limit == bytes);
        if result.is_ok() {
            assert_eq!(resources.usage().unwrap(), (0, work));
            assert_eq!(resources.byte_usage().unwrap(), bytes);
        }
    }
}

#[test]
fn sharing_and_identity_admit_each_actual_relationship_and_never_copy_buffers() {
    let source = source();
    for remaining in [0, 11, 12] {
        let resources = OperationGenerationResources::with_test_limits(remaining, 0);
        let result = source.share(&resources);
        assert_eq!(result.is_ok(), remaining == 12);
        assert_eq!(resources.byte_usage().unwrap(), 0);
        if let Ok(copy) = result {
            assert_eq!(copy, source);
        }
    }
    let copy = source.clone();
    for remaining in [0, 12, 13] {
        let resources = OperationGenerationResources::with_test_limits(remaining, 0);
        let result = source.same(&copy, &resources);
        assert_eq!(result.is_ok(), remaining == 13);
        if let Ok(equal) = result {
            assert!(equal);
        }
        assert_eq!(resources.usage().unwrap(), (0, remaining));
        assert_eq!(resources.byte_usage().unwrap(), 0);
    }
    let absent = source.clone().with_frame(None).unwrap();
    assert_eq!(absent.relationship_count(), 11);
    let resources = OperationGenerationResources::with_test_limits(1, 0);
    assert!(!source.same(&absent, &resources).unwrap());
}
