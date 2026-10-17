use super::*;

fn payload(result: &PlotRenderResult) -> &HeatmapPayload {
    let crate::layout::interior::MarkPayload::Heatmap(payload) =
        &result.marks.as_ref().unwrap().payload
    else {
        panic!("selected Heatmap payload")
    };
    payload
}

#[test]
fn final_cell_alpha_selects_the_role_used_by_padding_tracks_and_capture() {
    let source = Plot::heatmap(
        crate::CategoryValues::new("column", ["a"]).unwrap(),
        crate::CategoryValues::new("row", ["one"]).unwrap(),
        crate::NumericValues::new("value", [1.0]),
    )
    .continuous_color_scale(
        crate::ContinuousColorScale::sequential(
            None,
            Some(crate::ColorDomain::exact(0.0, 1.0).unwrap()),
        )
        .unwrap(),
    )
    .annotations(HeatmapAnnotations::Values(
        crate::NumericLabelFormat::from_policy(
            crate::numeric::format::NumericFormat::new(
                crate::numeric::decimal::NumericNotation::Decimal,
                crate::numeric::format::NumericPrecision::AdaptiveFractional { min: 2, max: 3 },
            )
            .unwrap(),
        ),
    ))
    .erase();
    let operation = crate::font::canonical::OperationContext::new();
    let inputs = crate::plots::shared_x::prepare_plot(
        &source,
        &operation,
        Some(&crate::theme::Theme::publication()),
    )
    .unwrap();
    let chart = inputs.attach(source).unwrap();
    let mut theme = crate::Theme::publication().resolve().unwrap();
    theme.typography.annotations.contrast.light.font_size = 22;
    theme.typography.annotations.contrast.dark.font_size = 11;
    let panel = PanelBox::new(0.0, 400.0, 0.0, 300.0);
    let opaque = chart
        .measure_selected_interior(PlotRenderContext::new(chart.data(), panel, &theme))
        .unwrap();
    assert_eq!(
        payload(&opaque).cells[0]
            .text
            .as_ref()
            .unwrap()
            .plan
            .style
            .font_size,
        22
    );
    let cells = theme.marks.cells().clone().with_fill_alpha(0).unwrap();
    theme.marks = theme.marks.clone().with_cells(cells).unwrap();
    let transparent = chart
        .measure_selected_interior(PlotRenderContext::new(chart.data(), panel, &theme))
        .unwrap();
    let cell = &payload(&transparent).cells[0];
    assert_eq!(cell.fill.color.alpha(), 0);
    let plan = &cell.text.as_ref().unwrap().plan;
    assert_eq!(plan.style.font_size, 11);
    let marks = retained_in(&chart).unwrap();
    let axes = chart.prepared_axis_inputs().unwrap();
    let original = prepare_intrinsics(marks, axes, &theme, None).unwrap();
    let padding = theme
        .annotations
        .cells()
        .clone()
        .with_value_horizontal_padding(crate::theme::RelativeSpacing::ratio(1, 1).unwrap())
        .unwrap()
        .with_value_vertical_padding(crate::theme::RelativeSpacing::ratio(2, 1).unwrap())
        .unwrap();
    theme.annotations = theme.annotations.clone().with_cells(padding).unwrap();
    assert!(!axes
        .heatmap_intrinsics
        .get()
        .unwrap()
        .matches_inputs(marks, &theme));
    let edited = prepare_intrinsics(marks, axes, &theme, None).unwrap();
    let required = (plan.width() + 11.0, plan.height() + 22.0);
    assert!(edited.0 >= original.0 && edited.1 >= original.1);
    assert!(edited.0 >= required.0 && edited.1 >= required.1);
    let annotations = AnnotationPlans {
        values: vec![Some(plan.clone())],
        rows: vec![None],
    };
    check_annotation_tracks(
        marks,
        &annotations,
        &[0.0, required.0],
        &[required.1, 0.0],
        &theme,
        &crate::layout::fit::ROOT_SCOPE,
    )
    .unwrap();
    assert!(check_annotation_tracks(
        marks,
        &annotations,
        &[0.0, required.0 - 1.0 / 1024.0],
        &[required.1, 0.0],
        &theme,
        &crate::layout::fit::ROOT_SCOPE
    )
    .is_err());
}
