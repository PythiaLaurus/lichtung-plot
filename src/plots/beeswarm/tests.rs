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
    .erase()
    .with_suppressed_continuous_guide();
    let mut theme = crate::Theme::publication().resolve().unwrap();
    let primary = theme
        .marks
        .points()
        .primary()
        .clone()
        .with_fill_alpha(0)
        .unwrap();
    let points = theme.marks.points().clone().with_primary(primary).unwrap();
    theme.marks = theme.marks.clone().with_points(points).unwrap();
    let select = |source: &Plot| {
        let chart = crate::plots::shared_x::prepare_panel_for_test(source, &theme).unwrap();
        let selected = chart
            .measure_selected_interior(PlotRenderContext::new(
                chart.data(),
                PanelBox::new(20.0, 700.0, 20.0, 420.0),
                &theme,
            ))
            .unwrap();
        assert!(selected.discrete_guide.is_none());
        selected
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
            .map(|(_, style)| style.color.alpha())
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
        assert_eq!(style.color.with_alpha(255), opaque_style.color);
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
        .with_diverging(gradient)
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
    let mut prepared = prepare(&source, source.data(), selection.clone()).unwrap();
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
    };
    let sibling = ContinuousContributor {
        field: "color".into(),
        title: "color".into(),
        selection: selection
            .with_domain(Some(ColorDomain::exact(-4.0, 12.0).unwrap()))
            .unwrap(),
        samples: vec![-4.0, 6.0, 12.0],
        missing: None,
        domain: (-4.0, 12.0),
        exact: true,
        format: Some(std::sync::Arc::new(format)),
    };
    let shared = ContinuousContributor::merge([contributor, sibling])
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
    assert!(shared.stops.iter().any(
        |stop| stop.offset.to_bits() == 0.125_f64.to_bits() && stop.color == colors[1].as_rgb()
    ));
    assert!(shared.stops.iter().any(
        |stop| stop.offset.to_bits() == 0.625_f64.to_bits() && stop.color == colors[3].as_rgb()
    ));
    for (&(_, _, value, _), color) in prepared.rows.iter().zip(&prepared.colors) {
        let offset = crate::numeric::mapping::normalized_fraction(shared.domain, value).unwrap();
        assert!(shared
            .stops
            .iter()
            .any(|stop| stop.offset.to_bits() == offset.to_bits() && stop.color == color.as_rgb()));
    }
    let expected = prepared.colors.clone();
    let inputs = crate::plots::shared_x::prepare_reference_plot(
        &source,
        crate::plots::shared_x::PreparedReference::Beeswarm(prepared),
        None,
        None,
        None,
    )
    .unwrap();
    let theme = crate::Theme::publication().resolve().unwrap();
    let chart = inputs.attach_with_guides_for_test(source, &theme).unwrap();
    let measured = chart
        .measure_selected_interior(PlotRenderContext::new(
            chart.data(),
            PanelBox::new(0.0, 700.0, 0.0, 400.0),
            &theme,
        ))
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
            style.color.alpha(),
            theme.marks.points().primary().fill_alpha()
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
}
