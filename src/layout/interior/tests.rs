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
    let mut counts = [0; 4];
    for kind in kinds {
        let route = InteriorRoute::classify(kind);
        assert_eq!(route.kind(), kind);
        // Both classification and actual dispatch are exhaustive in the macro;
        // a new PlotKind cannot compile without choosing a concrete renderer.
        let index = match route {
            InteriorRoute::StandardCoordinate(_) => 2,
            InteriorRoute::OneCategoricalAxis(_) => 3,
            InteriorRoute::CategoricalLegacy(_) => 0,
            InteriorRoute::SpecializedLegacy(_) => 1,
        };
        counts[index] += 1;
    }
    assert_eq!(counts, [4, 5, 8, 8]);
    let parents = [
        ParentCompositionAdapter::Facet,
        ParentCompositionAdapter::Canvas,
    ];
    assert_ne!(parents[0], parents[1]);
}

#[test]
fn invalid_route_rejects_before_fonts_or_combined_dispatch_without_fallback() {
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
    use crate::font::canonical::OperationGenerationResources;
    let source = crate::Plot::scatter(
        crate::NumericValues::new("x", [0.0, 1.0]),
        crate::NumericValues::new("y", [1.0, 2.0]),
    )
    .erase();
    let resources = OperationGenerationResources::new();
    let prepared = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
    assert_eq!(prepared.route, InteriorRoute::classify(PlotKind::Scatter));
    let visits = resources.usage().unwrap().1;
    let bytes = resources.byte_usage().unwrap();
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
    for (visits, bytes, succeeds) in [
        (visits, bytes, true),
        (visits - 1, bytes, false),
        (visits, bytes - 1, false),
    ] {
        let resources = OperationGenerationResources::with_test_limits(visits, bytes);
        assert_eq!(
            crate::plots::shared_x::prepare_plot(&source, &resources).is_ok(),
            succeeds
        );
        if succeeds {
            // Another preparation cannot silently acquire a fresh budget.
            assert!(crate::plots::shared_x::prepare_plot(&source, &resources).is_err());
        }
    }
}
