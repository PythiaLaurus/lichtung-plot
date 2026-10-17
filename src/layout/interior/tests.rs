use super::*;

#[test]
fn invalid_route_rejects_before_fonts_or_selected_dispatch_without_fallback() {
    let plot = crate::Plot::scatter(
        crate::NumericValues::new("x", [0.0, 1.0]),
        crate::NumericValues::new("y", [1.0, 2.0]),
    )
    .erase();
    crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
    let error = plot.validate_configuration(PlotKind::Line).unwrap_err();
    assert!(error
        .to_string()
        .contains("interior route Line does not match"));
    assert_eq!(
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        0
    );
}

#[test]
fn prepared_contract_retains_sources_and_uses_the_callers_operation_limits() {
    use crate::font::canonical::OperationContext;
    let source = crate::Plot::scatter(
        crate::NumericValues::new("x", [0.0, 1.0]),
        crate::NumericValues::new("y", [1.0, 2.0]),
    )
    .erase();
    let resources = OperationContext::new();
    let prepared = crate::plots::shared_x::prepare_plot(
        &source,
        &resources,
        Some(&crate::theme::Theme::publication()),
    )
    .unwrap();
    assert_eq!(prepared.route, InteriorRoute::classify(PlotKind::Scatter));

    let attached = prepared.attach(source.clone()).unwrap();
    let wrong_family = crate::Plot::density(crate::NumericValues::new("x", [0.0, 1.0])).erase();
    assert!(matches!(prepared.attach(wrong_family), Err(Error::Data(_))));
    assert!(std::sync::Arc::ptr_eq(
        &attached.prepared_axes,
        &prepared.axes
    ));
    assert!(!std::sync::Arc::ptr_eq(
        &source.prepared_axes,
        &prepared.axes
    ));
}
