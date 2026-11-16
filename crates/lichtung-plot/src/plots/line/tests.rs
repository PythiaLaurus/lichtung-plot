use super::*;

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one real builder and measurement operation checks local identity, sorted source alpha and the independent parent palette"
)]
fn trajectory_row_hue_identity_survives_parent_palette_order_and_condition_sort() {
    let source = Plot::trajectory(
        crate::NumericValues::new("condition", [3.0, 1.0, 2.0, 0.0]),
        crate::NumericValues::new("value", [3.0, 1.0, 2.0, 0.0]),
        crate::CategoryValues::new("path", ["A"; 4]).unwrap(),
    )
    .hue(crate::CategoryValues::new("cohort", ["second", "first", "first", "second"]).unwrap())
    .with_alpha(
        crate::alpha::AlphaTarget::Point,
        crate::alpha::AlphaEncoding::observations(
            crate::alpha::AlphaValues::new("confidence", [11, 23, 37, 41]).unwrap(),
        ),
    )
    .erase();
    let theme = crate::Theme::publication().resolve().unwrap();
    let text = crate::text::TextService::default();
    let selected = crate::orchestrator::with_prepared_plot_for_test(
        &source,
        &crate::text::TextRequestContext::new(&theme, &text),
        |registry| {
            let parent = crate::data::source::source_arc(
                crate::data::category::CategoryDomain::from_values(
                    source.description().hue_field().as_ref(),
                    vec!["first".into(), "second".into()],
                )?,
            )?;
            registry.panels[0].set_parent_colors(parent);
            Ok(())
        },
        |plot, inputs, _request, _, round| {
            let registered = &inputs.plots[0];
            let crate::orchestrator::registration::PreparedFamily::Line {
                marks: prepared, ..
            } = &registered.family
            else {
                panic!("prepared Trajectory")
            };
            assert_eq!(
                registered
                    .categorical
                    .as_ref()
                    .unwrap()
                    .local_values()
                    .collect::<Vec<_>>(),
                ["second", "first"]
            );
            let points = &prepared.groups[0].points;
            assert_eq!(
                points
                    .iter()
                    .map(|point| point.alpha.slot)
                    .collect::<Vec<_>>(),
                [3, 1, 2, 0]
            );
            assert_eq!(
                points
                    .iter()
                    .map(|point| point.hue_key.as_ref().unwrap().as_ref())
                    .collect::<Vec<_>>(),
                ["second", "first", "first", "second"].map(|value| {
                    crate::data::category::CategoryRef {
                        field: "cohort",
                        value,
                    }
                })
            );
            let measurement = inputs.assignment_factory().assign(
                plot.sizing_request()
                    .assigned_size()
                    .unwrap_or(crate::layout::RootSize::new(1, 1)),
            );
            let factory = inputs
                .plot_assignment_factory(&measurement, &registered.scope.id)
                .unwrap();
            let entry = factory
                .assign(
                    crate::orchestrator::assignment::PlotAllocationInputs::for_test(
                        measurement.measurement_size,
                        crate::layout::PixelRect::new(20.0, 600.0, 20.0, 420.0),
                        round,
                        &registered.scope.address,
                    ),
                    round,
                    None,
                )
                .unwrap();
            let scope = crate::layout::fit::FitScope::root();
            crate::orchestrator::measure_probe(
                &factory,
                entry,
                crate::decorations::legend::discrete::scoped::GuideConsumption {
                    use_kind: crate::decorations::legend::discrete::scoped::GuideUse::Local,
                    ..crate::decorations::legend::discrete::scoped::SUPPRESSED_GUIDES
                },
                &scope,
            )
            .unwrap()
            .1
        },
    )
    .unwrap();
    let crate::layout::interior::MarkPayload::Line(payload) =
        &selected.marks.as_ref().unwrap().payload
    else {
        panic!("Trajectory payload")
    };
    let palette = theme.color_scales.get_qualitative().colors();
    let first = palette[0].as_rgb();
    let second = palette[1].as_rgb();
    let paint = |rgb: (u8, u8, u8), alpha| crate::Color::rgba(rgb.0, rgb.1, rgb.2, alpha);
    assert_eq!(
        payload
            .marks
            .iter()
            .filter_map(|mark| match mark {
                LineMark::Points(_, style) => Some(style.color),
                _ => None,
            })
            .collect::<Vec<_>>(),
        [
            paint(second, 41),
            paint(first, 23),
            paint(first, 37),
            paint(second, 11)
        ]
    );
}
