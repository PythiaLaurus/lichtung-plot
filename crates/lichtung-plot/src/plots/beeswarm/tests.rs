use super::*;
use crate::decorations::legend::descriptor::ContinuousContributor;
use crate::layout;
use crate::theme::{ColorDomain, ColorStop, ContinuousColorScale, DivergingGradient, OpaqueColor};

#[test]
fn point_alpha_retains_source_order_color_samples_and_coincident_packing_even_at_zero() {
    let source = Plot::beeswarm(
        crate::CategoryValues::new("category", ["a"; 5]).unwrap(),
        crate::NumericValues::new("value", [0.0; 5]),
        crate::NumericValues::new("color", [-2.0, -1.0, 0.0, 1.0, 2.0]),
    )
    .source_rows(crate::SourceRows::new([90, 10, 80, 20, 70]).unwrap())
    .with_alpha(
        AlphaTarget::Point,
        crate::alpha::AlphaEncoding::observations(
            crate::alpha::AlphaValues::new("confidence", [0, 64, 128, 230, 0]).unwrap(),
        ),
    )
    .x_limits(-1.0, 1.0)
    .hide_point_outline()
    .legend(crate::LegendPolicy::Hidden)
    .erase();
    let mut theme = crate::Theme::publication().resolve().unwrap();
    let primary = theme
        .marks
        .get_points()
        .get_primary()
        .clone()
        .fill_alpha(0)
        .unwrap();
    let points = theme.marks.get_points().clone().primary(primary).unwrap();
    theme.marks = theme.marks.clone().points(points).unwrap();
    let text = crate::text::TextService::default();
    let theme = crate::text::TextRequestContext::new(&theme, &text);
    let select = |source: &Plot| {
        crate::orchestrator::with_prepared_plot_for_test(
            source,
            &theme,
            |chart, inputs, theme, _operation| {
                let chart = chart.clone();
                let operation = theme.text.prepared_font_environment().unwrap().operation();
                let selected = chart
                    .measure_selected_interior(
                        crate::decorations::legend::discrete::scoped::GuideUse::Hidden,
                        PlotRenderContext::new(
                            inputs.plots[0].data(&operation).unwrap(),
                            PixelRect::new(20.0, 700.0, 20.0, 420.0),
                            theme,
                        )
                        .with_registration(&inputs.plots[0], inputs.intrinsic(&inputs.plots[0])),
                    )
                    .unwrap();
                assert!(selected.discrete_guide.is_none());
                selected
            },
        )
        .unwrap()
    };
    let varied = select(&source);
    let layout::interior::MarkPayload::Beeswarm(varied) = &varied.marks.as_ref().unwrap().payload
    else {
        panic!("Beeswarm payload")
    };
    assert_eq!(
        varied.source_rows,
        [(0, 10), (0, 20), (0, 70), (0, 80), (0, 90)]
    );
    assert_eq!(
        varied
            .points
            .iter()
            .map(|(_, style)| style.color.get_alpha())
            .collect::<Vec<_>>(),
        [64, 230, 0, 128, 0]
    );
    let transparent = source.clone().with_alpha(
        AlphaTarget::Point,
        crate::alpha::AlphaEncoding::observations(
            crate::alpha::AlphaValues::new("confidence", [0; 5]).unwrap(),
        ),
    );
    let opaque = source.with_alpha(
        AlphaTarget::Point,
        crate::alpha::AlphaEncoding::observations(
            crate::alpha::AlphaValues::new("confidence", [255; 5]).unwrap(),
        ),
    );
    let transparent = select(&transparent);
    let layout::interior::MarkPayload::Beeswarm(transparent) =
        &transparent.marks.as_ref().unwrap().payload
    else {
        panic!("Beeswarm payload")
    };
    let opaque = select(&opaque);
    let layout::interior::MarkPayload::Beeswarm(opaque) = &opaque.marks.as_ref().unwrap().payload
    else {
        panic!("Beeswarm payload")
    };
    assert_eq!(transparent.points.len(), 5);
    assert!(transparent.geometry().unwrap().bounds().is_empty());
    assert_eq!(varied.source_rows, opaque.source_rows);
    for (((point, style), (opaque, opaque_style)), (transparent, _)) in varied
        .points
        .iter()
        .zip(&opaque.points)
        .zip(&transparent.points)
    {
        assert_eq!(point, opaque);
        assert_eq!(point, transparent);
        assert_eq!(style.color.alpha(255), opaque_style.color);
        assert!(style.outline.is_none());
    }
}

#[test]
fn shared_diverging_remapping_reaches_all_beeswarm_marks_and_the_common_guide() {
    let format = crate::numeric::format::NumericFormat::default()
        .precision(crate::numeric::format::NumericPrecision::AdaptiveFractional { min: 0, max: 2 })
        .unwrap()
        .decimal_scale(2)
        .unwrap()
        .suffix("%")
        .unwrap();
    let colors = [
        OpaqueColor::rgb(10, 20, 30),
        OpaqueColor::rgb(20, 90, 160),
        OpaqueColor::rgb(247, 247, 247),
        OpaqueColor::rgb(180, 100, 40),
        OpaqueColor::rgb(220, 40, 30),
    ];
    let gradient = DivergingGradient::new(
        [0.0, 0.25, 0.5, 0.75, 1.0]
            .into_iter()
            .zip(colors)
            .map(|(u, color)| ColorStop::new(u, color).unwrap()),
    )
    .unwrap();
    let owner = crate::theme::Theme::publication()
        .color_scales()
        .clone()
        .diverging(gradient)
        .unwrap();
    let selection = ContinuousColorScale::diverging(0.0, None, None)
        .unwrap()
        .resolved_in(&owner)
        .unwrap();
    let source = Plot::beeswarm(
        crate::CategoryValues::new("category", ["a"; 5]).unwrap(),
        crate::NumericValues::new("value", [0.0, 1.0, 2.0, 3.0, 4.0]),
        crate::NumericValues::new("color", [-2.0, -1.0, 0.0, 1.0, 2.0]),
    )
    .x_limits(-1.0, 5.0)
    .color_label_format(format.clone())
    .erase();
    let appearance = crate::Theme::publication().resolve().unwrap();
    let text = crate::text::TextService::default();
    let theme = crate::text::TextRequestContext::new(&appearance, &text);
    crate::layout::plan_root(
        crate::layout::RootRequest::ExactSize {
            width: 700,
            height: 400,
        },
        |stage| {
            let operation = stage.operation();
            let scope = crate::orchestrator::registration::Scope::new(
                &source,
                &operation,
                0,
                crate::layout::ComponentAddress::root()
                    .child(crate::layout::AddressRole::PlotBody, 0)?,
                crate::orchestrator::registration::RowSelection::Full,
                None,
                None,
            )
            .unwrap();
            let mut registry = crate::orchestrator::registration::PlotRegistryBuilder::new(
                operation.root_identity(),
            );
            let id = registry.reserve(source.description(), scope, &operation)?;
            let panel = &registry.panels[id.slot];
            let scope = &panel.scope;
            let row_source = &panel.row_source;
            let (mut prepared, levels) = prepare(
                &source,
                registry.data(&id, &operation)?,
                selection.clone(),
                scope
                    .numeric_binding(source.description(), row_source, &operation)
                    .unwrap(),
                panel.policy.continuous_materialization,
            )
            .unwrap();
            assert_eq!(prepared.colors, colors);
            assert_eq!(
                prepared.values.as_ref().unwrap().labels,
                ["-200%", "0%", "200%"]
            );
            let contributor = ContinuousContributor {
                field: "color".into(),
                title: "color".into(),
                selection: selection.clone(),
                samples: prepared.guide_samples().unwrap(),
                missing: None,
                domain: prepared.scale.domain(),
                exact: false,
                format: Some(std::sync::Arc::new(format.clone())),
                numeric: scope
                    .numeric_binding(source.description(), row_source, &operation)
                    .unwrap(),
                raw_windows: [source.description().x_axis_spec().window.as_ref(), None],
            };
            let sibling = ContinuousContributor {
                field: "color".into(),
                title: "color".into(),
                selection: selection
                    .domain(Some(ColorDomain::exact(-4.0, 12.0).unwrap()))
                    .unwrap(),
                samples: vec![-4.0, 6.0, 12.0],
                missing: None,
                domain: (-4.0, 12.0),
                exact: true,
                format: Some(std::sync::Arc::new(format)),
                numeric: scope
                    .numeric_binding(source.description(), row_source, &operation)
                    .unwrap(),
                raw_windows: [source.description().x_axis_spec().window.as_ref(), None],
            };
            let shared = crate::decorations::legend::descriptor::merge_continuous(
                &operation.root_identity(),
                &operation,
                [contributor, sibling],
            )
            .unwrap()
            .unwrap();
            prepared.install_shared_domain(&shared).unwrap();
            assert_eq!(
                prepared.values.as_ref().unwrap().labels,
                ["-400%", "0%", "1200%"]
            );
            assert_eq!(prepared.colors[0], colors[1]);
            assert_eq!(prepared.colors[2], colors[2]);
            assert_eq!(prepared.rows.len(), 5);
            assert!(shared
                .stops
                .iter()
                .any(|stop| stop.offset.to_bits() == 0.125_f64.to_bits()
                    && stop.color == colors[1].as_rgb()));
            assert!(shared
                .stops
                .iter()
                .any(|stop| stop.offset.to_bits() == 0.625_f64.to_bits()
                    && stop.color == colors[3].as_rgb()));
            for (&(_, _, value, _), color) in prepared.rows.iter().zip(&prepared.colors) {
                let offset =
                    crate::numeric::mapping::normalized_fraction(shared.domain, value).unwrap();
                assert!(shared
                    .stops
                    .iter()
                    .any(|stop| stop.offset.to_bits() == offset.to_bits()
                        && stop.color == color.as_rgb()));
            }
            let expected = prepared.colors.clone();
            let panel = &mut registry.panels[id.slot];
            #[expect(
                clippy::cast_precision_loss,
                reason = "category lengths are admitted source rows"
            )]
            let slots = (0.0, levels.len() as f64);
            panel.axes = Some(crate::plots::shared_x::prepare_pair_axes(
                panel,
                [prepared.x_data, slots].map(Some),
                [None, Some(levels)],
            )?);
            registry.panels[id.slot].family = Some(
                crate::orchestrator::registration::PreparedFamily::Beeswarm(prepared),
            );
            crate::plots::shared_x::prepare_reference_plot(
                &source,
                &mut registry.panels[id.slot],
                None,
                false,
                &operation,
            )
            .unwrap();
            let root_guide = registry.prepare_discrete_guides(
                &source,
                &theme,
                &operation,
                &crate::layout::ComponentAddress::root(),
                false,
            )?;
            let inputs = crate::layout::interior::PreparedInteriorInputs::from_registry(
                registry,
                crate::layout::interior::InteriorRoute::classify(source.kind()),
                root_guide,
            )?;
            let requirements = crate::layout::foundational_plot_requirements(
                Some(&inputs),
                &source,
                source.kind(),
                &theme,
                &crate::layout::ComponentAddress::root(),
                true,
                true,
                true,
                Some(stage),
            )?;
            let mut presence = stage.reserve_scalar_presence_map()?;
            let mut census = crate::layout::PreparedTextCensus::materialize(
                requirements,
                Some(stage),
                Some(&mut presence),
            )
            .map_err(|error| Error::Render(error.to_string()))?;
            text.prepare_font_environment_from_census(&mut census, presence, stage)?;
            Ok((inputs, expected))
        },
        |(inputs, expected), assigned, _| {
            let (width, height) = assigned.assigned_size()?.dimensions()?;
            let chart = source.clone().with_canvas_size(width, height);
            let operation = text.prepared_font_environment()?.operation();
            let measured = chart
                .measure_selected_interior(
                    crate::decorations::legend::discrete::scoped::GuideUse::Local,
                    PlotRenderContext::new(
                        inputs.plots[0].data(&operation)?,
                        PixelRect::new(0.0, 700.0, 0.0, 400.0),
                        &theme,
                    )
                    .with_registration(&inputs.plots[0], inputs.intrinsic(&inputs.plots[0])),
                )
                .unwrap();
            let crate::layout::interior::MarkPayload::Beeswarm(payload) =
                &measured.marks.as_ref().unwrap().payload
            else {
                panic!("Beeswarm payload")
            };
            assert_eq!(payload.points.len(), 5);
            for ((_, style), color) in payload.points.iter().zip(expected) {
                assert_eq!(
                    (style.color.red(), style.color.green(), style.color.blue()),
                    color.as_rgb()
                );
                assert_eq!(
                    style.color.get_alpha(),
                    theme.marks.get_points().get_primary().get_fill_alpha()
                );
            }
            assert!(payload
                .colorbar
                .as_ref()
                .unwrap()
                .calls
                .iter()
                .any(|call| matches!(call,
        crate::decorations::legend::LegendCall::Text(text) if text.plan.source == "0%")));
            Ok(crate::layout::MeasuredLayout::new(
                assigned.assigned_size()?,
                (),
            ))
        },
        |_, _, (), _| Ok(()),
    )
    .unwrap();
}
