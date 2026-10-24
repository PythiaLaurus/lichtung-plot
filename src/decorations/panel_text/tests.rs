use super::*;

#[test]
fn selected_local_style_padding_is_retained_for_fit_and_reassignment() {
    let appearance = crate::Theme::publication()
        .typography()
        .get_annotations()
        .get_panel_text()
        .clone()
        .font_size(28.0)
        .unwrap();
    let source = Plot::scatter(
        crate::NumericValues::new("x", [0.0, 1.0]),
        crate::NumericValues::new("y", [0.0, 1.0]),
    )
    .panel_text_annotation(
        crate::PanelTextAnnotation::new(
            PanelCorner::TopLeft,
            crate::Text::new("Note\nx")
                .style(appearance)
                .alignment(TextAlignment::End),
        )
        .unwrap(),
    )
    .erase();
    let operation = crate::font::canonical::OperationContext::new();
    let inputs = crate::plots::shared_x::prepare_plot(&source, &operation, None).unwrap();
    let mut theme = crate::Theme::publication().resolve().unwrap();
    theme.annotations = theme
        .annotations
        .clone()
        .panel_text_padding(crate::theme::RelativeSpacing::ratio(1, 1).unwrap())
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
    assert!(plan.text_plan.style.exact_override);
    assert_eq!(
        plan.text_plan.style.alignment,
        crate::layout::PhysicalTextAlignment::End
    );
    assert!(
        plan.text_plan.lines[1].translation_x_quanta > plan.text_plan.lines[0].translation_x_quanta
    );
    assert_eq!(
        (plan.position.x - plan.text_plan.width()).to_bits(),
        (panel.x0 + 28.0).to_bits()
    );
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
