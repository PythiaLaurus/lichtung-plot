use super::*;

#[test]
fn matrix_hue_colors_follow_source_order_not_group_major_pairs() {
    let source = Plot::matrix(vec![
        crate::NumericValues::new("x", [1.0, 10.0, 4.0, 2.0, 11.0, 5.0, 3.0, 12.0, 6.0]),
        crate::NumericValues::new("y", [2.0, 20.0, 8.0, 4.0, 22.0, 10.0, 6.0, 24.0, 12.0]),
    ])
    .group(
        crate::CategoryValues::new(
            "group",
            ["g1", "g2", "g1", "g1", "g2", "g1", "g1", "g2", "g1"],
        )
        .unwrap(),
    )
    .hue(
        crate::CategoryValues::new(
            "cohort",
            [
                "first", "second", "third", "first", "second", "third", "first", "second", "third",
            ],
        )
        .unwrap(),
    )
    .erase();
    let theme = crate::Theme::publication().resolve().unwrap();
    let text = crate::text::TextService::default();
    crate::orchestrator::with_prepared_plot_for_test(
        &source,
        &crate::text::TextRequestContext::new(&theme, &text),
        |_| Ok(()),
        |chart, inputs, _request, _operation, round| {
            let registered = &inputs.plots[0];
            assert_eq!(
                registered
                    .categorical
                    .as_ref()
                    .unwrap()
                    .local_values()
                    .collect::<Vec<_>>(),
                ["first", "second", "third"]
            );
            let measurement = inputs.assignment_factory().assign(
                chart
                    .sizing_request()
                    .assigned_size()
                    .unwrap_or(crate::layout::RootSize::new(1, 1)),
            );
            let factory = inputs
                .plot_assignment_factory(&measurement, &registered.scope.id)
                .unwrap();
            let selected = crate::layout::interior::measure_for_test(
                &factory,
                PixelRect::new(20.0, 800.0, 20.0, 620.0),
                round,
            )
            .unwrap();
            let layout::interior::MarkPayload::Matrix(payload) =
                &selected.marks.as_ref().unwrap().payload
            else {
                panic!("Matrix payload")
            };
            let palette = theme.color_scales.get_qualitative().colors();
            let colors = payload.cells[1]
                .marks
                .iter()
                .filter_map(|mark| match mark {
                    MatrixPrimitive::Scatter(_, _, style) => {
                        Some((style.color.red(), style.color.green(), style.color.blue()))
                    }
                    MatrixPrimitive::Polygon(..) => None,
                })
                .collect::<Vec<_>>();
            assert_eq!(
                colors,
                [
                    palette[0].as_rgb(),
                    palette[2].as_rgb(),
                    palette[1].as_rgb()
                ]
            );
            assert_eq!(
                registered.guide.descriptor.as_ref().unwrap().counts.entries,
                3
            );
        },
    )
    .unwrap();
}
