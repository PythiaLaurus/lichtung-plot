use super::*;
use crate::decorations::legend::source::{
    AlphaBinding, AlphaExplanation, FillRole, GuideEncoding, GuideExplanation, MarkPaintRole,
    PointRole, RgbBinding, RgbExplanation, StyleBinding, StyleExplanation,
};
use crate::{CategoryValues, Color, NumericValues, Plot};

fn prepare(
    plot: &Plot,
    theme: &crate::theme::ResolvedTheme,
) -> crate::layout::interior::PreparedInteriorInputs {
    let operation = crate::font::canonical::OperationContext::new();
    let inputs = crate::plots::shared_x::prepare_plot(plot, &operation, Some(theme)).unwrap();
    inputs
        .prepare_discrete_guides(
            plot,
            theme,
            &operation,
            &ComponentAddress::root()
                .child(AddressRole::PlotBody, 0)
                .unwrap(),
            false,
        )
        .unwrap();
    inputs
}

fn guide(inputs: &crate::layout::interior::PreparedInteriorInputs) -> &Arc<PreparedGuide> {
    inputs.discrete_guides.get().unwrap().root.as_ref().unwrap()
}

fn separate_inputs(
    plot: &Plot,
    theme: &crate::theme::ResolvedTheme,
) -> crate::layout::interior::PreparedInteriorInputs {
    prepare(
        &plot
            .clone()
            .with_parent_legend_composition(LegendComposition::Separate),
        theme,
    )
}

fn bar_fill(theme: &mut crate::theme::ResolvedTheme, body: crate::DataFillTheme) {
    let bodies = theme
        .marks
        .fills()
        .bodies()
        .clone()
        .with_bar_body(body)
        .unwrap();
    let fills = theme.marks.fills().clone().with_bodies(bodies).unwrap();
    theme.marks = theme.marks.clone().with_fills(fills).unwrap();
}

#[test]
fn prepared_bar_bindings_distinguish_equal_rgb_outlines_and_preserve_only_observed_keys() {
    let plot = Plot::bar(
        CategoryValues::new("category", ["one", "two"]).unwrap(),
        crate::BarInput::estimates(NumericValues::new("value", [1.0, 2.0])),
    )
    .series(CategoryValues::new("cohort", ["seen", "seen"]).unwrap())
    .erase()
    .with_global_group_levels(vec!["absent".into(), "seen".into()])
    .with_global_hue_levels(vec!["absent".into(), "seen".into()]);
    let mut theme = crate::Theme::publication().resolve().unwrap();
    let fill = crate::DataFillTheme::new(
        17,
        Some(crate::DataOutlineTheme::mapped(2.375, 211).unwrap()),
    )
    .unwrap();
    bar_fill(&mut theme, fill);
    crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
    let inputs = prepare(&plot, &theme);
    let projected_mapped =
        PreparedGuide::prepare_for_test(&plot, &theme, LegendComposition::Separate)
            .unwrap()
            .unwrap();
    let projected_inputs = separate_inputs(&plot, &theme);
    assert_eq!(self::guide(&projected_inputs), &projected_mapped);
    let bars = &inputs.bars.as_ref().unwrap().panels[0].1;
    assert_eq!(bars.series, ["absent", "seen"]);
    assert!(bars
        .bars
        .iter()
        .all(|bar| bar.series_index == Some(1) && bar.hue_index == Some(1)));
    let guide = guide(&inputs);
    assert_eq!(guide.counts.entries, 1);
    assert_eq!(guide.counts.primary_source_keys, 1);
    assert_eq!(guide.counts.source_occurrences, 1);
    let entry = &guide.sections[0].entries()[0];
    let source = &entry.sources[0];
    let GuideEntryIdentity::Category { field, value } = &entry.identity else {
        panic!("raw category")
    };
    let mapped = RgbBinding::Mapped {
        field: field.clone(),
        value: value.clone(),
    };
    assert_eq!(
        source.mapping.paint_bindings.fill_paint,
        Some(mapped.clone())
    );
    assert_eq!(source.mapping.paint_bindings.fill_outline, Some(mapped));
    assert!(source.mapping.paint_bindings.point_fill.is_none());
    assert!(source.mapping.style_bindings.line_pattern.is_none());
    let fill = entry.key.fill.as_ref().unwrap();
    assert_eq!(fill.color, Color::rgba(213, 94, 0, 17));
    assert_eq!(fill.stroke, Color::rgba(213, 94, 0, 211));
    assert_eq!(fill.stroke_width.to_bits(), 2.375_f32.to_bits());
    let fixed = crate::DataOutlineTheme::fixed(
        crate::OutlineTheme::new(Color::rgba(213, 94, 0, 211), 2.375).unwrap(),
    );
    let body = theme
        .marks
        .fills()
        .bodies()
        .bar_body()
        .clone()
        .with_outline(Some(fixed))
        .unwrap();
    bar_fill(&mut theme, body);
    let fixed_inputs = prepare(&plot, &theme);
    let projected_fixed =
        PreparedGuide::prepare_for_test(&plot, &theme, LegendComposition::Separate)
            .unwrap()
            .unwrap();
    let fixed_guide = self::guide(&fixed_inputs);
    let fixed_entry = &fixed_guide.sections[0].entries()[0];
    assert_eq!(fixed_entry.key, entry.key);
    assert_ne!(fixed_guide, guide);
    assert_eq!(
        fixed_entry.sources[0].mapping.paint_bindings.fill_outline,
        Some(RgbBinding::Role(MarkPaintRole::FillOutline(
            FillRole::BarBody
        )))
    );
    let merged = PreparedGuide::merge([guide, fixed_guide], &[], "equal complete Bar keys")
        .unwrap()
        .unwrap();
    assert_eq!(
        (merged.counts.entries, merged.counts.source_occurrences),
        (1, 2)
    );
    assert_outline_explanations(&projected_mapped, &projected_fixed, entry, fixed_entry);
    assert_bar_alpha_context(&plot, &mut theme);
    assert_eq!(
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        0
    );
}

fn assert_bar_alpha_context(plot: &Plot, theme: &mut crate::theme::ResolvedTheme) {
    use crate::alpha::{AlphaEncoding, AlphaTarget, AlphaUnitKind, AlphaValues};
    let body = theme
        .marks
        .fills()
        .bodies()
        .bar_body()
        .clone()
        .with_fill_alpha(255)
        .unwrap();
    bar_fill(theme, body);
    let mapped = plot.clone().with_alpha(
        AlphaTarget::Fill,
        AlphaEncoding::keyed(
            [
                CategoryValues::new("category", ["one", "two"]).unwrap(),
                CategoryValues::new("cohort", ["seen", "seen"]).unwrap(),
            ],
            AlphaValues::new("confidence", [0, 128]).unwrap(),
        )
        .unwrap(),
    );
    for composition in [LegendComposition::Combined, LegendComposition::Separate] {
        let constant = PreparedGuide::prepare_for_test(plot, theme, composition)
            .unwrap()
            .unwrap();
        let data = PreparedGuide::prepare_for_test(&mapped, theme, composition)
            .unwrap()
            .unwrap();
        assert_eq!(data.counts.sections, 1);
        assert_eq!(data.counts.entries, 1);
        let constant_key = &constant.sections[0].entries()[0].key;
        let entry = &data.sections[0].entries()[0];
        let constant_fill = constant_key.fill.as_ref().unwrap();
        let data_fill = entry.key.fill.as_ref().unwrap();
        assert_eq!(constant_fill.color, data_fill.color);
        assert_eq!(constant_fill.stroke, data_fill.stroke);
        assert_eq!(
            constant_fill.stroke_width.to_bits(),
            data_fill.stroke_width.to_bits()
        );
        assert_ne!(constant_key, &entry.key);
        assert_eq!(entry.key.fill.as_ref().unwrap().color.get_alpha(), 255);
        assert_eq!(entry.key.fill.as_ref().unwrap().stroke.get_alpha(), 211);
        let alpha = match &entry.key.explanation {
            GuideExplanation::Complete(alpha) => alpha,
            GuideExplanation::Projected(mask) => &mask.alpha,
        };
        assert_eq!(alpha.fill_paint, Some(AlphaExplanation::Neutralized));
        assert_eq!(alpha.fill_outline, Some(AlphaExplanation::Constant));
        let Some(AlphaBinding::Data {
            target: AlphaTarget::Fill,
            unit: AlphaUnitKind::Aggregate,
            input,
        }) = &entry.sources[0].mapping.alpha_bindings.fill_paint
        else {
            panic!("explicit aggregate fill binding")
        };
        assert_eq!(input, mapped.alpha_inputs().get(AlphaTarget::Fill));
        assert!(
            PreparedGuide::merge([&constant, &data], &[], "alpha-context compatibility")
                .unwrap_err()
                .to_string()
                .contains("explanation mask")
        );
    }
}

fn assert_outline_explanations(
    mapped: &Arc<PreparedGuide>,
    fixed: &Arc<PreparedGuide>,
    complete_mapped: &PreparedGuideEntry,
    complete_fixed: &PreparedGuideEntry,
) {
    let GuideExplanation::Projected(mapped_mask) = &mapped.sections[0].entries()[0].key.explanation
    else {
        panic!("mapped color explanation")
    };
    let GuideExplanation::Projected(fixed_mask) = &fixed.sections[0].entries()[0].key.explanation
    else {
        panic!("fixed color explanation")
    };
    assert!(matches!(
        mapped.sections[0].identity,
        GuideSectionIdentity::HueColor(_)
    ));
    assert_eq!(
        mapped_mask.fill_outline,
        Some(RgbExplanation::ExplainedColor)
    );
    assert_eq!(fixed_mask.fill_outline, Some(RgbExplanation::Constant));
    assert!(
        PreparedGuide::merge([mapped, fixed], &[], "projected Bar outlines")
            .unwrap_err()
            .to_string()
            .contains("explanation mask")
    );
    let neutral = Some(crate::OpaqueColor::rgb(40, 50, 60));
    let mapped_group = super::projection::project(
        &complete_mapped.key,
        &complete_mapped.sources[0].mapping,
        Projection::GroupStyle,
        neutral,
    )
    .unwrap();
    let fixed_group = super::projection::project(
        &complete_fixed.key,
        &complete_fixed.sources[0].mapping,
        Projection::GroupStyle,
        neutral,
    )
    .unwrap();
    assert_eq!(
        mapped_group.fill.as_ref().unwrap().color,
        Color::rgba(40, 50, 60, 17)
    );
    assert_eq!(
        mapped_group.fill.as_ref().unwrap().stroke,
        Color::rgba(40, 50, 60, 211)
    );
    assert_eq!(
        fixed_group.fill.as_ref().unwrap().stroke,
        complete_fixed.key.fill.as_ref().unwrap().stroke
    );
}

fn interaction_input() -> Plot<crate::kind::Interaction> {
    crate::plots::interaction::extraction::source().hue(
        CategoryValues::new(
            "hue",
            ["control", "control", "treated", "treated", "treated"],
        )
        .unwrap(),
    )
}

fn interaction() -> Plot {
    interaction_input()
        .styles([
            crate::InteractionStyle::new("one")
                .unwrap()
                .point_shape(crate::PointShape::TriangleUp)
                .unwrap()
                .line_pattern(crate::LinePattern::Dashed)
                .unwrap(),
            crate::InteractionStyle::new("two").unwrap(),
        ])
        .erase()
}

fn interaction_theme(
    ratio: &crate::RelativeSpacing,
    outline_paint: Color,
) -> crate::theme::ResolvedTheme {
    let mut theme = crate::Theme::publication().resolve().unwrap();
    let outline = crate::PointOutlineTheme::new(outline_paint, ratio.clone()).unwrap();
    let primary = crate::PointTheme::new(17, crate::PointShape::TriangleUp, Some(outline)).unwrap();
    let radii = theme
        .marks
        .points()
        .radii()
        .clone()
        .with_interaction(14)
        .unwrap();
    let points = theme
        .marks
        .points()
        .clone()
        .with_primary(primary)
        .unwrap()
        .with_radii(radii)
        .unwrap();
    theme.marks = theme.marks.clone().with_points(points).unwrap();
    let line = theme
        .marks
        .data_lines()
        .interaction_connector()
        .clone()
        .alpha(211)
        .unwrap();
    let lines = theme
        .marks
        .data_lines()
        .clone()
        .with_interaction_connector(line)
        .unwrap();
    theme.marks = theme.marks.clone().with_data_lines(lines).unwrap();
    theme
}

fn assert_complete_interaction(
    guide: &PreparedGuide,
    ratio: &crate::RelativeSpacing,
    outline_paint: Color,
) {
    assert_eq!(
        (
            guide.counts.entries,
            guide.counts.points,
            guide.counts.lines,
            guide.counts.fills
        ),
        (3, 3, 3, 0)
    );
    for (ordinal, entry) in guide.sections[0].entries().iter().enumerate() {
        let GuideEntryIdentity::Pair {
            group_field,
            group_value,
            hue_field,
            hue_value,
        } = &entry.identity
        else {
            panic!("observed pair")
        };
        let source = &entry.sources[0];
        assert_eq!(source.producer, crate::PlotKind::Interaction);
        assert_eq!(source.source_ordinal, u32::try_from(ordinal).unwrap());
        assert_eq!(
            source.mapping.encoding,
            GuideEncoding::IndependentPair {
                group: group_field.clone(),
                hue: hue_field.clone()
            }
        );
        assert_eq!(source.projection, Projection::Complete);
        let color = RgbBinding::Mapped {
            field: hue_field.clone(),
            value: hue_value.clone(),
        };
        assert_eq!(
            source.mapping.paint_bindings.point_fill,
            Some(color.clone())
        );
        assert_eq!(source.mapping.paint_bindings.line_stroke, Some(color));
        assert_eq!(
            source.mapping.paint_bindings.point_outline,
            Some(RgbBinding::Role(MarkPaintRole::PointOutline(
                PointRole::Primary
            )))
        );
        let style = StyleBinding::InteractionGroup {
            field: group_field.clone(),
            value: group_value.clone(),
        };
        assert_eq!(
            source.mapping.style_bindings.point_shape,
            Some(style.clone())
        );
        assert_eq!(source.mapping.style_bindings.line_pattern, Some(style));
        let point = entry.key.point.as_ref().unwrap();
        assert_eq!(point.radius, 14);
        assert_eq!(point.color.get_alpha(), 17);
        let outline = point.outline.as_ref().unwrap();
        assert_eq!(outline.color, outline_paint);
        assert_eq!(&outline.width_ratio, ratio);
        assert_eq!(outline.width_for_radius(14).unwrap().units(), 5120);
        assert_eq!(entry.key.line.as_ref().unwrap().color.get_alpha(), 211);
        if group_value.as_str() == "one" {
            assert_eq!(point.shape, crate::PointShape::TriangleUp);
            assert_eq!(
                entry.key.line.as_ref().unwrap().pattern,
                crate::StrokePattern::LongDash
            );
        } else {
            assert_eq!(point.shape, crate::PointShape::Circle);
            assert_eq!(
                entry.key.line.as_ref().unwrap().pattern,
                crate::StrokePattern::Solid
            );
        }
    }
}

fn assert_projected_interaction(guide: &PreparedGuide, outline_paint: Color) {
    assert_eq!(guide.counts.sections, 2);
    assert_eq!(
        (
            guide.counts.entries,
            guide.counts.points,
            guide.counts.lines,
            guide.counts.fills
        ),
        (4, 4, 4, 0)
    );
    assert_eq!(
        (
            guide.counts.primary_source_keys,
            guide.counts.observed_pair_source_keys,
            guide.counts.source_occurrences
        ),
        (3, 3, 6)
    );
    assert!(matches!(
        guide.sections[0].identity,
        GuideSectionIdentity::GroupStyle(_)
    ));
    assert!(matches!(
        guide.sections[1].identity,
        GuideSectionIdentity::HueColor(_)
    ));
    for entry in guide.sections[0].entries() {
        let point = entry.key.point.as_ref().unwrap();
        let line = entry.key.line.as_ref().unwrap();
        assert_eq!(point.color, Color::rgba(40, 50, 60, 17));
        assert_eq!(line.color, Color::rgba(40, 50, 60, 211));
        assert_eq!(point.outline.as_ref().unwrap().color, outline_paint);
        assert_eq!(point.radius, 14);
        let GuideEntryIdentity::Category { value, .. } = &entry.identity else {
            panic!("group category")
        };
        let (shape, pattern) = if value.as_str() == "one" {
            (
                crate::PointShape::TriangleUp,
                crate::StrokePattern::LongDash,
            )
        } else {
            (crate::PointShape::Circle, crate::StrokePattern::Solid)
        };
        assert_eq!((point.shape, line.pattern), (shape, pattern));
        let GuideExplanation::Projected(mask) = &entry.key.explanation else {
            panic!("group explanation")
        };
        assert_eq!(mask.point_fill, Some(RgbExplanation::NeutralizedColor));
        assert_eq!(mask.point_outline, Some(RgbExplanation::Constant));
        assert_eq!(mask.point_shape, Some(StyleExplanation::ExplainedGroup));
        assert_eq!(mask.line_pattern, Some(StyleExplanation::ExplainedGroup));
        assert!(entry
            .sources
            .iter()
            .all(|source| source.group_neutral_rgb == Some(crate::OpaqueColor::rgb(40, 50, 60))));
    }
    for entry in guide.sections[1].entries() {
        let point = entry.key.point.as_ref().unwrap();
        let line = entry.key.line.as_ref().unwrap();
        assert_eq!(
            (point.shape, line.pattern),
            (crate::PointShape::Circle, crate::StrokePattern::Solid)
        );
        assert_eq!(point.outline.as_ref().unwrap().color, outline_paint);
        assert_eq!((point.color.get_alpha(), line.color.get_alpha()), (17, 211));
        let GuideEntryIdentity::Category { value, .. } = &entry.identity else {
            panic!("hue category")
        };
        let color = if value.as_str() == "control" {
            (0, 114, 178)
        } else {
            (213, 94, 0)
        };
        assert_eq!(
            (point.color.red(), point.color.green(), point.color.blue()),
            color
        );
        let GuideExplanation::Projected(mask) = &entry.key.explanation else {
            panic!("hue explanation")
        };
        assert_eq!(mask.point_fill, Some(RgbExplanation::ExplainedColor));
        assert_eq!(mask.point_outline, Some(RgbExplanation::Constant));
        assert_eq!(mask.point_shape, Some(StyleExplanation::NeutralizedGroup));
        assert_eq!(mask.line_pattern, Some(StyleExplanation::NeutralizedGroup));
        assert!(entry
            .sources
            .iter()
            .all(|source| source.group_neutral_rgb.is_none()));
    }
}

#[test]
fn prepared_interaction_bindings_retain_exact_point_outlines_and_actual_group_styles() {
    let plot = interaction();
    let ratio = crate::RelativeSpacing::ratio(5, 14).unwrap();
    let outline_paint = Color::rgba(0, 114, 178, 53);
    let mut theme = interaction_theme(&ratio, outline_paint);
    theme.typography.guides.legend_label = theme
        .typography
        .guides
        .legend_label
        .clone()
        .color(Color::rgba(40, 50, 60, 0))
        .unwrap();
    crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
    let inputs = prepare(&plot, &theme);
    assert_complete_interaction(guide(&inputs), &ratio, outline_paint);
    let projected = PreparedGuide::prepare_for_test(&plot, &theme, LegendComposition::Separate)
        .unwrap()
        .unwrap();
    let projected_inputs = separate_inputs(&plot, &theme);
    assert_eq!(guide(&projected_inputs), &projected);
    assert_projected_interaction(&projected, outline_paint);
    assert_interaction_alpha_context(&plot, &theme, outline_paint);
    let unconfigured = interaction_input().legend_composition(LegendComposition::Separate);
    let error = unconfigured.clone().preflight().unwrap_err();
    assert!(error
        .to_string()
        .contains("unsupported discrete guide factorization"));
    assert!(PreparedGuide::prepare_for_test(
        &unconfigured.legend(crate::LegendPolicy::Hidden).erase(),
        &theme,
        LegendComposition::Separate
    )
    .unwrap()
    .is_none());
    let collapsed = crate::plots::interaction::extraction::source()
        .hue(CategoryValues::new("group", ["one", "one", "one", "two", "two"]).unwrap())
        .erase();
    let joint = PreparedGuide::prepare_for_test(&collapsed, &theme, LegendComposition::Separate)
        .unwrap()
        .unwrap();
    assert_eq!(joint.counts.sections, 1);
    assert!(matches!(
        joint.sections[0].identity,
        GuideSectionIdentity::Joint(_)
    ));
    assert!(joint.sections[0]
        .entries()
        .iter()
        .all(|entry| matches!(entry.key.explanation, GuideExplanation::Complete(_))));
    let ambiguous = interaction().labels(
        crate::Labels::raw()
            .alias("group", "Shared title")
            .alias("hue", "Shared title"),
    );
    assert!(
        PreparedGuide::prepare_for_test(&ambiguous, &theme, LegendComposition::Separate)
            .unwrap_err()
            .to_string()
            .contains("distinct section identities")
    );
    let same_label = interaction().labels(
        crate::Labels::raw()
            .alias("one", "Equal label")
            .alias("control", "Equal label"),
    );
    let distinct_sections =
        PreparedGuide::prepare_for_test(&same_label, &theme, LegendComposition::Separate)
            .unwrap()
            .unwrap();
    assert_eq!(
        distinct_sections.sections[0].entries()[0].display_label,
        distinct_sections.sections[1].entries()[0].display_label
    );
    assert_eq!(
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        0
    );
    assert!(crate::PointOutlineTheme::new(
        outline_paint,
        crate::RelativeSpacing::ratio(0, 1).unwrap()
    )
    .is_err());
    for ratio in [f64::from_bits(1), f64::MAX] {
        let outline = crate::PointOutlineTheme::new(
            outline_paint,
            crate::RelativeSpacing::new(ratio).unwrap(),
        )
        .unwrap();
        assert!(outline.backend_style().width_for_radius(1).is_err());
    }
}

fn assert_interaction_alpha_context(
    plot: &Plot,
    theme: &crate::theme::ResolvedTheme,
    outline: Color,
) {
    use crate::alpha::{AlphaEncoding, AlphaTarget, AlphaValues};
    let mapped = plot
        .clone()
        .with_alpha(
            AlphaTarget::Point,
            AlphaEncoding::observations(
                AlphaValues::new("confidence", [0, 17, 64, 128, 230]).unwrap(),
            ),
        )
        .with_alpha(
            AlphaTarget::Line,
            AlphaEncoding::keyed(
                [
                    CategoryValues::new("group", ["one", "one", "two"]).unwrap(),
                    CategoryValues::new("hue", ["control", "treated", "treated"]).unwrap(),
                ],
                AlphaValues::new("trace confidence", [0, 64, 128]).unwrap(),
            )
            .unwrap(),
        );
    for composition in [LegendComposition::Combined, LegendComposition::Separate] {
        let guide = PreparedGuide::prepare_for_test(&mapped, theme, composition)
            .unwrap()
            .unwrap();
        assert_eq!(
            guide.counts.sections,
            if composition == LegendComposition::Combined {
                1
            } else {
                2
            }
        );
        for section in guide.sections.iter() {
            for entry in section.entries() {
                let point = entry.key.point.as_ref().unwrap();
                let line = entry.key.line.as_ref().unwrap();
                assert_eq!(point.color.get_alpha(), 255);
                assert_eq!(line.color.get_alpha(), 255);
                assert_eq!(point.outline.as_ref().unwrap().color, outline);
                let alpha = match &entry.key.explanation {
                    GuideExplanation::Complete(alpha) => alpha,
                    GuideExplanation::Projected(mask) => &mask.alpha,
                };
                assert_eq!(alpha.point_fill, Some(AlphaExplanation::Neutralized));
                assert_eq!(alpha.line_stroke, Some(AlphaExplanation::Neutralized));
                assert_eq!(alpha.point_outline, Some(AlphaExplanation::Constant));
                if matches!(section.identity, GuideSectionIdentity::GroupStyle(_)) {
                    assert_eq!(point.color, Color::rgb(40, 50, 60));
                    assert_eq!(line.color, Color::rgb(40, 50, 60));
                }
            }
        }
    }
}
