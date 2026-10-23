use super::*;
use crate::font::canonical::OperationContext;

fn stop(offset: f64, red: u8) -> ColorStop {
    ColorStop::new(
        offset,
        OpaqueColor::rgb(red, red.wrapping_add(3), red.wrapping_add(7)),
    )
    .unwrap()
}

#[test]
fn map_retains_exact_raw_keys_order_and_opaque_colors_without_normalization() {
    let rgb = OpaqueColor::rgb(13, 29, 47);
    let map = CategoricalColorMap::new([
        ("A  B".into(), rgb),
        ("a b".into(), rgb),
        ("é".into(), rgb),
        ("e\u{301}".into(), rgb),
    ])
    .unwrap();
    let entries = map.entries().collect::<Vec<_>>();
    assert_eq!(
        entries,
        [("A  B", rgb), ("a b", rgb), ("é", rgb), ("e\u{301}", rgb)]
    );
    assert_eq!(map.color("A  B"), Some(rgb));
    assert_eq!(map.color("é"), Some(rgb));
    assert_eq!(map.color("A B"), None);
    assert_eq!(map.color("unused"), None);
    assert!(CategoricalColorMap::new([]).is_err());
    for key in ["", " leading", "trailing ", "control\n", "\u{0000}"] {
        assert!(CategoricalColorMap::new([(key.into(), rgb)]).is_err());
    }
    assert!(
        CategoricalColorMap::new([("duplicate".into(), rgb), ("duplicate".into(), rgb)]).is_err()
    );
    let copied = map.clone();
    assert_eq!(
        copied.entries().next().unwrap().0.as_ptr(),
        map.entries().next().unwrap().0.as_ptr()
    );
}

#[test]
fn domains_and_continuous_edits_validate_kind_center_and_complete_result() {
    let constant = ColorDomain::exact(-0.0, 0.0).unwrap();
    assert_eq!(constant.minimum().to_bits(), 0);
    assert_eq!(constant.maximum().to_bits(), 0);
    assert_eq!(constant, ColorDomain::exact(0.0, -0.0).unwrap());
    for (minimum, maximum) in [
        (2.0, 1.0),
        (f64::NAN, 1.0),
        (0.0, f64::INFINITY),
        (f64::NEG_INFINITY, 1.0),
    ] {
        assert!(ColorDomain::exact(minimum, maximum).is_err());
    }
    let extreme = ColorDomain::exact(-f64::MAX, f64::MAX).unwrap();
    assert_eq!(extreme.minimum().to_bits(), (-f64::MAX).to_bits());
    assert_eq!(extreme.maximum().to_bits(), f64::MAX.to_bits());
    let defaults = test_defaults();
    let source = ContinuousColorScale::diverging(
        -0.0,
        Some(defaults.diverging().clone()),
        Some(ColorDomain::exact(-2.0, 3.0).unwrap()),
    )
    .unwrap();
    assert_eq!(source.kind(), ContinuousScaleKind::Diverging);
    assert_eq!(source.center().unwrap().to_bits(), 0);
    assert_eq!(
        source.domain().unwrap().minimum().to_bits(),
        (-2.0_f64).to_bits()
    );
    assert_eq!(
        source.domain().unwrap().maximum().to_bits(),
        3.0_f64.to_bits()
    );
    assert_eq!(source.diverging_gradient(), Some(defaults.diverging()));
    assert!(source.sequential_gradient().is_none());
    let shifted = source.clone().with_center(1.25).unwrap();
    assert_eq!(shifted.center().unwrap().to_bits(), 1.25_f64.to_bits());
    assert_eq!(shifted.domain(), source.domain());
    assert_eq!(shifted.diverging_gradient(), source.diverging_gradient());
    assert!(source.clone().with_center(f64::NAN).is_err());
    assert!(source.clone().with_center(4.0).is_err());
    assert!(source
        .clone()
        .with_domain(Some(ColorDomain::exact(1.0, 2.0).unwrap()))
        .is_err());
    let automatic = source
        .clone()
        .with_domain(None)
        .unwrap()
        .with_diverging_gradient(None)
        .unwrap();
    assert!(automatic.domain().is_none());
    assert!(automatic.diverging_gradient().is_none());
    assert_eq!(automatic.center(), source.center());
    assert!(source.clone().with_sequential_gradient(None).is_err());
    let sequential = ContinuousColorScale::sequential(
        Some(defaults.sequential().clone()),
        Some(constant.clone()),
    )
    .unwrap();
    assert_eq!(sequential.kind(), ContinuousScaleKind::Sequential);
    assert_eq!(sequential.domain(), Some(&constant));
    assert_eq!(
        sequential.sequential_gradient(),
        Some(defaults.sequential())
    );
    assert!(sequential.center().is_none());
    assert!(sequential.diverging_gradient().is_none());
    assert!(sequential.clone().with_center(0.0).is_err());
    assert!(sequential.clone().with_diverging_gradient(None).is_err());
    assert!(sequential
        .clone()
        .with_sequential_gradient(None)
        .unwrap()
        .sequential_gradient()
        .is_none());
    assert!(ContinuousColorScale::diverging(1.0, None, Some(constant)).is_err());
    assert!(ContinuousColorScale::diverging(f64::INFINITY, None, None).is_err());
}

#[test]
fn fixed_plot_palette_capacity_is_checked_before_fonts_even_with_hidden_legend() {
    let theme = crate::Theme::publication()
        .customize()
        .font_stack(
            crate::FontStack::new([crate::FontFamily::named(
                "Absent fixed palette preflight font",
            )
            .unwrap()])
            .unwrap(),
        )
        .unwrap()
        .build()
        .unwrap();
    let plot = crate::Plot::scatter(
        crate::NumericValues::new("x", [1.0, 2.0]),
        crate::NumericValues::new("y", [2.0, 1.0]),
    )
    .hue(crate::CategoryValues::new("raw", ["A  B", "A B"]).unwrap())
    .theme(theme)
    .legend(crate::LegendPolicy::Hidden)
    .categorical_color_scale(CategoricalColorScale::from_palette(
        QualitativePalette::fixed([OpaqueColor::rgb(13, 29, 47)]).unwrap(),
    ));
    let discoveries = crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get);
    let error = plot.preflight().unwrap_err();
    assert!(matches!(error, crate::Error::Data(_)), "{error}");
    assert!(
        error.to_string().contains("1 colors for 2 used identities"),
        "{error}"
    );
    assert_eq!(
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        discoveries,
    );
}

#[test]
fn hidden_shared_plots_validate_the_final_palette_domain_before_fonts() {
    let base = crate::Theme::publication();
    let scales = base
        .color_scales()
        .clone()
        .with_qualitative(QualitativePalette::fixed([OpaqueColor::rgb(13, 29, 47)]).unwrap())
        .unwrap();
    let theme = base
        .customize()
        .color_scales(scales)
        .font_stack(
            crate::FontStack::new([crate::FontFamily::named(
                "Absent shared palette preflight font",
            )
            .unwrap()])
            .unwrap(),
        )
        .unwrap()
        .build()
        .unwrap();
    let scatter = |key| {
        crate::Plot::scatter(
            crate::NumericValues::new("x", [1.0, 2.0]),
            crate::NumericValues::new("y", [2.0, 1.0]),
        )
        .hue(crate::CategoryValues::new("raw", [key, key]).unwrap())
        .legend(crate::LegendPolicy::Hidden)
    };
    let canvas = crate::Canvas::grid(1, 2)
        .unwrap()
        .theme(theme)
        .panel(0, 0, scatter("A  B"))
        .unwrap()
        .panel(0, 1, scatter("A B"))
        .unwrap()
        .legend_policy(crate::CanvasLegendPolicy::Shared);
    let discoveries = crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get);
    let error = canvas.preflight().unwrap_err();
    assert!(matches!(error, crate::Error::Data(_)), "{error}");
    assert!(
        error.to_string().contains("1 colors for 2 used identities"),
        "{error}"
    );
    assert_eq!(
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        discoveries,
    );
}

#[test]
fn resolved_palette_edits_do_not_change_accents_and_heatmap_reads_its_actual_owner() {
    use crate::render::backend::Backend;
    let resources = OperationContext::new();
    let mut theme = crate::Theme::publication().resolve().unwrap();
    let original = theme.clone();
    let palette =
        QualitativePalette::fixed([OpaqueColor::rgb(11, 23, 37), OpaqueColor::rgb(41, 59, 73)])
            .unwrap();
    let gradient = SequentialGradient::new([stop(0.0, 83), stop(0.4, 97), stop(1.0, 109)]).unwrap();
    theme.color_scales = theme
        .color_scales
        .clone()
        .with_qualitative(palette.clone())
        .unwrap()
        .with_sequential(gradient.clone())
        .unwrap();
    assert_eq!(theme.marks.accents(), original.marks.accents());
    let keys = ["one".into(), "two".into()];
    let mapped = super::super::CategoryColors::resolve(
        &keys,
        theme.color_scales.qualitative(),
        None,
        &resources,
    )
    .unwrap();
    assert_eq!(mapped.colors(), palette.colors());
    assert!(super::super::CategoryColors::resolve(
        &["one".into(), "two".into(), "three".into()],
        theme.color_scales.qualitative(),
        None,
        &resources
    )
    .is_err());
    let source = crate::Plot::heatmap(
        crate::CategoryValues::new("column", ["left", "right"]).unwrap(),
        crate::CategoryValues::new("row", ["one", "one"]).unwrap(),
        crate::NumericValues::new("value", [0.0, 1.0]),
    )
    .erase();
    crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
    let inputs = crate::plots::shared_x::prepare_plot(&source, &resources, Some(&theme)).unwrap();
    let prepared = &inputs.heatmap.as_ref().unwrap().panels[0].1;
    let semantics = prepared.guide_semantics(&theme).unwrap();
    assert_eq!(semantics.selection.sequential_gradient(), Some(&gradient));
    assert_eq!(
        semantics.selection.stops().unwrap().as_ptr(),
        gradient.stops().as_ptr()
    );
    assert_eq!(
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        0
    );
    assert!(crate::plots::shared_x::prepare_plot(&source, &resources, None).is_err());
    let chart = inputs.attach_with_guides_for_test(source, &theme).unwrap();
    let selected = chart
        .measure_selected_interior(crate::spec::PlotRenderContext::new(
            chart.data(),
            crate::layout::PanelBox::new(20.0, 640.0, 20.0, 480.0),
            &theme,
        ))
        .unwrap();
    let crate::layout::interior::MarkPayload::Heatmap(payload) =
        &selected.marks.as_ref().unwrap().payload
    else {
        panic!("selected owner Heatmap")
    };
    let mut backend = crate::render::svg_backend::SvgBackend::in_memory_with_fonts(
        700,
        500,
        theme.prepared_font_environment().unwrap().clone(),
    );
    backend.begin(700, 500).unwrap();
    payload.draw(&mut backend).unwrap();
    backend.end().unwrap();
    let svg = String::from_utf8(backend.into_bytes().unwrap()).unwrap();
    assert!(svg.contains("fill=\"#53565A\""));
    assert!(svg.contains("fill=\"#6D7074\""));
    assert_eq!(
        original.color_scales.qualitative().capacity(),
        PaletteCapacity::Extendable
    );
}
