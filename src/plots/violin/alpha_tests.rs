use super::*;
use crate::alpha::{AlphaEncoding, AlphaTarget, AlphaValues};

#[test]
fn overlay_alpha_follows_original_slots_and_source_jitter_through_positioned_groups() {
    let source = Plot::violin(
        crate::CategoryValues::new("category", ["B", "A", "B", "A"]).unwrap(),
        crate::NumericValues::new("value", [3.0, 1.0, 4.0, 2.0]),
    )
    .series(crate::CategoryValues::new("cohort", ["u", "v", "u", "v"]).unwrap())
    .with_alpha(
        AlphaTarget::Line,
        AlphaEncoding::keyed(
            [
                crate::CategoryValues::new("category", ["A", "B"]).unwrap(),
                crate::CategoryValues::new("cohort", ["v", "u"]).unwrap(),
            ],
            AlphaValues::new("summary confidence", [230, 0]).unwrap(),
        )
        .unwrap(),
    )
    .raw_overlay(crate::RawOverlay::Jittered)
    .source_rows(crate::SourceRows::new([900, 100, 800, 200]).unwrap())
    .with_alpha(
        AlphaTarget::Point,
        AlphaEncoding::observations(AlphaValues::new("confidence", [0, 64, 230, 64]).unwrap()),
    )
    .y_limits(-5.0, 10.0)
    .erase();
    let prepared = prepare(
        &source,
        PlotData::with_rows(&source, &[3, 0, 2], &[200, 900, 800]),
        None,
    )
    .unwrap();
    assert_eq!(
        prepared
            .sample
            .groups
            .iter()
            .flat_map(|group| group.observations.iter().map(|observation| (
                observation.alpha.slot,
                observation.alpha.source_row,
                observation.alpha.value.unwrap().alpha
            )))
            .collect::<Vec<_>>(),
        [(3, 200, 64), (0, 900, 0), (2, 800, 230)]
    );
    let theme = crate::Theme::publication().resolve().unwrap();
    let chart = crate::plots::shared_x::prepare_panel_for_test(&source, &theme).unwrap();
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
        panic!("Violin payload")
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
        [(1, 0), (1, 230), (2, 64)]
    );
    let expected =
        crate::plots::distribution_sample::overlay_x(crate::RawOverlay::Jittered, 0.32, 0.34, 800);
    let expected = selected.layout.projection.project(expected, 4.0);
    assert_eq!(points[1].0[0], Point2D::new(expected.0, expected.1));
    assert!(points.iter().all(|(_, style)| style.outline.is_some()));
    assert_eq!(
        payload
            .marks
            .iter()
            .filter_map(|mark| match mark {
                categorical_distribution::Mark::Line(_, style) => Some(style.color.alpha()),
                _ => None,
            })
            .collect::<Vec<_>>(),
        [0, 0, 0, 0, 230, 230, 230, 230]
    );
}
