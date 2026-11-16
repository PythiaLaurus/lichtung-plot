use super::*;

#[test]
fn projected_span_validation_preserves_source_intent_without_reprojection_or_requantization() {
    let projection = Projection::new((0.0, 1.0), (0.0, 1.0), (0.0, 100.0, 0.0, 100.0));
    for axis in [AxisSide::Bottom, AxisSide::Left] {
        for (source, nonzero, collapse) in [
            ((0.0, 1e-9), true, true),
            ((1e-9, 0.0), true, true),
            ((0.5, 0.5), true, true),
            ((0.0, -0.0), false, false),
            ((0.5, 0.5), false, false),
            ((0.0, 1.0 / (100.0 * 1024.0)), true, false),
            ((0.5, 1.5), true, false),
            ((2.0, 3.0), true, false),
            ((2.0, 2.0 + 1e-9), true, false),
            ((2.0, 2.0), true, false),
        ] {
            let project = |value| match axis {
                AxisSide::Bottom => projection.project(value, 0.0).0,
                AxisSide::Left => projection.project(0.0, value).1,
            };
            let projected = (project(source.0), project(source.1));
            let calls = Projection::test_projection_calls();
            let conversions =
                crate::render::coordinates::SPATIAL_CONVERSIONS.with(std::cell::Cell::get);
            let result = projection.validate_span(axis, source, projected, nonzero, "test polygon");
            assert_eq!(result.is_err(), collapse, "{source:?}: {result:?}");
            if let Err(error) = result {
                assert!(matches!(error, crate::Error::Render(_)), "{error}");
                assert!(error.to_string().contains("test polygon"), "{error}");
            }
            assert_eq!(calls, Projection::test_projection_calls());
            assert_eq!(
                conversions,
                crate::render::coordinates::SPATIAL_CONVERSIONS.with(std::cell::Cell::get)
            );
        }
    }
    let outside = (2.0, 2.0 + 1e-9);
    let projected = (
        projection.project(outside.0, 0.5).0,
        projection.project(outside.1, 0.5).0,
    );
    projection
        .validate_span(
            AxisSide::Bottom,
            outside,
            projected,
            true,
            "outside interval",
        )
        .unwrap();
    let error = crate::render::backend::validate_line_points(&[
        crate::render::backend::Point2D::new(projected.0, 50.0),
        crate::render::backend::Point2D::new(projected.1, 50.0),
    ])
    .unwrap_err();
    assert!(error.to_string().contains("collapse"), "{error}");
}
