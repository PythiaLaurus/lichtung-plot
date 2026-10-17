use super::*;
use crate::decorations::legend::descriptor::ContinuousContributor;
use crate::theme::{ColorDomain, ColorStop, ContinuousColorScale, DivergingGradient, OpaqueColor};

#[test]
fn shared_diverging_remapping_reaches_all_beeswarm_marks_and_the_common_guide() {
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
    let owner = crate::theme::ThemeValue::publication()
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
    .erase();
    let mut prepared = prepare(&source, source.data(), selection.clone()).unwrap();
    assert_eq!(prepared.colors, colors);
    assert_eq!(prepared.guide_values(), ["-2", "0", "2"]);
    let contributor = ContinuousContributor {
        field: "color".into(),
        title: "color".into(),
        selection: selection.clone(),
        samples: prepared.guide_samples().unwrap(),
        missing: None,
        domain: prepared.scale.domain(),
        exact: false,
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
    };
    let shared = ContinuousContributor::merge([contributor, sibling])
        .unwrap()
        .unwrap();
    prepared.install_shared_domain(&shared).unwrap();
    assert_eq!(prepared.guide_values(), ["-4", "0", "12"]);
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
        let offset =
            crate::theme::ContinuousColors::normalized_fraction(shared.domain, value).unwrap();
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
    let chart = inputs.attach(source).unwrap();
    let theme = crate::Theme::Publication.resolve().unwrap();
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
        crate::decorations::legend::LegendCall::Text(text) if text.plan.source == "0")));
}
