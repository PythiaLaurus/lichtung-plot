use super::*;

#[test]
fn root_padding_keeps_fractional_edges_until_the_complete_hard_boundary() {
    use crate::layout::canvas::RootFramePlan;
    let mut theme = super::super::Theme::publication().resolve().unwrap();
    let third = RelativeSpacing::ratio(1, 17 * 3072).unwrap();
    let zero = RelativeSpacing::ratio(0, 1).unwrap();
    theme.composition = theme
        .composition
        .clone()
        .root_padding(RootPadding::new(third.clone(), third, zero.clone(), zero).unwrap())
        .unwrap();
    let measure = |hard| {
        RootFramePlan::measure(
            (1, 1),
            &theme,
            [None; 3],
            crate::decorations::title_block::plan::RootAlignments::resolve(
                &theme, [None; 2], [None; 2],
            ),
            (hard, 1.0 / 1024.0),
        )
    };
    assert!(measure(1023.0 / 1024.0).is_ok());
    assert!(matches!(
        measure(1.0),
        Err(crate::layout::fit::PlanError::Fit(_))
    ));

    assert_eq!(
        theme
            .root_padding_lengths()
            .unwrap()
            .horizontal()
            .round(SpatialRounding::Ceil)
            .unwrap()
            .quanta(),
        1
    );
}
