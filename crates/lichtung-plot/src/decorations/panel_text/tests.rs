use super::*;
use crate::orchestrator::assignment::PlotMeasureDimensions;
use crate::Plot;

fn near(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 1.0 / 1024.0,
        "{actual} != {expected}"
    );
}

#[test]
fn raw_category_references_use_native_assigned_track_centers_not_aliases_or_uniform_bands() {
    use crate::{DataCoordinate, TextCoordinate};
    let theme = crate::Theme::publication().resolve().unwrap();
    let text = crate::text::TextService::default();
    let theme = crate::text::TextRequestContext::new(&theme, &text);
    let source = Plot::bar(
        crate::CategoryValues::new("group", ["first_raw", "second_raw"]).unwrap(),
        crate::BarInput::estimates(crate::NumericValues::new("value", [1.0, 2.0])),
    )
    .erase();
    crate::orchestrator::with_prepared_plot_for_test(
        &source,
        &theme,
        |_| Ok(()),
        |_, inputs, theme, _operation, _round| {
            let panel = PixelRect::new(20.0, 420.0, 30.0, 230.0);
            let mut layout = Layout::envelope(panel);
            layout.x_axis.assigned_cells = Some(vec![(20.0, 120.0), (120.0, 420.0)]);
            let position = TextPosition::new(
                TextCoordinate::data(DataCoordinate::category("second_raw")),
                TextCoordinate::data_area_fraction(0.75).unwrap(),
            );
            let reference = crate::text::orientation::TextReferenceResolver {
                registered: &inputs.plots[0],
                layout: &layout,
                dimensions: PlotMeasureDimensions {
                    canvas: crate::layout::RootSize::new(420, 230),
                    output_override: None,
                },
            };
            let resolved = resolve::resolve(&position, &reference, theme).unwrap();
            let point = resolve::point(resolved, panel);
            near(point.x, 270.0);
            near(point.y, 180.0);
            let bad = TextPosition::new(
                TextCoordinate::data(DataCoordinate::category("Second raw")),
                TextCoordinate::data_area_fraction(0.5).unwrap(),
            );
            assert!(resolve::resolve(&bad, &reference, theme).is_err());
        },
    )
    .unwrap();
}
