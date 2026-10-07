use super::*;

#[test]
fn fractional_insets_and_gap_fit_once_and_subquantum_text_capacity_keeps_its_source_address() {
    let mut theme = crate::Theme::Publication
        .resolve(&crate::font::canonical::OperationGenerationResources::new())
        .unwrap();
    theme.typography.composition.text_panel_title = theme
        .typography
        .composition
        .text_panel_title
        .clone()
        .with_font_size(1)
        .unwrap();
    theme.typography.composition.body = theme
        .typography
        .composition
        .body
        .clone()
        .with_font_size(1)
        .unwrap();
    let third = crate::theme::RelativeSpacing::ratio(1, 3072).unwrap();
    theme.composition = theme
        .composition
        .clone()
        .with_text_panel(
            crate::theme::composition::TextPanelSpacing::new(third.clone(), third).unwrap(),
        )
        .unwrap();
    let source = TextPanel::new("0", "0").unwrap();
    let intrinsics = TextPanelIntrinsics::prepare(&source, &theme).unwrap();
    let hard = intrinsics.title.0.max(intrinsics.body.0);
    let title = fit::measured(fit::text(
        &theme,
        source.title(),
        &theme.typography.composition.text_panel_title,
        hard,
        PhysicalTextAlignment::Start,
        "fixture title",
        &FitScope::root(),
    ))
    .unwrap();
    let body = fit::measured(fit::text(
        &theme,
        source.body(),
        &theme.typography.composition.body,
        hard,
        PhysicalTextAlignment::Start,
        "fixture body",
        &FitScope::root(),
    ))
    .unwrap();
    let height = title.height() + body.height() + 1.0 / 1024.0;
    let measure = |width, height| {
        TextPanelPlan::measure(
            &source,
            0,
            PanelBox::new(0.0, width, 0.0, height),
            &theme,
            intrinsics,
            &FitScope::root(),
        )
        .unwrap()
    };
    let FitOutcome::Measured(plan) = measure(hard + 1.0 / 1024.0, height) else {
        panic!("complete fractional TextPanel reserve must fit");
    };
    assert_eq!(plan.title.plan().source, "0");
    assert_eq!(plan.body.plan().source, "0");
    assert!(matches!(
        measure(hard + 1.0 / 1024.0, height - 1.0 / 1024.0),
        FitOutcome::Miss(_)
    ));
    for width in [hard, 1.0 / 1024.0] {
        let FitOutcome::Miss(failure) = measure(width, height) else {
            panic!("insufficient exact text capacity must miss");
        };
        assert_eq!(failure.reason.kind, FitKind::Text);
        assert_eq!(
            failure.reason.address.leaf_role(),
            AddressRole::TextPanelTitle
        );
    }
    let mut ink = crate::render::geometry::GeometrySnapshot::new(&theme).unwrap();
    ink.text(plan.title_placement, plan.title.plan(), None)
        .unwrap();
    ink.text(plan.body_placement, plan.body.plan(), None)
        .unwrap();
    assert!(ink.bounds().iter().all(|bounds| bounds.left >= 0.0
        && bounds.right <= plan.cell.x1
        && bounds.bottom >= 0.0
        && bounds.top <= plan.cell.y1));
}
