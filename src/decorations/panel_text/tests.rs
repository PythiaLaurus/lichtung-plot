use super::*;

#[test]
fn selected_role_padding_is_retained_for_fit_and_reassignment() {
    let source = Plot::scatter(
        crate::NumericValues::new("x", [0.0, 1.0]),
        crate::NumericValues::new("y", [0.0, 1.0]),
    )
    .panel_text_annotation(crate::PanelTextAnnotation::new(PanelCorner::TopLeft, "Note").unwrap())
    .erase();
    let operation = crate::font::canonical::OperationContext::new();
    let inputs = crate::plots::shared_x::prepare_plot(&source, &operation, None).unwrap();
    let mut theme = crate::Theme::publication().resolve().unwrap();
    theme.typography.annotations.panel_text.font_size = 28;
    theme.annotations = theme
        .annotations
        .clone()
        .with_panel_text_padding(crate::theme::RelativeSpacing::ratio(1, 1).unwrap())
        .unwrap();
    let chart = inputs.attach_with_guides_for_test(source, &theme).unwrap();
    let panel = PanelBox::new(10.0, 410.0, 20.0, 320.0);
    let result = chart
        .measure_selected_interior(crate::spec::PlotRenderContext::new(
            chart.data(),
            panel,
            &theme,
        ))
        .unwrap();
    let plan = &result.panel_text[0];
    assert_eq!(plan.position.x.to_bits(), (panel.x0 + 28.0).to_bits());
    assert_eq!(
        plan.minimum_size.0.to_bits(),
        (plan.text_plan.width() + 56.0).to_bits()
    );
    assert_eq!(
        plan.minimum_size.1.to_bits(),
        (plan.text_plan.height() + 56.0).to_bits()
    );
    let moved = PanelBox::new(60.0, 460.0, 50.0, 350.0);
    let placement = plan.placement_in(moved);
    assert_eq!(
        placement.position.x.to_bits(),
        (plan.position.x + 50.0).to_bits()
    );
    assert_eq!(
        placement.position.y.to_bits(),
        (plan.position.y + 30.0).to_bits()
    );
    let bounds = plan.bounds_in(moved);
    assert_eq!(bounds.x0.to_bits(), (plan.bounds.x0 + 50.0).to_bits());
    assert_eq!(bounds.y1.to_bits(), (plan.bounds.y1 + 30.0).to_bits());
    let mut short = result.layout.clone();
    short.projection = crate::layout::Projection::new(
        (0.0, 1.0),
        (0.0, 1.0),
        (0.0, plan.minimum_size.0 - 1.0 / 1024.0, 0.0, 300.0),
    );
    assert!(measure_plans(&chart, &short, &theme, &crate::layout::fit::ROOT_SCOPE).is_err());
}
