//! Supplied hue titles through the real preparation, census and scoped operands.

use super::scoped::{ScopedGuideInputs, SUPPRESSED_GUIDES};
use super::{GuideSectionIdentity, PreparedGuide};
use crate::orchestrator::registration::PlotRegistration;
use crate::text::{TextRequestContext, TextService};
use crate::{
    Canvas, CanvasLegendPolicy, CategoryValues, Error, GuideTextRole, Labels, LegendComposition,
    NumericValues, Plot, RaincloudGeometry, RaincloudInput, Text, TextAlignment, TextLayout,
    TextTheme, Theme,
};
use std::sync::Arc;

fn geometry() -> RaincloudGeometry {
    crate::plots::categorical_distribution::extraction::supplied()
        .description()
        .raincloud_geometry_input()
        .unwrap()
        .clone()
}

fn supplied() -> Plot<crate::kind::Raincloud> {
    Plot::raincloud(RaincloudInput::geometry(geometry()))
}

fn unhued() -> Plot<crate::kind::Raincloud> {
    let mut input = geometry();
    input.hue_dimension = None;
    input.hue = None;
    input.density_hue = None;
    input.point_hue = None;
    Plot::raincloud(RaincloudInput::geometry(input))
}

fn samples() -> Plot<crate::kind::Raincloud> {
    Plot::raincloud(RaincloudInput::samples(
        CategoryValues::new("category", ["sample"; 5]).unwrap(),
        NumericValues::new("value", [1.0, 2.0, 3.0, 4.0, 5.0]),
    ))
    .series(CategoryValues::new("sample_arm", ["one"; 5]).unwrap())
}

fn with_prepared<T>(
    plot: &Plot,
    theme: &TextRequestContext<'_, '_>,
    inspect: impl FnOnce(Option<Arc<PreparedGuide>>, &[PlotRegistration<'_>]) -> T,
) -> T {
    plot.description()
        .validate_configuration(plot.kind())
        .unwrap();
    let operation = crate::font::canonical::OperationContext::new();
    let (mut registry, _) =
        crate::plots::shared_x::prepare_plot(plot.description(), &operation, theme).unwrap();
    let guide = crate::layout::interior::guide::prepare_discrete_guides(
        plot.description(),
        &mut registry.panels,
        &operation,
        false,
    )
    .unwrap();
    inspect(guide, &registry.finish().unwrap())
}

fn prepare(plot: &Plot, theme: &TextRequestContext<'_, '_>) -> Arc<PreparedGuide> {
    with_prepared(plot, theme, |guide, _| guide.unwrap())
}

fn assert_title_census(
    guide: &PreparedGuide,
    theme: &TextRequestContext<'_, '_>,
    source: &str,
    style: &TextTheme,
) -> crate::layout::PreparedTextCensus {
    use crate::layout::{ComponentAddress, PreparedTextCensus, PreparedTextRole};

    let mut requirements = Vec::new();
    guide
        .append_requirements(&mut requirements, theme, &ComponentAddress::root(), None)
        .unwrap();
    let census = PreparedTextCensus::materialize(requirements, None, None).unwrap();
    let title = census
        .requirements
        .iter()
        .find(|request| request.text_role == PreparedTextRole::LegendTitle)
        .unwrap();
    assert_eq!(title.source, source);
    assert_eq!(&title.stack, style.get_stack());
    assert_eq!(title.weight, style.get_weight());
    let pair = &census.pairs[usize::try_from(title.pair_ordinal).unwrap()];
    assert_eq!(&pair.stack, style.get_stack());
    assert_eq!(pair.weight, style.get_weight());
    census
}

#[test]
fn raincloud_title_instructions_survive_replacement_and_clear_independently_in_census() {
    let mut appearance = Theme::publication().resolve().unwrap();
    let inherited_layout = TextLayout::default().alignment(TextAlignment::Start);
    appearance.text_layouts.guide[0] = Some(inherited_layout.clone());
    let text = TextService::default();
    let theme = TextRequestContext::new(&appearance, &text);
    let style = appearance
        .typography
        .guides
        .legend_title
        .clone()
        .stack(crate::font::canonical::named_stack(&[
            "Authored raincloud hue title census font",
        ]))
        .unwrap();
    let layout = TextLayout::default().alignment(TextAlignment::End);
    let original = supplied()
        .labels(
            Labels::raw()
                .alias("color", "\u{7}unused fallback")
                .alias("changed", "not the authored source")
                .alias("one", "First group"),
        )
        .hue_label(
            Text::new("original")
                .style(style.clone())
                .layout(layout.clone()),
        );
    let retained = original.clone().hue_label("changed");
    let guide = with_prepared(&retained.clone().erase(), &theme, |guide, inputs| {
        let snapshot = ScopedGuideInputs::capture_sizing(&inputs[0], SUPPRESSED_GUIDES).unwrap();
        assert!(snapshot.matches_source(&inputs[0]));
        guide.unwrap()
    });
    assert_eq!(
        guide.sections[0].display_title.as_ref(),
        &Text::new("changed")
            .style(style.clone())
            .layout(layout.clone())
    );
    assert_eq!(
        guide.sections[0].entries()[0].display_label.as_str(),
        "First group"
    );
    assert_eq!(
        guide.sections[0].entries()[0].identity,
        crate::decorations::legend::GuideEntryIdentity::category("color", "one").unwrap()
    );
    assert_title_census(&guide, &theme, "changed", &style);

    let cleared_style = retained.clone().clear_hue_label_style().erase();
    with_prepared(&cleared_style, &theme, |cleared, _| {
        let cleared = cleared.unwrap();
        assert_eq!(
            cleared.sections[0].display_title.as_ref(),
            &Text::new("changed").layout(layout)
        );
        let census = assert_title_census(
            &cleared,
            &theme,
            "changed",
            &appearance.typography.guides.legend_title,
        );
        assert!(census
            .pairs
            .iter()
            .all(|pair| &pair.stack != style.get_stack()));
    });
    let cleared_layout = retained.clear_hue_label_layout().erase();
    with_prepared(&cleared_layout, &theme, |cleared, _| {
        let cleared = cleared.unwrap();
        let title = &cleared.sections[0].display_title;
        assert_eq!(title.as_ref(), &Text::new("changed").style(style.clone()));
        assert_eq!(
            title
                .appearance(theme.guide_text_appearance(GuideTextRole::Title))
                .layout,
            Some(&inherited_layout)
        );
    });
    with_prepared(&original.erase(), &theme, |original, _| {
        assert_eq!(
            original.unwrap().sections[0].display_title.as_str(),
            "original"
        );
    });
}

#[test]
fn unused_raincloud_titles_do_not_create_guides_but_remain_complete_scoped_operands() {
    let appearance = Theme::publication().resolve().unwrap();
    let text = TextService::default();
    let theme = TextRequestContext::new(&appearance, &text);
    let base = samples();
    let (baseline, unset) = with_prepared(&base.clone().erase(), &theme, |guide, inputs| {
        (
            guide.unwrap(),
            ScopedGuideInputs::capture_sizing(&inputs[0], SUPPRESSED_GUIDES).unwrap(),
        )
    });
    assert_eq!(baseline.sections[0].display_title.as_str(), "Sample arm");
    let unused = base.hue_label("");
    let snapshot = with_prepared(&unused.clone().erase(), &theme, |guide, inputs| {
        assert_eq!(guide.as_ref().unwrap(), &baseline);
        assert!(!unset.matches_source(&inputs[0]));
        ScopedGuideInputs::capture_sizing(&inputs[0], SUPPRESSED_GUIDES).unwrap()
    });
    let style = appearance
        .typography
        .guides
        .legend_title
        .clone()
        .stack(crate::font::canonical::named_stack(&[
            "Unconsumed raincloud hue title font",
        ]))
        .unwrap();
    let styled = unused
        .clone()
        .hue_label(Text::new("").style(style.clone()))
        .erase();
    with_prepared(&styled, &theme, |guide, inputs| {
        let guide = guide.unwrap();
        assert_eq!(guide, baseline);
        assert!(!snapshot.matches_source(&inputs[0]));
    });
    let default_layout = unused
        .hue_label(Text::new("").layout(TextLayout::default()))
        .erase();
    with_prepared(&default_layout, &theme, |guide, inputs| {
        assert_eq!(guide.as_ref().unwrap(), &baseline);
        assert!(!snapshot.matches_source(&inputs[0]));
    });
    let unreached = Text::new("unused")
        .style(style)
        .layout(
            TextLayout::default().orientation(crate::TextOrientation::automatic_ticks(vec![
                crate::TextRotation::horizontal(),
            ])),
        );
    with_prepared(
        &unhued().hue_label(unreached.clone()).erase(),
        &theme,
        |guide, _| {
            assert!(guide.is_none());
        },
    );
    let hidden = supplied()
        .hue_label(unreached)
        .legend(crate::LegendPolicy::Hidden);
    let hidden_key = with_prepared(&hidden.clone().erase(), &theme, |guide, inputs| {
        assert!(guide.is_none());
        ScopedGuideInputs::capture_sizing(&inputs[0], SUPPRESSED_GUIDES).unwrap()
    });
    with_prepared(
        &hidden.hue_label("changed unused").erase(),
        &theme,
        |guide, inputs| {
            assert!(guide.is_none());
            assert!(!hidden_key.matches_source(&inputs[0]));
        },
    );
    let empty = prepare(&supplied().hue_label("").erase(), &theme);
    assert!(empty.sections[0].display_title.is_provided());
    assert!(empty.sections[0].display_title.as_str().is_empty());
    assert!(!empty.sections[0].entries().is_empty());
}

fn assert_data_before_fonts(plot: &Plot<crate::kind::Raincloud>, role: &str) {
    let discoveries = crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get);
    let error = plot.preflight().unwrap_err();
    assert!(matches!(&error, Error::Data(_)), "{error}");
    assert!(error.to_string().contains(role), "{error}");
    assert_eq!(
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        discoveries
    );
}

#[test]
fn unused_and_hidden_raincloud_title_controls_keep_data_validation_order() {
    assert_data_before_fonts(
        &samples().hue_label("bad\u{7}title"),
        "raincloud hue guide title",
    );
    assert_data_before_fonts(
        &unhued().hue_label("bad\u{7}title"),
        "raincloud hue guide title",
    );
    assert_data_before_fonts(
        &supplied()
            .hue_label("bad\u{7}title")
            .legend(crate::LegendPolicy::Hidden),
        "raincloud hue guide title",
    );
    assert_data_before_fonts(
        &supplied()
            .x_label("bad\u{8}axis")
            .hue_label("bad\u{7}title"),
        "x-axis title",
    );
}

fn shared(left: Text, right: Text) -> Canvas {
    Canvas::grid(1, 2)
        .unwrap()
        .legend_policy(CanvasLegendPolicy::Shared)
        .legend_composition(LegendComposition::Separate)
        .panel(0, 0, supplied().hue_label(left))
        .unwrap()
        .panel(0, 1, supplied().hue_label(right))
        .unwrap()
}

fn assert_title_conflict_before_fonts(canvas: &Canvas) {
    let discoveries = crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get);
    let error = canvas.preflight().unwrap_err();
    assert!(matches!(&error, Error::Render(_)), "{error}");
    assert!(
        error
            .to_string()
            .contains("requires compatible semantic titles"),
        "{error}"
    );
    assert_eq!(
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        discoveries
    );
}

#[test]
fn shared_raincloud_titles_merge_full_text_and_reject_style_presence_and_raw_field_conflicts() {
    let appearance = Theme::publication().resolve().unwrap();
    let text = TextService::default();
    let theme = TextRequestContext::new(&appearance, &text);
    let style = appearance
        .typography
        .guides
        .legend_title
        .clone()
        .stack(crate::font::canonical::named_stack(&[
            "Unavailable shared raincloud hue title font",
        ]))
        .unwrap();
    let plain = Text::new("Study arm").style(style);
    let full = plain
        .clone()
        .layout(TextLayout::default().alignment(TextAlignment::End));
    let plot = supplied()
        .hue_label(full.clone())
        .legend_composition(LegendComposition::Separate);
    let first = prepare(&plot.clone().erase(), &theme);
    let equal = prepare(&plot.erase(), &theme);
    let merged = PreparedGuide::merge([&first, &equal], &[], "shared raincloud title")
        .unwrap()
        .unwrap();
    assert_eq!(merged.sections[0].display_title.as_ref(), &full);
    assert_eq!(
        (merged.counts.entries, merged.counts.source_occurrences),
        (1, 2)
    );
    assert!(matches!(
        merged.sections[0].identity,
        GuideSectionIdentity::HueColor(_)
    ));

    assert_title_conflict_before_fonts(&shared(
        full.clone(),
        full.clone()
            .style(appearance.typography.guides.legend_title.clone()),
    ));
    assert_title_conflict_before_fonts(&shared(plain.clone(), plain.layout(TextLayout::default())));

    let mut other = geometry();
    other.hue_dimension = Some("other raw field".into());
    let other = Plot::raincloud(RaincloudInput::geometry(other))
        .hue_label(full)
        .legend_composition(LegendComposition::Separate)
        .erase();
    let other = prepare(&other, &theme);
    let error = PreparedGuide::merge([&first, &other], &[], "different raw fields").unwrap_err();
    assert!(matches!(&error, Error::Render(_)), "{error}");
    assert!(
        error
            .to_string()
            .contains("distinct section identities resolve to title"),
        "{error}"
    );
}
