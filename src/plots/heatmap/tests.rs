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
    let source = |format| {
        Plot::heatmap(
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
        .annotations(HeatmapAnnotations::Values(format))
        .erase()
    };
    let policy = crate::NumericFormat::new(
        crate::NumericNotation::Decimal,
        crate::NumericPrecision::AdaptiveFractional { min: 2, max: 3 },
    )
    .unwrap();
    let original_source = source(policy.clone());
    let operation = crate::font::canonical::OperationContext::new();
    let inputs = crate::plots::shared_x::prepare_plot(
        &original_source,
        &operation,
        Some(&crate::theme::Theme::publication()),
    )
    .unwrap();
    let mut theme = crate::Theme::publication().resolve().unwrap();
    theme.typography.annotations.contrast.light.font_size = 22;
    theme.typography.annotations.contrast.dark.font_size = 11;
    let chart = inputs
        .attach_with_guides_for_test(original_source, &theme)
        .unwrap();
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
    let mapped = chart.clone().with_alpha(
        AlphaTarget::Fill,
        crate::alpha::AlphaEncoding::observations(
            crate::alpha::AlphaValues::new("cell confidence", [0]).unwrap(),
        ),
    );
    let chart = crate::plots::shared_x::prepare_panel_for_test(&mapped, &theme).unwrap();
    let transparent = chart
        .measure_selected_interior(PlotRenderContext::new(chart.data(), panel, &theme))
        .unwrap();
    let cell = &payload(&transparent).cells[0];
    assert_eq!(cell.fill.color.get_alpha(), 0);
    let plan = &cell.text.as_ref().unwrap().plan;
    assert_eq!(plan.style.font_size, 11);
    let marks = retained_in(&chart).unwrap();
    let axes = chart.prepared_axis_inputs().unwrap();
    let original = prepare_intrinsics(marks, axes, &theme, None).unwrap();
    let same_paint = source(policy.clone()).with_alpha(
        AlphaTarget::Fill,
        crate::alpha::AlphaEncoding::single(
            crate::alpha::AlphaValues::new("cell confidence", [0]).unwrap(),
        )
        .unwrap(),
    );
    let same_paint_inputs = crate::plots::shared_x::prepare_plot(
        &same_paint,
        &operation,
        Some(&crate::Theme::publication()),
    )
    .unwrap();
    let same_paint_marks = &same_paint_inputs.heatmap.as_ref().unwrap().panels[0].1;
    assert_eq!(marks.fill_alphas, same_paint_marks.fill_alphas);
    assert!(!axes
        .heatmap_intrinsics
        .get()
        .unwrap()
        .matches_inputs(same_paint_marks, &theme));
    let changed_source = source(
        policy
            .try_copy()
            .unwrap()
            .negative_zero(crate::numeric::decimal::NegativeZero::Normalize)
            .unwrap(),
    )
    .with_alpha(
        AlphaTarget::Fill,
        crate::alpha::AlphaEncoding::observations(
            crate::alpha::AlphaValues::new("cell confidence", [0]).unwrap(),
        ),
    );
    let changed_inputs = crate::plots::shared_x::prepare_plot(
        &changed_source,
        &operation,
        Some(&crate::Theme::publication()),
    )
    .unwrap();
    let changed_marks = &changed_inputs.heatmap.as_ref().unwrap().panels[0].1;
    assert_eq!(marks.value_labels, changed_marks.value_labels);
    assert!(!axes
        .heatmap_intrinsics
        .get()
        .unwrap()
        .matches_inputs(changed_marks, &theme));
    let padding = theme
        .annotations
        .get_cells()
        .clone()
        .value_horizontal_padding(crate::theme::RelativeSpacing::ratio(1, 1).unwrap())
        .unwrap()
        .value_vertical_padding(crate::theme::RelativeSpacing::ratio(2, 1).unwrap())
        .unwrap();
    theme.annotations = theme.annotations.clone().cells(padding).unwrap();
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

#[test]
fn cell_alpha_retains_original_members_through_normalized_order_and_missing_grid_cells() {
    let source = Plot::heatmap(
        crate::CategoryValues::new("column", [" B", "A", " B", "A"]).unwrap(),
        crate::CategoryValues::new("row", ["two", "one", "one", "two"]).unwrap(),
        crate::NumericValues::nullable("value", [Some(1.0), None, Some(-1.0), Some(0.0)]),
    )
    .source_rows(crate::SourceRows::new([100, 200, 300, 400]).unwrap())
    .x_category_order(crate::CategoryOrder::explicit(["A", "B"]))
    .y_category_order(crate::CategoryOrder::explicit(["one", "two"]))
    .with_alpha(
        AlphaTarget::Fill,
        crate::alpha::AlphaEncoding::observations(
            crate::alpha::AlphaValues::categorical(
                crate::CategoryValues::new(
                    "cell confidence",
                    ["zero", "\u{10ffff}", "high", "mid"],
                )
                .unwrap(),
                |value| match value {
                    "zero" => 0,
                    "high" => 230,
                    "mid" => 64,
                    _ => 17,
                },
            )
            .unwrap(),
        ),
    )
    .labels(crate::Labels::raw())
    .erase();
    let prepared = prepare(
        &source,
        crate::data::PlotData::with_rows(&source, &[3, 2, 0, 1], &[400, 300, 100, 200]),
        source
            .continuous_selection_in(crate::Theme::publication().color_scales())
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        prepared
            .fill_alphas
            .iter()
            .map(|member| member.map(|member| (
                member.slot,
                member.source_row,
                member.value.unwrap().alpha
            )))
            .collect::<Vec<_>>(),
        [
            None,
            Some((2, 300, 230)),
            Some((3, 400, 64)),
            Some((0, 100, 0))
        ]
    );
    assert_eq!(
        prepared
            .guide_samples()
            .unwrap()
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>(),
        [-1.0_f64, 0.0, 1.0].map(f64::to_bits)
    );
    let theme = crate::Theme::publication().resolve().unwrap();
    let chart = crate::plots::shared_x::prepare_panel_for_test(&source, &theme).unwrap();
    let selected = chart
        .measure_selected_interior(PlotRenderContext::new(
            chart.data(),
            PanelBox::new(20.0, 800.0, 20.0, 620.0),
            &theme,
        ))
        .unwrap();
    let cells = &payload(&selected).cells;
    assert_eq!(
        cells
            .iter()
            .map(|cell| cell.fill.color.get_alpha())
            .collect::<Vec<_>>(),
        [
            theme.marks.get_cells().get_missing_fill().get_alpha(),
            230,
            64,
            0
        ]
    );
    assert!(cells[0].cross.is_some());
    assert!(cells[1..].iter().all(|cell| cell.cross.is_none()));
    let descriptor = selected.discrete_guide.as_ref().unwrap();
    let directory = tempfile::tempdir().unwrap();
    let destination = directory.path().join("native.svg");
    source
        .clone()
        .size(1200, 900)
        .svg_preserve_text(&destination)
        .unwrap();
    let svg = std::fs::read_to_string(destination).unwrap();
    assert_eq!(
        svg.matches("<linearGradient").count(),
        1,
        "the native color-plus-alpha guide has exactly one placement owner"
    );
    assert_eq!(descriptor.sections.len(), 2);
    assert!(matches!(
        descriptor.sections[0].body,
        crate::decorations::legend::discrete::GuideSectionBody::ContinuousColor(_)
    ));
    assert_eq!(
        descriptor.sections[1]
            .entries()
            .iter()
            .map(|entry| entry.display_label.as_ref())
            .collect::<Vec<_>>(),
        ["zero", "high", "mid"]
    );
    let crate::decorations::legend::discrete::GuideSectionBody::Alpha(alpha) =
        &descriptor.sections[1].body
    else {
        panic!("finite-cell alpha section")
    };
    assert_eq!(alpha.sources[0].glyphs[0].members.len(), 3);
    assert_eq!(alpha.sources[0].input.values().unwrap().len(), 4);
}
