use super::*;
use crate::font::canonical::OperationContext;
use crate::{CategoryValues, NumericValues};

fn source() -> Plot<crate::kind::Marginal> {
    Plot::marginal(
        NumericValues::new("x", [1.0, 1.2, 1.4, 3.0, 3.2, 3.4]),
        NumericValues::new("y", [2.0, 2.1, 2.3, 4.0, 4.2, 4.3]),
    )
    .hue(CategoryValues::new("hue", ["b", "b", "b", "a", "a", "a"]).unwrap())
    .size(800, 600)
}

#[test]
fn mapped_rows_survive_finite_filtering_grouping_and_selected_central_marks() {
    let source = Plot::marginal(
        NumericValues::new("x", [1.0, f64::NAN, 1.2, 1.4, 3.0, 3.2, 3.4]),
        NumericValues::new("y", [2.0, 0.0, 2.1, 2.3, 4.0, 4.2, 4.3]),
    )
    .hue(CategoryValues::new("hue", ["b", "discarded", "b", "b", "a", "a", "a"]).unwrap())
    .erase();
    let resources = OperationContext::new();
    let data = source
        .data()
        .map_source_rows(&[80, 100, 30, 70, 60, 10, 90])
        .with_operation(&resources);
    let prepared = prepare(&source, data).unwrap();
    assert_eq!(prepared.groups[0].source_rows, [80, 30, 70]);
    assert_eq!(prepared.groups[1].source_rows, [60, 10, 90]);
    let inputs = crate::plots::shared_x::prepare_reference_plot(
        &source,
        crate::plots::shared_x::PreparedReference::Marginal(prepared),
        None,
        None,
        None,
    )
    .unwrap();
    let chart = inputs.attach(source).unwrap();
    let theme = crate::Theme::Publication.resolve().unwrap();
    let selected = measure(
        &chart,
        PlotRenderContext::new(
            chart.data(),
            PanelBox::new(10.0, 900.0, 10.0, 700.0),
            &theme,
        ),
    )
    .unwrap();
    let crate::layout::interior::MarkPayload::Marginal(payload) =
        &selected.marks.as_ref().unwrap().payload
    else {
        panic!("Marginal")
    };
    assert_eq!(payload.groups[0].source_rows, [80, 30, 70]);
    assert_eq!(payload.groups[1].source_rows, [60, 10, 90]);
}

#[test]
fn complete_large_point_and_density_ink_survives_feedback_per_automatic_dimension() {
    for exact_x in [false, true] {
        let source = if exact_x {
            source().size(1600, 1400).x_limits(1.0, 3.4)
        } else {
            source().size(1600, 1400)
        }
        .erase();
        let inputs =
            crate::plots::shared_x::prepare_plot(&source, &OperationContext::new(), None).unwrap();
        let chart = inputs.attach(source).unwrap();
        let mut theme = crate::Theme::Publication.resolve().unwrap();
        theme.marks.point_sizes.marginal = 80;
        theme.marks.marginal_density.outline =
            Some(crate::theme::DataOutlineTheme::mapped(32.0, 255).unwrap());
        let selected = crate::decorations::plan::plan_panel_for_test(
            &chart,
            crate::PlotKind::Marginal,
            PanelBox::new(60.0, 1500.0, 60.0, 1250.0),
            crate::decorations::plan::LegendRole::Panel,
            &theme,
        )
        .unwrap();
        let crate::layout::interior::MarkPayload::Marginal(payload) =
            &selected.result.marks.as_ref().unwrap().payload
        else {
            panic!("Marginal")
        };
        assert!(payload.domain_expansions.iter().all(Option::is_none));
        let [central, top, right] = payload.clips();
        for group in &payload.groups {
            for (panel, strip) in [(top, &group.top), (right, &group.right)] {
                let mut geometry = crate::render::geometry::GeometryBackend::new(&theme).unwrap();
                geometry.begin(1600, 1400).unwrap();
                geometry.draw_polygon(strip, &group.fill).unwrap();
                geometry.end().unwrap();
                for bounds in geometry.bounds() {
                    if !exact_x || panel == right {
                        assert!(
                            bounds.left >= panel.x0 && bounds.right <= panel.x1,
                            "{bounds:?} {panel:?}"
                        );
                    }
                    assert!(
                        bounds.bottom >= panel.y0 && bounds.top <= panel.y1,
                        "{bounds:?} {panel:?}"
                    );
                }
            }
            let mut geometry = crate::render::geometry::GeometryBackend::new(&theme).unwrap();
            geometry.begin(1600, 1400).unwrap();
            geometry
                .draw_scatter(&group.points, &group.point_style)
                .unwrap();
            geometry.end().unwrap();
            for bounds in geometry.bounds() {
                if !exact_x {
                    assert!(bounds.left >= central.x0 && bounds.right <= central.x1);
                }
                assert!(bounds.bottom >= central.y0 && bounds.top <= central.y1);
            }
        }
    }
}

#[test]
fn nested_linked_panel_retains_its_address_and_projection() {
    let source = source().erase();
    let scope = layout::fit::FitScope::root()
        .child(layout::AddressRole::CanvasCell, 4)
        .unwrap()
        .child(layout::AddressRole::PlotBody, 0)
        .unwrap();
    let theme = crate::Theme::Publication.resolve().unwrap();

    let result = measure_linked_panel(
        PanelBox::new(10.0, 110.0, 10.0, 110.0),
        [(0.0, 1.0); 2],
        layout::AddressRole::MarginalTop,
        PlotRenderContext::new(source.data(), PanelBox::new(0.0, 200.0, 0.0, 200.0), &theme)
            .with_fit_scope(&scope),
    );
    let linked = result.unwrap();
    assert_eq!(linked.address.leaf_role(), layout::AddressRole::MarginalTop);
    assert_eq!(linked.address.leaf_slot_index(), 0);

    assert_eq!(
        linked.projection.panel_box(),
        PanelBox::new(10.0, 110.0, 10.0, 110.0)
    );
    assert_eq!(linked.projection.x_range(), (0.0, 1.0));
    assert_eq!(linked.projection.y_range(), (0.0, 1.0));
}

#[test]
fn marginal_raw_hue_color_map_requires_every_observed_identity_before_fonts() {
    let source = source()
        .categorical_colors(
            crate::CategoricalColors::new([
                ("b".into(), (8, 16, 24)),
                ("unused".into(), (8, 16, 24)),
            ])
            .unwrap(),
        )
        .erase();
    crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
    let error = crate::plots::shared_x::prepare_plot(&source, &OperationContext::new(), None)
        .err()
        .expect("missing visible hue must refuse");
    assert!(matches!(error, Error::Data(message) if message.contains("category \"a\"")));
    assert_eq!(
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        0
    );
}

#[test]
fn marginal_root_semantics_and_curves_are_prepared_before_fonts() {
    let source = source().erase();
    crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
    STAGES.with(|counts| counts.set([0; 3]));
    let resources = OperationContext::new();
    let inputs = crate::plots::shared_x::prepare_plot(&source, &resources, None).unwrap();
    assert_eq!(STAGES.with(std::cell::Cell::get), [1, 1, 0]);
    let marks = inputs.marginal.as_ref().unwrap();
    assert_eq!(marks.ranges, inputs.axes[0].1.ranges);
    assert_eq!(marks.curves.x_grid.len(), 80);
    assert_eq!(marks.groups.len(), 2);

    let attached = inputs.attach(source).unwrap();
    assert!(std::sync::Arc::ptr_eq(
        attached.prepared_marginal.as_ref().unwrap(),
        marks
    ));
    assert_eq!(
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        0
    );
}

#[test]
fn marginal_selected_geometry_preserves_linked_axes_and_drawn_ink() {
    let source = source().erase();
    let inputs =
        crate::plots::shared_x::prepare_plot(&source, &OperationContext::new(), None).unwrap();
    let source = inputs.attach(source).unwrap();
    let theme = crate::Theme::Publication.resolve().unwrap();
    let reads = crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get);
    let begins = crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(std::cell::Cell::get);
    let selected = source
        .measure_selected_interior(PlotRenderContext::new(
            source.data(),
            PanelBox::new(70.25, 670.75, 60.5, 450.25),
            &theme,
        ))
        .unwrap();
    assert_eq!(
        reads,
        crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get)
    );
    assert_eq!(
        begins,
        crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(std::cell::Cell::get)
    );
    let marks = selected.marks.as_ref().unwrap();
    let mut drawn = crate::render::geometry::GeometryBackend::new(&theme).unwrap();
    drawn.begin(800, 600).unwrap();
    marks.draw(&mut drawn).unwrap();
    drawn.end().unwrap();
    assert_eq!(marks.geometry.bounds(), drawn.bounds());
    let crate::layout::interior::MarkPayload::Marginal(payload) = &marks.payload else {
        panic!("wrong payload")
    };
    assert_eq!(payload.clips()[0], selected.layout.projection.panel_box());
    let [central, top, right] = payload.clips();
    assert_eq!((central.x0, central.x1), (top.x0, top.x1));
    assert_eq!((central.y0, central.y1), (right.y0, right.y1));
    assert_eq!(
        payload.panels[0].projection.x_range(),
        payload.panels[1].projection.x_range()
    );
    assert_eq!(
        payload.panels[0].projection.y_range(),
        payload.panels[2].projection.y_range()
    );
    assert_eq!(
        payload
            .panels
            .each_ref()
            .map(|panel| panel.address.leaf_role()),
        [
            layout::AddressRole::MarginalCentral,
            layout::AddressRole::MarginalTop,
            layout::AddressRole::MarginalRight
        ]
    );
    assert_eq!(
        (top.y0 - central.y1).to_bits(),
        theme.composition_grid_gap().unwrap().to_bits()
    );
    assert_eq!(
        (right.x0 - central.x1).to_bits(),
        theme.composition_grid_gap().unwrap().to_bits()
    );
    assert_eq!(payload.groups.len(), 2);
    assert_eq!(payload.groups[0].top.len(), 82);
    assert!(marks
        .geometry
        .bounds()
        .iter()
        .any(|ink| ink.left > payload.clips()[0].x1));
}

#[test]
fn marginal_invalid_semantics_fail_before_fonts_even_with_exact_limits() {
    crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
    for values in [[1.0, 1.0], [f64::MAX, -f64::MAX]] {
        let plot = Plot::marginal(
            NumericValues::new("x", values),
            NumericValues::new("y", [1.0, 2.0]),
        )
        .x_limits(0.0, 3.0)
        .y_limits(0.0, 3.0);
        assert!(matches!(plot.preflight(), Err(Error::Data(_))));
    }
    assert_eq!(
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        0
    );
}

#[test]
fn marginal_root_and_canvas_prepare_curves_once_per_component() {
    STAGES.with(|counts| counts.set([0; 3]));
    source().preflight().unwrap();
    assert_eq!(STAGES.with(std::cell::Cell::get)[..2], [1, 1]);
    STAGES.with(|counts| counts.set([0; 3]));
    crate::Canvas::grid(1, 2)
        .unwrap()
        .size(1600, 640)
        .panel(0, 0, source())
        .unwrap()
        .panel(0, 1, source())
        .unwrap()
        .preflight()
        .unwrap();
    assert_eq!(STAGES.with(std::cell::Cell::get)[..2], [2, 2]);
}

#[test]
fn marginal_parent_shared_domain_is_applied_before_curve_preparation() {
    let source = source().erase();
    let resources = OperationContext::new();
    let marks = crate::plots::shared_x::PreparedReference::new(
        &source,
        source.data().with_operation(&resources),
        None,
    )
    .unwrap();
    let inputs = crate::plots::shared_x::prepare_reference_plot(
        &source,
        marks,
        Some("panel-0"),
        Some((-5.0, 8.0)),
        None,
    )
    .unwrap();
    let marks = inputs.marginal.as_ref().unwrap();
    assert_eq!(marks.ranges[0], (-5.0, 8.0));
    assert_eq!(marks.ranges, inputs.axes[0].1.ranges);
    assert_eq!(marks.curves.x_grid.first(), Some(&-5.0));
    assert_eq!(marks.curves.x_grid.last(), Some(&8.0));
}

#[test]
fn marginal_parent_hue_order_controls_selected_colors_without_repreparation() {
    let source = source().erase();
    let inputs =
        crate::plots::shared_x::prepare_plot(&source, &OperationContext::new(), None).unwrap();
    let source = inputs
        .attach(source)
        .unwrap()
        .with_global_hue_levels(vec!["a".into(), "b".into()]);
    let theme = crate::Theme::Publication.resolve().unwrap();
    let stages = STAGES.with(std::cell::Cell::get);
    let selected = source
        .measure_selected_interior(PlotRenderContext::new(
            source.data(),
            PanelBox::new(70.0, 670.0, 60.0, 450.0),
            &theme,
        ))
        .unwrap();
    let crate::layout::interior::MarkPayload::Marginal(payload) =
        &selected.marks.as_ref().unwrap().payload
    else {
        panic!("wrong payload")
    };
    assert_eq!(
        payload.groups[0].point_style.color,
        crate::theme::Color::from(theme.color_scales.qualitative().colors()[1])
            .with_alpha(theme.marks.primary_point.fill_alpha)
    );
    assert_eq!(
        payload.groups[1].point_style.color,
        crate::theme::Color::from(theme.color_scales.qualitative().colors()[0])
            .with_alpha(theme.marks.primary_point.fill_alpha)
    );
    assert_eq!(
        payload.groups[0].fill.color,
        crate::theme::Color::from(theme.color_scales.qualitative().colors()[1]).with_alpha(115)
    );
    assert_eq!(stages[..2], STAGES.with(std::cell::Cell::get)[..2]);
}
