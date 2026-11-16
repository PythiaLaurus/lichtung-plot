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
    let mut theme = crate::Theme::publication().resolve().unwrap();
    theme.typography.annotations.contrast.light.font_size = 22.0;
    theme.typography.annotations.contrast.dark.font_size = 11.0;
    let text = crate::text::TextService::default();
    let mut appearance = theme;
    let baseline_appearance = appearance.clone();
    let theme = crate::text::TextRequestContext::new(&baseline_appearance, &text);
    let panel = PixelRect::new(0.0, 400.0, 0.0, 300.0);
    let opaque = original_source
        .preflight_panel(original_source.kind(), panel, &theme)
        .unwrap();
    assert_eq!(
        payload(&opaque).cells[0]
            .text
            .as_ref()
            .unwrap()
            .plan
            .style
            .font_size
            .to_bits(),
        22.0_f64.to_bits()
    );
    let mapped = original_source.clone().with_alpha(
        AlphaTarget::Fill,
        crate::alpha::AlphaEncoding::observations(
            crate::alpha::AlphaValues::new("cell confidence", [0]).unwrap(),
        ),
    );
    crate::orchestrator::with_prepared_plot_for_test(
        &mapped,
        &theme,
        |_| Ok(()),
        |chart, inputs, theme, operation, round| {
            let measurement = inputs.assignment_factory().assign(
                chart
                    .sizing_request()
                    .assigned_size()
                    .unwrap_or(crate::layout::RootSize::new(1, 1)),
            );
            let factory = inputs
                .plot_assignment_factory(&measurement, &inputs.plots[0].scope.id)
                .unwrap();
            let transparent =
                crate::layout::interior::measure_for_test(&factory, panel, round).unwrap();
            let cell = &payload(&transparent).cells[0];
            assert_eq!(cell.fill.color.get_alpha(), 0);
            let plan = &cell.text.as_ref().unwrap().plan;
            assert_eq!(plan.style.font_size.to_bits(), 11.0_f64.to_bits());
            let (marks, levels) = retained_in(inputs.plots.first()).unwrap();
            let axes = inputs.plots[0].axes.coordinate().unwrap();
            let intrinsic = inputs.plot_intrinsic(&inputs.plots[0].scope.id).unwrap();
            let original = heatmap_minima(
                intrinsic,
                &crate::orchestrator::root::intrinsic_inputs(
                    inputs.plots.first(),
                    theme,
                    operation,
                )
                .unwrap(),
            )
            .unwrap()
            .0;
            let crate::orchestrator::root::PlotIntrinsics::Heatmap { value } = intrinsic else {
                panic!("Heatmap slot")
            };
            let same_paint = source(policy.clone()).with_alpha(
                AlphaTarget::Fill,
                crate::alpha::AlphaEncoding::single(
                    crate::alpha::AlphaValues::new("cell confidence", [0]).unwrap(),
                )
                .unwrap(),
            );
            let (same_paint_inputs, _) =
                crate::plots::shared_x::prepare_plot(same_paint.description(), operation, theme)
                    .unwrap();
            let Some(crate::orchestrator::registration::PreparedFamily::Heatmap(same_paint_marks)) =
                &same_paint_inputs.panels[0].family
            else {
                panic!("prepared Heatmap")
            };
            assert_eq!(marks.fill_alphas, same_paint_marks.fill_alphas);
            assert!(!value.get().unwrap().matches_inputs(
                same_paint_marks,
                axes,
                &inputs.plots[0],
                theme
            ));
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
            let (changed_inputs, _) = crate::plots::shared_x::prepare_plot(
                changed_source.description(),
                operation,
                theme,
            )
            .unwrap();
            let Some(crate::orchestrator::registration::PreparedFamily::Heatmap(changed_marks)) =
                &changed_inputs.panels[0].family
            else {
                panic!("prepared Heatmap")
            };
            assert_eq!(marks.value_labels, changed_marks.value_labels);
            assert!(!value.get().unwrap().matches_inputs(
                changed_marks,
                axes,
                &inputs.plots[0],
                theme
            ));
            let padding = theme
                .annotations
                .get_cells()
                .clone()
                .value_horizontal_padding(crate::theme::RelativeSpacing::ratio(1, 1).unwrap())
                .unwrap()
                .value_vertical_padding(crate::theme::RelativeSpacing::ratio(2, 1).unwrap())
                .unwrap();
            appearance.annotations = appearance.annotations.clone().cells(padding).unwrap();
            let theme = crate::text::TextRequestContext::new(&appearance, theme.text);
            assert!(!value
                .get()
                .unwrap()
                .matches_inputs(marks, axes, &inputs.plots[0], &theme));
            let edited = heatmap_minima(
                intrinsic,
                &crate::orchestrator::root::intrinsic_inputs(
                    inputs.plots.first(),
                    &theme,
                    operation,
                )
                .unwrap(),
            )
            .unwrap()
            .0;
            assert_eq!(value.get().unwrap().minimum, original);
            assert!(!value
                .get()
                .unwrap()
                .matches_inputs(marks, axes, &inputs.plots[0], &theme));
            let required = (plan.width() + 11.0, plan.height() + 22.0);
            assert!(edited.0 >= original.0 && edited.1 >= original.1);
            assert!(edited.0 >= required.0 && edited.1 >= required.1);
            let annotations = AnnotationPlans {
                values: vec![Some(plan.clone())],
                rows: vec![None],
            };
            check_annotation_tracks(
                marks,
                levels,
                &annotations,
                &[0.0, required.0],
                &[required.1, 0.0],
                &theme,
                &crate::layout::fit::ROOT_SCOPE,
            )
            .unwrap();
            assert!(check_annotation_tracks(
                marks,
                levels,
                &annotations,
                &[0.0, required.0 - 1.0 / 1024.0],
                &[required.1, 0.0],
                &theme,
                &crate::layout::fit::ROOT_SCOPE
            )
            .is_err());
        },
    )
    .unwrap();
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
    let operation = crate::font::canonical::OperationContext::new();
    let scope = crate::orchestrator::registration::Scope::new(
        source.description(),
        &operation,
        0,
        crate::layout::ComponentAddress::root()
            .child(crate::layout::AddressRole::PlotBody, 0)
            .unwrap(),
        crate::orchestrator::registration::RowSelection::Selected {
            input_slots: vec![3, 2, 0, 1],
            diagnostic_rows: vec![400, 300, 100, 200],
        },
        None,
        None,
    )
    .unwrap();
    let row_source =
        crate::data::source::RowSourceContext::new(source.description(), &scope, &operation);
    let (prepared, _) = prepare(
        source.description(),
        crate::data::PlotData::checked(source.description(), &scope, &row_source, &operation)
            .unwrap(),
        source
            .description()
            .continuous_selection_in(crate::Theme::publication().color_scales())
            .unwrap(),
        scope
            .numeric_binding(source.description(), &row_source, &operation)
            .unwrap(),
        crate::orchestrator::registration::ContinuousMaterialization::Local,
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
    let text = crate::text::TextService::default();
    let theme = crate::text::TextRequestContext::new(&theme, &text);
    let frozen = crate::layout::plan_root(
        crate::layout::RootRequest::ExactSize {
            width: 1200,
            height: 900,
        },
        |stage| {
            crate::orchestrator::root::Root::prepare(
                crate::orchestrator::root::RootSource::Plot(source.description()),
                stage,
            )
        },
        |root, assigned, round| {
            let size = assigned.assigned_size()?;
            let measurement = root.assignment_factory().assign(size);
            let registered = &root.plots[0];
            let factory = root.plot_assignment_factory(&measurement, &registered.scope.id)?;
            let local = registered
                .local_guide_slot()?
                .expect("the color-plus-alpha native guide is present");
            let entry = factory.assign(
                crate::orchestrator::assignment::PlotAllocationInputs::for_test(
                    size,
                    PixelRect::new(20.0, 800.0, 20.0, 620.0),
                    round,
                    &registered.scope.address,
                ),
                round,
                None,
            )?;
            crate::decorations::plan::OUTER_GUIDE_MEASUREMENTS.with(|calls| calls.set(0));
            let planned = crate::layout::fit::measured(crate::decorations::plan::plan_plot(
                &factory,
                entry,
                crate::decorations::plan::LegendRole::Panel,
                crate::decorations::legend::discrete::scoped::GuideConsumption {
                    use_kind: crate::decorations::legend::discrete::scoped::GuideUse::Local,
                    local_guide: Some(&local),
                    dependencies: &[],
                },
                &crate::layout::fit::FitScope::new(&registered.scope.address, 0),
                round,
            ))
            .map_err(crate::layout::fit::PlanError::into_error)?;
            assert_eq!(
                planned.result.decoration_mode,
                crate::spec::DecorationMode::Standard
            );
            assert!(planned.legend.is_none());
            assert!(std::sync::Arc::ptr_eq(
                planned.result.discrete_guide.as_ref().unwrap(),
                &local.descriptor,
            ));
            assert!(!payload(&planned.result)
                .colorbar
                .as_ref()
                .unwrap()
                .calls
                .is_empty());
            crate::decorations::plan::OUTER_GUIDE_MEASUREMENTS
                .with(|calls| assert_eq!(calls.get(), 0));
            Ok(crate::layout::MeasuredLayout::new(size, planned))
        },
        |_, _, selected, _| Ok(selected),
    )
    .unwrap();
    let selected = frozen.plan();
    let cells = &payload(&selected.result).cells;
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
    let descriptor = selected.result.discrete_guide.as_ref().unwrap();
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
            .map(|entry| entry.display_label.as_str())
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
