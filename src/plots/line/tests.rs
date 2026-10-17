use super::*;

fn labeled_layer(value: f64, label: &str) -> crate::PointLayer {
    crate::PointLayer::new(
        crate::NumericValues::new("layer x", [value]),
        crate::NumericValues::new("layer y", [value]),
    )
    .labels(crate::CategoryValues::new("label", [label]).unwrap())
    .legend_label(label)
}

#[test]
fn supplemental_defaults_and_explicit_alpha_keep_label_paint_independent_and_guides_truthful() {
    let automatic = labeled_layer(1.0, "Auto");
    let alpha = labeled_layer(1.5, "Alpha")
        .opacity(0.5)
        .unwrap()
        .radius(9)
        .unwrap();
    let source = Plot::line(
        crate::NumericValues::new("x", [0.0, 2.0]),
        crate::NumericValues::new("y", [0.0, 2.0]),
    )
    .markers(false)
    .x_limits(-2.0, 4.0)
    .y_limits(-2.0, 4.0)
    .point_layer(automatic)
    .point_layer(alpha)
    .erase();
    let operation = crate::font::canonical::OperationContext::new();
    let inputs = crate::plots::shared_x::prepare_plot(&source, &operation, None).unwrap();
    let chart = inputs.attach(source).unwrap();
    let mut theme = crate::Theme::Publication.resolve().unwrap();
    let defaults = theme
        .marks
        .points()
        .supplemental()
        .clone()
        .with_radius(7)
        .unwrap()
        .with_fill_alpha(17)
        .unwrap();
    let points = theme
        .marks
        .points()
        .clone()
        .with_supplemental(defaults)
        .unwrap();
    theme.marks = theme.marks.clone().with_points(points).unwrap();
    let role_paint = crate::theme::Color::rgba(11, 23, 37, 71);
    theme.typography.annotations.point_layer_label = theme
        .typography
        .annotations
        .point_layer_label
        .clone()
        .with_color(role_paint)
        .unwrap()
        .with_font_size(16)
        .unwrap()
        .with_weight(crate::TextWeight::Semibold)
        .unwrap();
    let measured = chart
        .measure_selected_interior(PlotRenderContext::new(
            chart.data(),
            crate::layout::PanelBox::new(0.0, 600.0, 0.0, 400.0),
            &theme,
        ))
        .unwrap();
    let crate::layout::interior::MarkPayload::Line(payload) =
        &measured.marks.as_ref().unwrap().payload
    else {
        panic!("Line payload")
    };
    assert_eq!(payload.layers.len(), 2);
    let (automatic_points, automatic_style, automatic_text) = &payload.layers[0];
    assert_eq!(automatic_style.radius, 7);
    assert_eq!(automatic_style.color.alpha(), 17);
    assert_eq!(automatic_text[0].plan.style.paint, role_paint);
    let (alpha_points, alpha_style, alpha_text) = &payload.layers[1];
    assert_eq!(alpha_style.radius, 9);
    assert_eq!(alpha_style.color.alpha(), 128);
    assert_eq!(alpha_text[0].plan.style.paint, role_paint.with_alpha(128));
    assert_eq!(alpha_text[0].plan.style.font_size, 16);
    assert_eq!(alpha_text[0].plan.style.weight, crate::TextWeight::Semibold);
    assert_eq!(automatic_points.len(), 1);
    assert_eq!(alpha_points.len(), 1);
    let placement = &alpha_text[0].placement;
    assert_eq!(
        placement.position.x.to_bits(),
        (alpha_points[0].x + 9.0 + 3.2).round().to_bits()
    );
    assert_eq!(
        placement.position.y.to_bits(),
        (alpha_points[0].y + 9.0 + 3.2).round().to_bits()
    );
    assert_eq!(placement.anchor, TextAnchor::Start);
    assert_eq!(
        placement.origin,
        crate::render::backend::PlannedTextOrigin::Middle
    );
    assert_eq!(measured.legend_entries.len(), 2);
    let mut expected = vec![LegendEntry::point(
        chart.point_legend_identity("Alpha").unwrap(),
        alpha_style.clone(),
    )
    .without_hue_level()];
    crate::decorations::legend::resolve_entries(&chart, &mut expected).unwrap();
    assert!(measured.legend_entries[1].same_child_input(&expected[0]));
}
