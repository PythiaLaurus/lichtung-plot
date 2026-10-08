use super::*;

#[test]
fn exhaustive_route_census_has_twenty_ordinary_five_specialized_two_parents() {
    use PlotKind::*;
    let kinds = [
        Scatter,
        Histogram,
        Density,
        Qq,
        Regression,
        Line,
        Trajectory,
        LinePath,
        Interaction,
        PointInterval,
        Coefficient,
        Forest,
        Dumbbell,
        Bar,
        StackedBar,
        HorizontalBar,
        StackedHorizontalBar,
        Boxplot,
        Violin,
        Raincloud,
        Heatmap,
        Marginal,
        Matrix,
        Beeswarm,
        Waterfall,
    ];
    let mut counts = [0; 3];
    for kind in kinds {
        let route = InteriorRoute::classify(kind);
        assert_eq!(route.kind(), kind);
        // Both this independent census and production dispatch are exhaustive.
        let expected = match kind {
            Scatter | Line | Trajectory | LinePath | Histogram | Density | Qq | Regression => 0,
            PointInterval | Coefficient | Forest | Dumbbell | Bar | HorizontalBar | StackedBar
            | StackedHorizontalBar | Boxplot | Violin | Raincloud | Interaction => 1,
            Heatmap | Matrix | Marginal | Beeswarm | Waterfall => 2,
        };
        let index = match route {
            InteriorRoute::StandardCoordinate(_) => 0,
            InteriorRoute::OneCategoricalAxis(_) => 1,
            InteriorRoute::SpecializedGrid(_) => 2,
        };
        assert_eq!(index, expected);
        counts[index] += 1;
    }
    assert_eq!(counts, [8, 12, 5]);
    let parents = [
        CompositionChildPolicy::Facet,
        CompositionChildPolicy::Canvas,
    ];
    assert_ne!(parents[0], parents[1]);
}

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
        Some(&crate::theme::scale_values::test_defaults()),
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
