use super::*;
use crate::alpha::{AlphaEncoding, AlphaTarget, AlphaValues};

#[test]
fn duplicate_outliers_keep_sorted_original_occurrences_and_replacement_paint() {
    let source = Plot::boxplot(
        crate::CategoryValues::new("category", ["a"; 12]).unwrap(),
        crate::NumericValues::new(
            "value",
            [
                10.0, 0.0, 0.0, 0.0, 0.0, 10.0, 0.0, -10.0, 0.0, 0.0, 0.0, 0.0,
            ],
        ),
    )
    .source_rows(crate::SourceRows::new((0..12).map(|row| 900 + 2 * row)).unwrap())
    .with_alpha(
        AlphaTarget::Point,
        AlphaEncoding::observations(
            AlphaValues::new(
                "confidence",
                [230, 17, 17, 17, 17, 0, 17, 128, 17, 17, 17, 17],
            )
            .unwrap(),
        ),
    )
    .erase();
    let prepared = prepare(
        &source,
        source
            .data()
            .map_source_rows(source.source_row_map().unwrap()),
        None,
    )
    .unwrap();
    assert_eq!(prepared.range, (-10.0, 10.0));
    assert_eq!(
        prepared.stats[0]
            .fliers
            .iter()
            .map(|observation| (observation.alpha.slot, observation.alpha.source_row))
            .collect::<Vec<_>>(),
        [(7, 914), (0, 900), (5, 910)]
    );
    assert_eq!(prepared.sample.groups[0].observations.len(), 12);
    let mut theme = crate::Theme::publication().resolve().unwrap();
    let primary = theme
        .marks
        .points()
        .primary()
        .clone()
        .with_fill_alpha(0)
        .unwrap()
        .with_outline(None)
        .unwrap();
    let radii = theme
        .marks
        .points()
        .radii()
        .clone()
        .with_box_flier(40)
        .unwrap();
    let points = theme
        .marks
        .points()
        .clone()
        .with_primary(primary)
        .unwrap()
        .with_radii(radii)
        .unwrap();
    theme.marks = theme.marks.clone().with_points(points).unwrap();
    let chart = crate::plots::shared_x::prepare_panel_for_test(&source, &theme).unwrap();
    assert_eq!(
        intrinsic_body_minimum(&chart, &theme).unwrap().1.to_bits(),
        80.0_f64.to_bits()
    );
    let selected = chart
        .measure_selected_interior(PlotRenderContext::new(
            chart.data(),
            crate::layout::PanelBox::new(20.0, 600.0, 20.0, 420.0),
            &theme,
        ))
        .unwrap();
    let crate::layout::interior::MarkPayload::Distribution(payload) =
        &selected.marks.as_ref().unwrap().payload
    else {
        panic!("Boxplot payload")
    };
    let points = payload
        .marks
        .iter()
        .filter_map(|mark| {
            if let categorical_distribution::Mark::Scatter(points, style) = mark {
                Some((points, style))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(
        points
            .iter()
            .map(|(points, style)| (points.len(), style.color.alpha()))
            .collect::<Vec<_>>(),
        [(1, 128), (1, 230), (1, 0)]
    );
    assert_eq!(points[1].0, points[2].0);
    assert!(points.iter().all(|(_, style)| style.outline.is_none()));
}
