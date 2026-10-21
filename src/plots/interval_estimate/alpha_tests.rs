use super::*;

#[test]
fn point_interval_alpha_keeps_original_slots_through_category_order_and_unavailable_rows() {
    let source = Plot::point_interval(
        crate::CategoryValues::new("category", ["C", "A", "B"]).unwrap(),
        crate::NumericValues::nullable("estimate", [Some(3.0), None, Some(2.0)]),
        crate::Bounds::new(
            crate::NumericValues::nullable("lower", [Some(2.5), None, Some(1.5)]),
            crate::NumericValues::nullable("upper", [Some(3.5), None, Some(2.5)]),
        ),
    )
    .estimate_status([
        EstimateStatus::Estimated,
        EstimateStatus::NotEstimable,
        EstimateStatus::Estimated,
    ])
    .category_order(crate::CategoryOrder::explicit(["A", "B", "C"]))
    .with_alpha(
        AlphaTarget::Point,
        crate::alpha::AlphaEncoding::observations(
            crate::alpha::AlphaValues::new("confidence", [0, 127, 255]).unwrap(),
        ),
    )
    .x_limits(0.0, 4.0)
    .erase();
    let prepared = PreparedEstimate::new(&source, source.data(), []).unwrap();
    assert_eq!(prepared.labels, ["A", "B", "C"]);
    assert_eq!(
        prepared
            .alphas
            .iter()
            .map(|alpha| alpha.slot)
            .collect::<Vec<_>>(),
        [1, 2, 0]
    );
    assert_eq!(prepared.primary_mark_count(), 2);
    let theme = crate::Theme::publication().resolve().unwrap();
    let chart = crate::plots::shared_x::prepare_panel_for_test(&source, &theme).unwrap();
    let selected = chart
        .measure_selected_interior(PlotRenderContext::new(
            chart.data(),
            crate::layout::PanelBox::new(40.0, 600.0, 40.0, 420.0),
            &theme,
        ))
        .unwrap();
    let crate::layout::interior::MarkPayload::Estimate(payload) =
        &selected.marks.as_ref().unwrap().payload
    else {
        panic!("Estimate payload")
    };
    assert!(payload.rows[0].point.is_none());
    assert!(payload.rows[0].text.is_some());
    assert_eq!(payload.rows[1].point_style.color.alpha(), 255);
    assert_eq!(payload.rows[2].point_style.color.alpha(), 0);
    assert!(payload.rows[1].interval.is_some());
    assert!(payload.rows[2].interval.is_some());
}
