use super::*;

fn near(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 1.0 / 1024.0,
        "{actual} != {expected}"
    );
}

#[test]
fn typed_numeric_and_mixed_figure_references_keep_one_anchor_as_content_changes() {
    use crate::{DataCoordinate, Text, TextCoordinate, TextLayout};
    let theme = crate::Theme::publication().resolve().unwrap();
    let source = Plot::scatter(
        crate::NumericValues::new("x", [0.0, 20.0]),
        crate::NumericValues::new("y", [0.0, 10.0]),
    )
    .size(400, 300)
    .erase()
    .with_output_dimensions(1000, 800);
    let plot = crate::plots::shared_x::prepare_panel_for_test(&source, &theme).unwrap();
    let panel = PanelBox::new(20.0, 420.0, 30.0, 230.0);
    let layout = Layout {
        projection: crate::layout::Projection::new((0.0, 20.0), (0.0, 10.0), panel.as_tuple()),
        ..Layout::envelope(panel)
    };
    let position = TextPosition::new(
        TextCoordinate::data(DataCoordinate::numeric(12.0).unwrap()),
        TextCoordinate::data(DataCoordinate::numeric(5.0).unwrap()),
    );
    for (source, size) in [("A", 12.5), ("Longer\nlabel", 27.25)] {
        let style = theme
            .typography
            .annotations
            .panel_text
            .clone()
            .font_size(size)
            .unwrap();
        let text = Text::new(source).style(style).layout(
            TextLayout::default()
                .anchor(TextAnchor::bottom_left())
                .alignment(TextAlignment::End),
        );
        let note = crate::PanelTextAnnotation::at(position.clone(), text).unwrap();
        let plan = measure(&note, &plot, &layout, &theme, &FitScope::root()).unwrap();
        near(plan.placement.position.x - plan.text_plan.width(), 260.0);
        near(
            plan.placement.position.y - plan.text_plan.height() / 2.0,
            130.0,
        );
    }
    let position = TextPosition::new(
        TextCoordinate::figure_fraction(0.75).unwrap(),
        TextCoordinate::panel_fraction(0.25).unwrap(),
    );
    let note = crate::PanelTextAnnotation::at(
        position,
        Text::new("Figure note").layout(TextLayout::default().anchor(TextAnchor::center())),
    )
    .unwrap();
    let plan = measure(&note, &plot, &layout, &theme, &FitScope::root()).unwrap();
    near(
        plan.placement.position.x + plan.text_plan.width() / 2.0,
        750.0,
    );
    near(plan.placement.position.y, 80.0);
}

#[test]
fn raw_category_references_use_native_assigned_track_centers_not_aliases_or_uniform_bands() {
    use crate::{DataCoordinate, TextCoordinate};
    let theme = crate::Theme::publication().resolve().unwrap();
    let source = Plot::bar(
        crate::CategoryValues::new("group", ["first_raw", "second_raw"]).unwrap(),
        crate::BarInput::estimates(crate::NumericValues::new("value", [1.0, 2.0])),
    )
    .erase();
    let plot = crate::plots::shared_x::prepare_panel_for_test(&source, &theme).unwrap();
    let panel = PanelBox::new(20.0, 420.0, 30.0, 230.0);
    let mut layout = Layout::envelope(panel);
    layout.x_axis.assigned_cells = Some(vec![(20.0, 120.0), (120.0, 420.0)]);
    let position = TextPosition::new(
        TextCoordinate::data(DataCoordinate::category("second_raw")),
        TextCoordinate::panel_fraction(0.75).unwrap(),
    );
    let resolved = resolve::resolve(&position, &plot, &layout, &theme).unwrap();
    let point = resolve::point(resolved, panel);
    near(point.x, 270.0);
    near(point.y, 180.0);
    let bad = TextPosition::new(
        TextCoordinate::data(DataCoordinate::category("Second raw")),
        TextCoordinate::panel_fraction(0.5).unwrap(),
    );
    assert!(resolve::resolve(&bad, &plot, &layout, &theme).is_err());
}

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
        (plan.placement.position.x - plan.text_plan.width()).to_bits(),
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
        (plan.placement.position.x + 50.0).to_bits()
    );
    assert_eq!(
        placement.position.y.to_bits(),
        (plan.placement.position.y + 30.0).to_bits()
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
