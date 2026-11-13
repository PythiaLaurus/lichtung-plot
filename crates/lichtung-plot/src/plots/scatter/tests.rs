use super::*;
use crate::alpha::{AlphaEncoding, AlphaValues};

#[test]
fn alpha_preserves_original_filtered_slots_coincident_points_paint_order_and_outlines() {
    let configured = Plot::scatter(
        crate::NumericValues::new("x", [1.0, f64::NAN, 1.0, 1.0]),
        crate::NumericValues::new("y", [1.0, 2.0, 1.0, 1.0]),
    )
    .source_rows(crate::SourceRows::new([100, 200, 300, 400]).unwrap())
    .with_alpha(
        AlphaTarget::Point,
        AlphaEncoding::observations(
            AlphaValues::categorical(
                crate::CategoryValues::new("confidence", ["low", "\u{10ffff}", "mid", "low"])
                    .unwrap(),
                |level| match level {
                    "low" => 0,
                    "mid" => 128,
                    _ => 42,
                },
            )
            .unwrap(),
        ),
    )
    .x_limits(0.0, 2.0)
    .y_limits(0.0, 2.0)
    .legend(crate::LegendPolicy::Hidden);
    let source = configured.clone().erase();
    let operation = crate::font::canonical::OperationContext::new();
    let (prepared, ranges) = crate::data::with_source_data_for_test(
        &source,
        &operation,
        crate::orchestrator::registration::RowSelection::Full,
        |data| PreparedScatter::new(source.description(), data).unwrap(),
    )
    .unwrap();
    assert_eq!(prepared.checked_primary_mark_count().unwrap(), 3);
    assert_eq!(
        prepared.groups[0]
            .points
            .iter()
            .map(|point| (point.alpha.slot, point.alpha.source_row))
            .collect::<Vec<_>>(),
        [(0, 100), (2, 300), (3, 400)]
    );
    assert_eq!(ranges, [(1.0, 1.0); 2]);
    let theme = crate::Theme::publication().resolve().unwrap();
    let text = crate::text::TextService::default();
    let theme = crate::text::TextRequestContext::new(&theme, &text);
    crate::orchestrator::with_prepared_plot_for_test(
        &source,
        &theme,
        |chart, inputs, theme, _operation| {
            let chart = chart.clone();
            let operation = theme.text.prepared_font_environment().unwrap().operation();
            let hidden_alpha = chart
                .prepared_discrete_guides
                .as_ref()
                .unwrap()
                .alpha_for_scope(None)
                .unwrap();
            assert!(chart
                .prepared_discrete_guide(inputs.plots.first())
                .unwrap()
                .is_none());
            assert_eq!(hidden_alpha.inputs, *source.description().alpha_inputs());
            assert_eq!(hidden_alpha.sources[0].glyphs[0].members.len(), 3);
            let selected = chart
                .measure_selected_interior(
                    crate::decorations::legend::discrete::scoped::GuideUse::Hidden,
                    PlotRenderContext::new(
                        inputs.plots[0].data(&operation).unwrap(),
                        crate::layout::PixelRect::new(10.0, 600.0, 20.0, 420.0),
                        theme,
                    )
                    .with_registration(&inputs.plots[0], inputs.intrinsic(&inputs.plots[0])),
                )
                .unwrap();
            let crate::layout::interior::MarkPayload::Scatter(payload) =
                &selected.marks.as_ref().unwrap().payload
            else {
                panic!("Scatter payload")
            };
            assert_eq!(
                payload
                    .groups
                    .iter()
                    .map(|(points, style)| (points.len(), style.color.get_alpha()))
                    .collect::<Vec<_>>(),
                [(1, 0), (1, 128), (1, 0)]
            );
            let outline = chart
                .description()
                .point_style(
                    theme,
                    (0, 0, 0),
                    theme.marks.get_points().get_radii().get_scatter(),
                )
                .outline;
            assert!(payload
                .groups
                .iter()
                .all(|(_, style)| style.outline == outline));
            let first = payload.groups[0].0[0];
            assert!(payload.groups.iter().all(|(points, _)| points[0] == first));
            let visible = configured
                .clone()
                .legend(crate::LegendPolicy::Visible(
                    crate::LegendPosition::OutsideRight,
                ))
                .labels(crate::Labels::raw())
                .erase();
            let descriptor = crate::decorations::legend::discrete::PreparedGuide::prepare_for_test(
                &visible,
                theme,
                crate::LegendComposition::Combined,
            )
            .unwrap()
            .unwrap();
            let section = &descriptor.sections[0];
            assert_eq!(
                section
                    .entries()
                    .iter()
                    .map(|entry| entry.display_label.as_str())
                    .collect::<Vec<_>>(),
                ["low", "mid"]
            );
            let crate::decorations::legend::discrete::GuideSectionBody::Alpha(alpha) =
                &section.body
            else {
                panic!("declared point alpha body")
            };
            assert_eq!(
                alpha.sources[0].glyphs[0]
                    .members
                    .iter()
                    .map(|member| member.value.alpha)
                    .collect::<Vec<_>>(),
                [0, 128, 0]
            );
            assert_eq!(alpha.sources[0].input.values().unwrap().len(), 4);
            assert_eq!(hidden_alpha.sources[0].glyphs, alpha.sources[0].glyphs);
            assert_eq!(hidden_alpha.sources[0].input, alpha.sources[0].input);
            let calls = crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get);
            let error = source
                .clone()
                .with_alpha(
                    AlphaTarget::Point,
                    AlphaEncoding::observations(AlphaValues::new("confidence", [0; 3]).unwrap()),
                )
                .preflight()
                .unwrap_err();
            assert!(matches!(error, Error::Data(_)));
            assert_eq!(
                crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
                calls
            );
        },
    )
    .unwrap();
}
