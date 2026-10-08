use super::*;
use crate::font::canonical::OperationContext;

#[test]
fn used_map_lookup_is_exact_and_clone_shares_keys() {
    let map =
        CategoricalColors::from_opaque([("αx".into(), OpaqueColor::rgb(11, 23, 37))]).unwrap();
    assert_eq!(map.color_in("αx"), Some(OpaqueColor::rgb(11, 23, 37)));
    assert_eq!(map.color_in("αy"), None);
    let copy = map.clone();
    assert_eq!(copy, map);
    assert_eq!(
        copy.entries().next().unwrap().0.as_ptr(),
        map.entries().next().unwrap().0.as_ptr()
    );
}

fn stop(offset: f64, red: u8) -> ColorStop {
    ColorStop::new(
        offset,
        OpaqueColor::rgb(red, red.wrapping_add(3), red.wrapping_add(7)),
    )
    .unwrap()
}

#[test]
fn palette_capacity_order_repeated_colors_and_immutable_edits_are_independent() {
    let colors = [
        OpaqueColor::rgb(13, 29, 47),
        OpaqueColor::rgb(71, 83, 97),
        OpaqueColor::rgb(13, 29, 47),
    ];
    let fixed = QualitativePalette::fixed(colors).unwrap();
    let extendable = QualitativePalette::extendable(colors).unwrap();
    assert_eq!(fixed.colors(), colors);
    assert_eq!(fixed.capacity(), PaletteCapacity::Fixed);
    assert_eq!(extendable.colors(), colors);
    assert_eq!(extendable.capacity(), PaletteCapacity::Extendable);
    assert_ne!(fixed, extendable);
    let accents = AccentPalette::new(colors).unwrap();
    assert_eq!(accents.seeds(), colors);
    assert!(QualitativePalette::fixed([]).is_err());
    assert!(QualitativePalette::extendable([]).is_err());
    assert!(AccentPalette::new([]).is_err());
    let selected = CategoricalColorScale::from_palette(fixed.clone());
    assert_eq!(selected.palette(), Some(&fixed));
    assert!(selected.map().is_none());
    let map = CategoricalColors::from_opaque([("raw".into(), colors[1])]).unwrap();
    let mapped = CategoricalColorScale::from_map(map.clone());
    assert_eq!(mapped.map(), Some(&map));
    assert!(mapped.palette().is_none());
}

#[test]
fn map_retains_exact_raw_keys_order_and_opaque_colors_without_normalization() {
    let rgb = OpaqueColor::rgb(13, 29, 47);
    let map = CategoricalColors::from_opaque([
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
    assert!(CategoricalColors::from_opaque([]).is_err());
    for key in ["", " leading", "trailing ", "control\n", "\u{0000}"] {
        assert!(CategoricalColors::from_opaque([(key.into(), rgb)]).is_err());
    }
    assert!(
        CategoricalColors::from_opaque([("duplicate".into(), rgb), ("duplicate".into(), rgb)])
            .is_err()
    );
    let copied = map.clone();
    assert_eq!(
        copied.entries().next().unwrap().0.as_ptr(),
        map.entries().next().unwrap().0.as_ptr()
    );
}

#[test]
fn ordered_gradients_normalize_zero_and_require_exact_endpoints_and_neutral() {
    assert_eq!(
        ColorStop::new(-0.0, OpaqueColor::rgb(1, 2, 3))
            .unwrap()
            .offset()
            .to_bits(),
        0
    );
    for offset in [-1.0, 1.000_001, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(ColorStop::new(offset, OpaqueColor::rgb(1, 2, 3)).is_err());
    }
    let values = [
        stop(0.0, 11),
        stop(0.125, 23),
        stop(0.5, 37),
        stop(0.875, 53),
        stop(1.0, 71),
    ];
    let sequential = SequentialGradient::new(values).unwrap();
    let diverging = DivergingGradient::new(values).unwrap();
    assert_eq!(sequential.stops(), values);
    assert_eq!(diverging.stops(), values);
    assert_eq!(diverging.neutral(), OpaqueColor::rgb(37, 40, 44));
    assert!(SequentialGradient::new([]).is_err());
    assert!(DivergingGradient::new([]).is_err());
    for invalid in [
        vec![values[0]],
        vec![values[4], values[0]],
        vec![values[0], values[0], values[4]],
        vec![values[1], values[4]],
        vec![values[0], values[3]],
    ] {
        assert!(SequentialGradient::new(invalid.clone()).is_err());
        assert!(DivergingGradient::new(invalid).is_err());
    }
    assert!(DivergingGradient::new([values[0], values[1], values[4]]).is_err());
    assert!(DivergingGradient::new([values[0], stop(0.5, 31), stop(0.5, 41), values[4]]).is_err());
    assert!(DivergingGradient::new([
        values[0],
        stop(f64::from_bits(0.5_f64.to_bits() + 1), 31),
        values[4]
    ])
    .is_err());
}

#[test]
fn complete_scale_component_replacements_leave_source_and_siblings_intact() {
    let original = test_defaults();
    let palette = QualitativePalette::fixed([OpaqueColor::rgb(13, 29, 47)]).unwrap();
    let seq = SequentialGradient::new([stop(0.0, 11), stop(1.0, 23)]).unwrap();
    let div = DivergingGradient::new([stop(0.0, 37), stop(0.5, 53), stop(1.0, 71)]).unwrap();
    let recolored = original.clone().with_qualitative(palette.clone()).unwrap();
    assert_eq!(recolored.qualitative(), &palette);
    assert_eq!(recolored.sequential(), original.sequential());
    assert_eq!(recolored.diverging(), original.diverging());
    let resequenced = original.clone().with_sequential(seq.clone()).unwrap();
    assert_eq!(resequenced.sequential(), &seq);
    assert_eq!(resequenced.qualitative(), original.qualitative());
    assert_eq!(resequenced.diverging(), original.diverging());
    let recentered = original.clone().with_diverging(div.clone()).unwrap();
    assert_eq!(recentered.diverging(), &div);
    assert_eq!(recentered.qualitative(), original.qualitative());
    assert_eq!(recentered.sequential(), original.sequential());
    assert_eq!(
        original.qualitative().capacity(),
        PaletteCapacity::Extendable
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
fn sequence_retention_and_immutable_sharing_preserve_values() {
    let values = [OpaqueColor::rgb(13, 29, 47), OpaqueColor::rgb(71, 83, 97)];
    assert_eq!(AccentPalette::new(values).unwrap().seeds(), values);
    let source = ContinuousColorScale::sequential(
        Some(SequentialGradient::new(HEATMAP_SEQUENTIAL_STOPS).unwrap()),
        None,
    )
    .unwrap();

    let copy = source.clone();
    assert_eq!(copy, source);
    assert_eq!(
        copy.stops().unwrap().as_ptr(),
        source.stops().unwrap().as_ptr()
    );
}

#[test]
fn unknown_length_sequence_growth_preserves_values() {
    let colors = [OpaqueColor::rgb(11, 23, 37), OpaqueColor::rgb(41, 59, 73)];
    let palette = AccentPalette::new(colors.into_iter().filter(|_| true)).unwrap();
    assert_eq!(palette.seeds(), colors);
}

#[test]
fn resolved_palette_edits_do_not_change_accents_and_heatmap_reads_its_actual_owner() {
    use crate::render::backend::Backend;
    let resources = OperationContext::new();
    let mut theme = crate::Theme::Publication.resolve().unwrap();
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
    assert_eq!(theme.marks.accents, original.marks.accents);
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
    let inputs =
        crate::plots::shared_x::prepare_plot(&source, &resources, Some(&theme.color_scales))
            .unwrap();
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
    let chart = inputs.attach(source).unwrap();
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
