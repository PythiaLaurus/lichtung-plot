use super::*;
use crate::font::canonical::{
    FontFamily, FontStack, GenericFontFamily, OperationGenerationResources,
};
use crate::{CategoryValues, NumericValues};

fn source() -> Plot {
    Plot::heatmap(
        CategoryValues::new("column", ["left", "right"]).unwrap(),
        CategoryValues::new("row", ["one", "one"]).unwrap(),
        NumericValues::new("value", [0.0, 1.0]),
    )
    .erase()
}

fn theme() -> ResolvedTheme {
    let mut theme = crate::Theme::Publication
        .resolve(&OperationGenerationResources::new())
        .unwrap();
    let stack = FontStack::new([FontFamily::generic(GenericFontFamily::SansSerif)]).unwrap();
    for role in [
        &mut theme.typography.contrast_light,
        &mut theme.typography.contrast_dark,
        &mut theme.typography.tick_label,
        &mut theme.typography.axis_title,
    ] {
        role.stack = stack.clone();
    }
    theme
}

#[test]
fn intrinsic_cache_copy_prices_one_gradient_handle_and_no_stop_records_or_bytes() {
    let source = source();
    let theme = theme();
    let preparation = OperationGenerationResources::new();
    let mut prepared = prepare(
        &source,
        source.data().with_axis_resources(&preparation),
        &theme.color_scales,
    )
    .unwrap();
    // The cache object and four existing one-entry generic font copies are the
    // entire storage oracle, independent of the number of retained color stops.
    let bytes = (size_of::<HeatmapIntrinsicCache>()
        + 4 * (size_of::<Vec<FontFamily>>() + 2 * size_of::<usize>() + size_of::<FontFamily>()))
        as u64;
    for stops in [
        crate::theme::HEATMAP_SEQUENTIAL_STOPS.to_vec(),
        vec![
            ColorStop::new(0.0, OpaqueColor::rgb(11, 23, 37)).unwrap(),
            ColorStop::new(0.25, OpaqueColor::rgb(41, 59, 73)).unwrap(),
            ColorStop::new(0.5, OpaqueColor::rgb(83, 97, 109)).unwrap(),
            ColorStop::new(0.75, OpaqueColor::rgb(127, 149, 163)).unwrap(),
            ColorStop::new(1.0, OpaqueColor::rgb(179, 193, 211)).unwrap(),
        ],
    ] {
        prepared.color.scale =
            crate::theme::ContinuousColors::resolve((0.0, 1.0), &stops, &preparation).unwrap();
        for (work, byte_limit, records) in [
            (0, bytes, 4),
            (4, bytes, 4),
            (5, bytes - 1, 4),
            (5, bytes, 3),
            (5, bytes, 4),
        ] {
            let resources = OperationGenerationResources::with_test_limits(work, byte_limit);
            resources.claim_slots(1_000_000 - records).unwrap();
            let result = HeatmapIntrinsicCache::new(
                (13.0, 17.0),
                (11.0, 19.0),
                &prepared,
                &theme,
                source.data().with_axis_resources(&resources),
            );
            assert_eq!(
                result.is_ok(),
                work == 5 && byte_limit == bytes && records == 4
            );
            if let Ok(cache) = result {
                assert_eq!(resources.usage().unwrap(), (1_000_000, 5));
                assert_eq!(resources.byte_usage().unwrap(), bytes);
                assert_eq!(
                    cache.color_scale.stops().as_ptr(),
                    prepared.color.scale.stops().as_ptr()
                );
                assert_eq!(cache.minimum.0.to_bits(), 13.0_f64.to_bits());
                assert_eq!(cache.minimum.1.to_bits(), 17.0_f64.to_bits());
                let matching = OperationGenerationResources::with_test_limits(10, 0);
                assert!(cache.matches(&prepared, &theme, &matching).unwrap());
                assert_eq!(matching.usage().unwrap(), (0, 10));
                assert_eq!(matching.byte_usage().unwrap(), 0);
                assert_eq!(
                    cache.color_scale.stops().as_ptr(),
                    prepared.color.scale.stops().as_ptr()
                );
            }
        }
    }
}

#[test]
fn intrinsic_cache_matches_admit_independent_full_stop_content_and_fatal_refusal() {
    let source = source();
    let theme = theme();
    let preparation = OperationGenerationResources::new();
    let mut prepared = prepare(
        &source,
        source.data().with_axis_resources(&preparation),
        &theme.color_scales,
    )
    .unwrap();
    let cache = HeatmapIntrinsicCache::new(
        (13.0, 17.0),
        (11.0, 19.0),
        &prepared,
        &theme,
        source.data().with_axis_resources(&preparation),
    )
    .unwrap();
    let stops = prepared.color.scale.stops().to_vec();
    let original = cache.color_scale.stops().as_ptr();
    prepared.color.scale =
        crate::theme::ContinuousColors::resolve((0.0, 1.0), &stops, &preparation).unwrap();
    assert_ne!(original, prepared.color.scale.stops().as_ptr());
    let complete = 10 + stops.len() as u128;
    for work in [complete - 1, complete] {
        let resources = OperationGenerationResources::with_test_limits(work, 0);
        let result = cache.matches(&prepared, &theme, &resources);
        if work == complete {
            assert!(result.unwrap());
        } else {
            assert!(result.is_err());
        }
        assert_eq!(resources.usage().unwrap().1, work);
        assert_eq!(resources.byte_usage().unwrap(), 0);
    }
    let mut changed = stops.clone();
    *changed.last_mut().unwrap() = ColorStop::new(1.0, OpaqueColor::rgb(71, 83, 97)).unwrap();
    prepared.color.scale =
        crate::theme::ContinuousColors::resolve((0.0, 1.0), &changed, &preparation).unwrap();
    let stop_comparison = 2 + stops.len() as u128;
    assert!(cache
        .matches(
            &prepared,
            &theme,
            &OperationGenerationResources::with_test_limits(stop_comparison - 1, 0)
        )
        .is_err());
    let resources = OperationGenerationResources::with_test_limits(stop_comparison, 0);
    assert!(!cache.matches(&prepared, &theme, &resources).unwrap());
    assert_eq!(resources.usage().unwrap().1, stop_comparison);
    assert_eq!(resources.byte_usage().unwrap(), 0);
}

#[test]
fn intrinsic_cache_hit_keeps_storage_and_refuses_matching_work_without_reanalysis() {
    let source = source();
    let theme = theme();
    let preparation = OperationGenerationResources::new();
    let inputs =
        crate::plots::shared_x::prepare_plot(&source, &preparation, Some(&theme.color_scales))
            .unwrap();
    let axes = &inputs.axes[0].1;
    let prepared = &inputs.heatmap.as_ref().unwrap().panels[0].1;
    let cache = HeatmapIntrinsicCache::new(
        (13.0, 17.0),
        (11.0, 19.0),
        prepared,
        &theme,
        source.data().with_axis_resources(&preparation),
    )
    .unwrap();
    let stops = cache.color_scale.stops().as_ptr();
    axes.heatmap_intrinsics.set(cache).unwrap();
    theme.prepare_font_environment().unwrap();
    let resources = theme
        .prepared_font_environment()
        .unwrap()
        .generation_resources();
    for matching_work in [9, 10] {
        let before = resources.usage().unwrap();
        let bytes = resources.byte_usage().unwrap();
        let calls = resources.call_cache_usage();
        resources.set_test_limits(before.1 + matching_work, bytes);
        let result = prepare_intrinsics(prepared, axes, source.data(), &theme, None);
        if matching_work == 10 {
            let minimum = result.unwrap();
            assert_eq!(minimum.0.to_bits(), 13.0_f64.to_bits());
            assert_eq!(minimum.1.to_bits(), 17.0_f64.to_bits());
        } else {
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("root processing visits"));
        }
        assert_eq!(
            resources.usage().unwrap(),
            (before.0, before.1 + matching_work)
        );
        assert_eq!(resources.byte_usage().unwrap(), bytes);
        assert_eq!(resources.call_cache_usage(), (calls.0 + 1, calls.1));
        assert_eq!(
            axes.heatmap_intrinsics
                .get()
                .unwrap()
                .color_scale
                .stops()
                .as_ptr(),
            stops
        );
    }
}
