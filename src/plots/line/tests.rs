use super::*;

fn labeled_layer(value: f64, label: &str) -> crate::PointLayer {
    crate::PointLayer::new(
        crate::NumericValues::new("layer x", [value]),
        crate::NumericValues::new("layer y", [value]),
    )
    .labels(crate::CategoryValues::new("label", [label]).unwrap())
    .legend_label(label)
}

fn measure_layer(
    layer: crate::PointLayer,
    theme: &crate::theme::ResolvedTheme,
) -> PlotRenderResult {
    let source = Plot::line(
        crate::NumericValues::new("x", [0.0, 2.0]),
        crate::NumericValues::new("y", [0.0, 2.0]),
    )
    .markers(false)
    .x_limits(-2.0, 4.0)
    .y_limits(-2.0, 4.0)
    .point_layer(layer)
    .erase();
    let chart = crate::plots::shared_x::prepare_panel_for_test(&source, theme).unwrap();
    chart
        .measure_selected_interior(PlotRenderContext::new(
            chart.data(),
            crate::layout::PanelBox::new(0.0, 600.0, 0.0, 400.0),
            theme,
        ))
        .unwrap()
}

#[test]
fn exact_supplemental_color_and_alpha_edits_freeze_identically_in_either_order() {
    let theme = crate::Theme::publication().resolve().unwrap();
    let paint = crate::Color::rgba(13, 29, 47, 53);
    let layer = labeled_layer(1.0, "Exact").with_radius(9).unwrap();
    let color_then_alpha = measure_layer(layer.clone().with_color(paint).with_alpha(131), &theme);
    let alpha_then_color = measure_layer(layer.with_alpha(131).with_color(paint), &theme);
    let crate::layout::interior::MarkPayload::Line(first) =
        &color_then_alpha.marks.as_ref().unwrap().payload
    else {
        panic!("Line payload")
    };
    let crate::layout::interior::MarkPayload::Line(second) =
        &alpha_then_color.marks.as_ref().unwrap().payload
    else {
        panic!("Line payload")
    };
    let (points, style, text) = &first.layers[0];
    let (other_points, other_style, other_text) = &second.layers[0];
    assert_eq!(style, other_style);
    assert_eq!(style.color, paint.with_alpha(131));
    assert_eq!(style.radius, 9);
    assert!(style.outline.is_none());
    assert_eq!(text[0].plan.style.paint, paint.with_alpha(131));
    assert_eq!(text[0].plan.style, other_text[0].plan.style);
    assert_eq!(text[0].plan.source, "Exact");
    assert_eq!(points.len(), 1);
    assert_eq!(points[0].x.to_bits(), other_points[0].x.to_bits());
    assert_eq!(points[0].y.to_bits(), other_points[0].y.to_bits());
    let captured =
        crate::render::primitives::CanonicalScatter::new(points, style, 400, None).unwrap();
    assert_eq!(captured.color, paint.with_alpha(131));
    assert_eq!(
        color_then_alpha
            .discrete_guide
            .as_ref()
            .unwrap()
            .counts
            .entries,
        1
    );
    assert_eq!(
        color_then_alpha.discrete_guide,
        alpha_then_color.discrete_guide
    );
}

#[test]
fn supplemental_defaults_and_explicit_alpha_keep_label_paint_independent_and_guides_truthful() {
    let automatic = labeled_layer(1.0, "Auto");
    let alpha = labeled_layer(1.5, "Alpha")
        .with_alpha(128)
        .with_radius(9)
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
    let mut theme = crate::Theme::publication().resolve().unwrap();
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
    let chart = crate::plots::shared_x::prepare_panel_for_test(&source, &theme).unwrap();
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
    let guide = measured.discrete_guide.as_ref().unwrap();
    assert_eq!(guide.counts.entries, 2);
    assert_eq!(
        guide.sections[0].entries[1].identity,
        crate::decorations::legend::GuideEntryIdentity::supplemental("Alpha").unwrap()
    );
    assert_eq!(
        guide.sections[0].entries[1].key,
        crate::decorations::legend::ResolvedGuideKey::point(alpha_style.clone())
    );
    assert_eq!(guide.sections[0].entries[1].display_label.as_ref(), "Alpha");
}
