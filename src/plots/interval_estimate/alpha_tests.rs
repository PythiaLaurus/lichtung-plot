use super::*;

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one source-slot contract compares independent point/interval paints through ordering and unavailable rows"
)]
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
    .with_alpha(
        AlphaTarget::Line,
        crate::alpha::AlphaEncoding::observations(
            crate::alpha::AlphaValues::new("interval confidence", [230, 99, 0]).unwrap(),
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
    assert_eq!(
        prepared
            .line_alphas
            .iter()
            .map(|alpha| alpha.slot)
            .collect::<Vec<_>>(),
        [1, 2, 0]
    );
    let mut theme = crate::Theme::publication().resolve().unwrap();
    let interval = theme
        .marks
        .strokes()
        .summaries()
        .estimate_interval()
        .clone()
        .with_color(crate::Color::rgba(17, 41, 67, 0))
        .unwrap()
        .with_width(40.0)
        .unwrap();
    let summaries = theme
        .marks
        .strokes()
        .summaries()
        .clone()
        .with_estimate_interval(interval)
        .unwrap();
    let strokes = theme
        .marks
        .strokes()
        .clone()
        .with_summaries(summaries)
        .unwrap();
    theme.marks = theme.marks.clone().with_strokes(strokes).unwrap();
    assert_eq!(
        prepared.interval_ink(&theme).unwrap().to_bits(),
        40.0_f64.to_bits()
    );
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
    assert_eq!(
        payload.rows[1].interval_style.color,
        crate::Color::rgba(17, 41, 67, 0)
    );
    assert_eq!(
        payload.rows[2].interval_style.color,
        crate::Color::rgba(17, 41, 67, 230)
    );
    assert!(payload.rows[1].caps.iter().all(Option::is_some));
    assert!(payload.rows[2].caps.iter().all(Option::is_some));
}
