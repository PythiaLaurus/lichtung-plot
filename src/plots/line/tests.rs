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
    .with_alpha(
        crate::alpha::AlphaTarget::Point,
        crate::alpha::AlphaEncoding::observations(
            crate::alpha::AlphaValues::new("confidence", [0, 0]).unwrap(),
        ),
    )
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
#[expect(
    clippy::too_many_lines,
    reason = "one source proves independent three-target marks and their common declared explanation plan"
)]
fn primary_point_alpha_follows_sorted_line_rows_and_keeps_supplemental_paint_independent() {
    use crate::decorations::legend::LegendCall;

    let source = Plot::line(
        crate::NumericValues::new("x", [2.0, 0.0, 1.0]),
        crate::NumericValues::new("y", [2.0, 0.0, 1.0]),
    )
    .hue(crate::CategoryValues::new("cohort", ["seen"; 3]).unwrap())
    .band(crate::Bounds::new(
        crate::NumericValues::new("lower", [1.5, -0.5, 0.5]),
        crate::NumericValues::new("upper", [2.5, 0.5, 1.5]),
    ))
    .with_alpha(
        crate::alpha::AlphaTarget::Fill,
        crate::alpha::AlphaEncoding::keyed(
            [crate::CategoryValues::new("cohort", ["seen"]).unwrap()],
            crate::alpha::AlphaValues::threshold(
                crate::NumericValues::new("band confidence", [0.0]),
                [],
                [0],
            )
            .unwrap(),
        )
        .unwrap(),
    )
    .with_alpha(
        crate::alpha::AlphaTarget::Point,
        crate::alpha::AlphaEncoding::observations(
            crate::alpha::AlphaValues::linear(
                crate::NumericValues::new("confidence", [230.0, 17.0, 90.0]),
                (0.0, 255.0),
                (0, 255),
            )
            .unwrap(),
        ),
    )
    .point_layer(labeled_layer(0.5, "Layer").alpha(61))
    .with_alpha(
        crate::alpha::AlphaTarget::Line,
        crate::alpha::AlphaEncoding::keyed(
            [crate::CategoryValues::new("cohort", ["unused", "seen"]).unwrap()],
            crate::alpha::AlphaValues::threshold(
                crate::NumericValues::new("series confidence", [0.0, 1.0]),
                [0.5],
                [0, 128],
            )
            .unwrap(),
        )
        .unwrap(),
    )
    .x_limits(-1.0, 3.0)
    .y_limits(-1.0, 3.0)
    .erase();
    let theme = crate::Theme::publication().resolve().unwrap();
    let chart = crate::plots::shared_x::prepare_panel_for_test(&source, &theme).unwrap();
    let selected = chart
        .measure_selected_interior(PlotRenderContext::new(
            chart.data(),
            crate::layout::PanelBox::new(20.0, 600.0, 20.0, 420.0),
            &theme,
        ))
        .unwrap();
    let crate::layout::interior::MarkPayload::Line(payload) =
        &selected.marks.as_ref().unwrap().payload
    else {
        panic!("Line payload")
    };
    let alphas = payload
        .marks
        .iter()
        .filter_map(|mark| {
            if let LineMark::Points(_, style) = mark {
                Some(style.color.alpha())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(alphas, [17, 90, 230]);
    assert_eq!(payload.layers[0].1.color.alpha(), 61);
    assert_eq!(payload.layers[0].2[0].plan.style.paint.alpha(), 61);
    let ribbons = payload
        .marks
        .iter()
        .filter_map(|mark| match mark {
            LineMark::Ribbon(points, fill) => Some((points.len(), fill.color.alpha(), fill.stroke)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        ribbons,
        [(
            6,
            0,
            theme
                .marks
                .fills()
                .bands()
                .line_band()
                .backend_style((0, 0, 0))
                .stroke
        )]
    );
    assert_eq!(
        payload
            .marks
            .iter()
            .filter_map(|mark| if let LineMark::Line(_, style) = mark {
                Some(style.color.alpha())
            } else {
                None
            })
            .collect::<Vec<_>>(),
        [128]
    );
    let descriptor = selected.discrete_guide.as_ref().unwrap();
    assert_eq!(
        descriptor
            .sections
            .iter()
            .map(|section| section.entries().len())
            .collect::<Vec<_>>(),
        [2, 5, 1, 1]
    );
    let measured =
        crate::layout::fit::measured(crate::decorations::legend::Legend::measure_assigned(
            std::sync::Arc::clone(descriptor),
            (900.0, 900.0),
            false,
            false,
            &theme,
            &mut false,
            &crate::layout::fit::FitScope::root(),
        ))
        .unwrap();
    let (x, y) = measured.box_at(0.0, 0.0).anchor();
    let draw = measured.select(x, y).unwrap();
    assert_eq!(
        draw.calls
            .iter()
            .filter(|call| matches!(call, LegendCall::Background(..)))
            .count(),
        1
    );
    assert_eq!(
        draw.calls
            .iter()
            .filter_map(|call| match call {
                LegendCall::Point(_, style) => Some(style.color.alpha()),
                _ => None,
            })
            .collect::<Vec<_>>(),
        [255, 61, 0, 64, 128, 191, 255]
    );
    assert_eq!(
        draw.calls
            .iter()
            .filter_map(|call| match call {
                LegendCall::Line(_, style) => Some(style.color.alpha()),
                _ => None,
            })
            .collect::<Vec<_>>(),
        [255, 128]
    );
    assert_eq!(
        draw.calls
            .iter()
            .filter_map(|call| match call {
                LegendCall::Fill(_, style) => Some(style.color.alpha()),
                _ => None,
            })
            .collect::<Vec<_>>(),
        [255, 0]
    );
}

#[test]
fn path_point_alpha_retains_source_vertex_order_without_regrouping_by_byte() {
    let source = Plot::line_path(
        crate::NumericValues::new("x", [2.0, 0.0, 1.0]),
        crate::NumericValues::new("y", [2.0, 0.0, 1.0]),
        crate::CategoryValues::new("path", ["A"; 3]).unwrap(),
    )
    .markers(true)
    .with_alpha(
        crate::alpha::AlphaTarget::Line,
        crate::alpha::AlphaEncoding::keyed(
            [crate::CategoryValues::new("path", ["A"]).unwrap()],
            crate::alpha::AlphaValues::new("path confidence", [64]).unwrap(),
        )
        .unwrap(),
    )
    .with_alpha(
        crate::alpha::AlphaTarget::Point,
        crate::alpha::AlphaEncoding::observations(
            crate::alpha::AlphaValues::new("confidence", [230, 17, 90]).unwrap(),
        ),
    )
    .x_limits(-1.0, 3.0)
    .y_limits(-1.0, 3.0)
    .erase();
    let prepared = PreparedLine::new(&source, source.data(), None).unwrap();
    assert_eq!(prepared.groups[0].line_alpha.unwrap().alpha, 64);
    assert_eq!(
        prepared.groups[0]
            .points
            .iter()
            .map(|point| point.alpha.slot)
            .collect::<Vec<_>>(),
        [0, 1, 2]
    );
    let theme = crate::Theme::publication().resolve().unwrap();
    let chart = crate::plots::shared_x::prepare_panel_for_test(&source, &theme).unwrap();
    let selected = chart
        .measure_selected_interior(PlotRenderContext::new(
            chart.data(),
            crate::layout::PanelBox::new(20.0, 600.0, 20.0, 420.0),
            &theme,
        ))
        .unwrap();
    let crate::layout::interior::MarkPayload::Line(payload) =
        &selected.marks.as_ref().unwrap().payload
    else {
        panic!("LinePath payload")
    };
    assert_eq!(
        payload
            .marks
            .iter()
            .filter_map(|mark| if let LineMark::Points(_, style) = mark {
                Some(style.color.alpha())
            } else {
                None
            })
            .collect::<Vec<_>>(),
        [230, 17, 90]
    );
    assert_eq!(
        payload
            .marks
            .iter()
            .filter_map(|mark| if let LineMark::Line(_, style) = mark {
                Some(style.color.alpha())
            } else {
                None
            })
            .collect::<Vec<_>>(),
        [64]
    );
}

#[test]
fn exact_supplemental_color_and_alpha_edits_freeze_identically_in_either_order() {
    let theme = crate::Theme::publication().resolve().unwrap();
    let paint = crate::Color::rgba(13, 29, 47, 53);
    let layer = labeled_layer(1.0, "Exact").radius(9).unwrap();
    let color_then_alpha = measure_layer(layer.clone().color(paint).alpha(131), &theme);
    let alpha_then_color = measure_layer(layer.alpha(131).color(paint), &theme);
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
#[expect(
    clippy::too_many_lines,
    reason = "one layer contract compares complete paints, annotations and source label context under both compositions"
)]
fn supplemental_defaults_and_explicit_alpha_keep_label_paint_independent_and_guides_truthful() {
    let automatic = labeled_layer(1.0, "Auto");
    let alpha = labeled_layer(1.5, "Alpha").alpha(128).radius(9).unwrap();
    let source = Plot::line(
        crate::NumericValues::new("x", [0.0, 2.0]),
        crate::NumericValues::new("y", [0.0, 2.0]),
    )
    .hue(crate::CategoryValues::new("cohort", ["seen", "seen"]).unwrap())
    .labels(crate::Labels::raw().alias("Alpha", "Legend alpha"))
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
    assert_eq!(guide.counts.entries, 3);
    assert_eq!(
        guide.sections[0].entries()[2].identity,
        crate::decorations::legend::GuideEntryIdentity::supplemental("Alpha").unwrap()
    );
    assert_eq!(
        guide.sections[0].entries()[2].key,
        crate::decorations::legend::ResolvedGuideKey::point(alpha_style.clone())
    );
    assert_eq!(alpha_text[0].plan.source, "Alpha");
    assert_eq!(
        guide.sections[0].entries()[2].display_label.as_ref(),
        "Legend alpha"
    );
    let separate = crate::decorations::legend::discrete::PreparedGuide::prepare_for_test(
        &source,
        &theme,
        crate::LegendComposition::Separate,
    )
    .unwrap()
    .unwrap();
    assert!(matches!(
        separate.sections[1].identity,
        crate::decorations::legend::discrete::GuideSectionIdentity::SupplementalSeries
    ));
    assert_eq!(
        separate.sections[1].entries()[1].display_label.as_ref(),
        "Legend alpha"
    );
    assert_eq!(
        separate.sections[1].entries()[1].key,
        guide.sections[0].entries()[2].key
    );
}
