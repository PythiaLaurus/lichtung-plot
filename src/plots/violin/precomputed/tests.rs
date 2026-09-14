use super::*;
use crate::font::canonical::OperationGenerationResources;

fn source(with_hues: bool) -> Plot {
    Plot::raincloud(crate::RaincloudInput::geometry(crate::RaincloudGeometry {
        dimensions: crate::RaincloudDimensions {
            category: "category".into(),
            value: "value".into(),
            hue: with_hues.then(|| "color".into()),
        },
        groups: ["z", "a", "z", "m"]
            .into_iter()
            .enumerate()
            .map(|(index, hue)| RaincloudGroup {
                category: format!("category {index}"),
                hue: with_hues.then(|| hue.into()),
                density: vec![
                    crate::RaincloudDensityPoint {
                        y: 0.0,
                        normalized_width: 0.0,
                    },
                    crate::RaincloudDensityPoint {
                        y: 3.0,
                        normalized_width: 1.0,
                    },
                    crate::RaincloudDensityPoint {
                        y: 6.0,
                        normalized_width: 0.0,
                    },
                ],
                q1: 1.0,
                median: 3.0,
                q3: 5.0,
                points: vec![crate::RaincloudPoint {
                    y: 3.0,
                    normalized_strip: 0.5,
                }],
            })
            .collect(),
    }))
    .erase()
}

#[test]
fn prepared_hue_indices_preserve_order_and_obey_exact_work_and_storage_caps() {
    let source = source(true);
    HUE_LOOKUP_VISITS.with(|count| count.set(0));
    let resources = OperationGenerationResources::new();
    let prepared = prepare(&source, source.data().with_axis_resources(&resources)).unwrap();
    assert_eq!(prepared.hues, ["z", "a", "m"]);
    assert_eq!(
        prepared
            .groups
            .iter()
            .map(|group| group.local_color_index)
            .collect::<Vec<_>>(),
        [0, 1, 0, 2]
    );
    // Nonzero evidence distinguishes retained semantic work from an unused probe.
    assert_eq!(HUE_LOOKUP_VISITS.with(std::cell::Cell::get), 5);
    let visits = resources.usage().unwrap().1;
    let bytes = resources.byte_usage().unwrap();
    assert!(visits >= 5);
    assert!(bytes >= u64::try_from(4 * std::mem::size_of::<PreparedRaincloudGroup>()).unwrap());
    for (visits, bytes, diagnostic) in [
        (visits - 1, bytes, "work admission"),
        (visits, bytes - 1, "storage admission"),
    ] {
        let limited = OperationGenerationResources::with_test_limits(visits, bytes);
        let error = prepare(&source, source.data().with_axis_resources(&limited)).unwrap_err();
        assert!(error.to_string().contains(diagnostic), "{error}");
    }
    let exact = OperationGenerationResources::with_test_limits(visits, bytes);
    prepare(&source, source.data().with_axis_resources(&exact)).unwrap();
    assert_eq!(exact.usage().unwrap().1, visits);
    assert_eq!(exact.byte_usage().unwrap(), bytes);
}

#[test]
fn repeated_measurement_uses_retained_indices_and_charges_candidate_work() {
    for with_hues in [true, false] {
        let source = source(with_hues);
        HUE_LOOKUP_VISITS.with(|count| count.set(0));
        let inputs =
            crate::plots::shared_x::prepare_plot(&source, &OperationGenerationResources::new())
                .unwrap();
        let source = inputs.attach(source).unwrap();
        let lookup_visits = HUE_LOOKUP_VISITS.with(std::cell::Cell::get);
        assert_eq!(lookup_visits, if with_hues { 5 } else { 0 });
        let crate::plots::categorical_distribution::PreparedDistribution::Supplied(prepared) =
            crate::plots::categorical_distribution::retained(&source).unwrap()
        else {
            unreachable!()
        };
        let theme = crate::Theme::Publication.resolve();
        for (index, group) in prepared.groups.iter().enumerate() {
            let expected = if with_hues {
                [0, 1, 0, 2][index]
            } else {
                index
            };
            assert_eq!(group.local_color_index, expected);
            assert_eq!(
                group_color(&source, &group.geometry, group.local_color_index, &theme).unwrap(),
                theme.colors[expected % theme.colors.len()]
            );
        }
        let resources = theme
            .prepared_font_environment()
            .unwrap()
            .generation_resources();
        let measure = || {
            source.measure_selected_interior(PlotRenderContext::new(
                source.data(),
                crate::layout::PanelBox::new(20.25, 1420.75, 30.5, 350.25),
                &theme,
            ))
        };
        for _ in 0..2 {
            let before = resources.usage().unwrap().1;
            measure().unwrap();
            assert!(resources.usage().unwrap().1 > before);
            assert_eq!(HUE_LOOKUP_VISITS.with(std::cell::Cell::get), lookup_visits);
        }
        resources.set_test_limits(resources.usage().unwrap().1, u64::MAX);
        let projections = crate::layout::Projection::test_projection_calls();
        let error = measure().unwrap_err().into_error();
        assert!(
            error.to_string().contains("root processing visits limit"),
            "{error}"
        );
        assert_eq!(
            crate::layout::Projection::test_projection_calls(),
            projections
        );
        assert_eq!(HUE_LOOKUP_VISITS.with(std::cell::Cell::get), lookup_visits);
    }
}
