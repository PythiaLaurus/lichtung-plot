use super::*;
use crate::decorations::legend::source::{
    FillRole, GuideEncoding, MarkPaintRole, PointRole, RgbBinding, StyleBinding,
};
use crate::{CategoryValues, Color, NumericValues};

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

fn guide(inputs: &crate::layout::interior::PreparedInteriorInputs) -> &PreparedDiscreteGuide {
    inputs
        .discrete_guides
        .get()
        .unwrap()
        .root
        .as_deref()
        .unwrap()
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
    let bars = &inputs.bars.as_ref().unwrap().panels[0].1;
    assert_eq!(bars.series, ["absent", "seen"]);
    assert!(bars
        .bars
        .iter()
        .all(|bar| bar.series_index == Some(1) && bar.hue_index == Some(1)));
    let guide = guide(&inputs);
    assert_eq!(
        (
            guide.counts.entries,
            guide.counts.primary_source_keys,
            guide.counts.source_occurrences
        ),
        (1, 1, 1)
    );
    let entry = &guide.sections[0].entries[0];
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
    let fixed_guide = self::guide(&fixed_inputs);
    let fixed_entry = &fixed_guide.sections[0].entries[0];
    assert_eq!(fixed_entry.key, entry.key);
    assert_ne!(fixed_guide, guide);
    assert_eq!(
        fixed_entry.sources[0].mapping.paint_bindings.fill_outline,
        Some(RgbBinding::Role(MarkPaintRole::FillOutline(
            FillRole::BarBody
        )))
    );
    let merged = PreparedDiscreteGuide::merge(
        [
            inputs.discrete_guides.get().unwrap().root.as_ref().unwrap(),
            fixed_inputs
                .discrete_guides
                .get()
                .unwrap()
                .root
                .as_ref()
                .unwrap(),
        ],
        &[],
        "equal complete Bar keys",
    )
    .unwrap()
    .unwrap();
    assert_eq!(
        (merged.counts.entries, merged.counts.source_occurrences),
        (1, 2)
    );
    assert_eq!(
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        0
    );
}

fn interaction() -> Plot {
    crate::plots::interaction::extraction::source()
        .hue(
            CategoryValues::new(
                "hue",
                ["control", "control", "treated", "treated", "treated"],
            )
            .unwrap(),
        )
        .styles([
            crate::InteractionStyle::new("one")
                .unwrap()
                .with_point_shape(crate::PointShape::TriangleUp)
                .unwrap()
                .with_line_pattern(crate::LinePattern::Dashed)
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
        .with_alpha(211)
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

#[test]
fn prepared_interaction_bindings_retain_exact_point_outlines_and_actual_group_styles() {
    let plot = interaction();
    let ratio = crate::RelativeSpacing::ratio(5, 14).unwrap();
    let outline_paint = Color::rgba(0, 114, 178, 53);
    let theme = interaction_theme(&ratio, outline_paint);
    crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
    let inputs = prepare(&plot, &theme);
    let guide = guide(&inputs);
    assert_eq!(
        (
            guide.counts.entries,
            guide.counts.points,
            guide.counts.lines,
            guide.counts.fills
        ),
        (3, 3, 3, 0)
    );
    for (ordinal, entry) in guide.sections[0].entries.iter().enumerate() {
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
        assert_eq!(point.color.alpha(), 17);
        let outline = point.outline.as_ref().unwrap();
        assert_eq!(outline.color, outline_paint);
        assert_eq!(outline.width_ratio, ratio);
        assert_eq!(outline.width_for_radius(14).unwrap().units(), 5120);
        assert_eq!(entry.key.line.as_ref().unwrap().color.alpha(), 211);
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
