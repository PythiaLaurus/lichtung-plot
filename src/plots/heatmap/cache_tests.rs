use super::*;
use crate::font::canonical::{FontFamily, FontStack, GenericFontFamily, OperationContext};
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
    let mut theme = crate::Theme::Publication.resolve().unwrap();
    let stack = FontStack::new([FontFamily::generic(GenericFontFamily::SansSerif)]).unwrap();
    for role in [
        &mut theme.typography.annotations.contrast.light,
        &mut theme.typography.annotations.contrast.dark,
        &mut theme.typography.axes.tick_label,
        &mut theme.typography.axes.axis_title,
    ] {
        role.stack = stack.clone();
    }
    theme
}

#[test]
fn intrinsic_cache_copy_shares_the_complete_gradient() {
    let source = source();
    let theme = theme();
    let preparation = OperationContext::new();
    let mut prepared = prepare(
        &source,
        source.data().with_operation(&preparation),
        &theme.color_scales,
    )
    .unwrap();
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
        prepared.color.scale = crate::theme::ContinuousColors::resolve((0.0, 1.0), &stops).unwrap();
        {
            let result = HeatmapIntrinsicCache::new((13.0, 17.0), (11.0, 19.0), &prepared, &theme);
            {
                let cache = result.unwrap();
                assert_eq!(
                    cache.color_scale.stops().as_ptr(),
                    prepared.color.scale.stops().as_ptr()
                );
                assert_eq!(cache.minimum.0.to_bits(), 13.0_f64.to_bits());
                assert_eq!(cache.minimum.1.to_bits(), 17.0_f64.to_bits());

                assert!(cache.matches_inputs(&prepared, &theme));

                assert_eq!(
                    cache.color_scale.stops().as_ptr(),
                    prepared.color.scale.stops().as_ptr()
                );
            }
        }
    }
}

#[test]
fn intrinsic_cache_matches_independent_full_stop_content() {
    let source = source();
    let theme = theme();
    let preparation = OperationContext::new();
    let mut prepared = prepare(
        &source,
        source.data().with_operation(&preparation),
        &theme.color_scales,
    )
    .unwrap();
    let cache = HeatmapIntrinsicCache::new((13.0, 17.0), (11.0, 19.0), &prepared, &theme).unwrap();
    let stops = prepared.color.scale.stops().to_vec();
    let original = cache.color_scale.stops().as_ptr();
    prepared.color.scale = crate::theme::ContinuousColors::resolve((0.0, 1.0), &stops).unwrap();
    assert_ne!(original, prepared.color.scale.stops().as_ptr());
    assert!(cache.matches_inputs(&prepared, &theme));
    let mut changed = stops.clone();
    *changed.last_mut().unwrap() = ColorStop::new(1.0, OpaqueColor::rgb(71, 83, 97)).unwrap();
    prepared.color.scale = crate::theme::ContinuousColors::resolve((0.0, 1.0), &changed).unwrap();

    assert!(!cache.matches_inputs(&prepared, &theme));
}

#[test]
fn intrinsic_cache_hits_keep_storage_without_reanalysis() {
    let source = source();
    let theme = theme();
    let preparation = OperationContext::new();
    let inputs =
        crate::plots::shared_x::prepare_plot(&source, &preparation, Some(&theme.color_scales))
            .unwrap();
    let axes = &inputs.axes[0].1;
    let prepared = &inputs.heatmap.as_ref().unwrap().panels[0].1;
    let cache = HeatmapIntrinsicCache::new((13.0, 17.0), (11.0, 19.0), prepared, &theme).unwrap();
    let stops = cache.color_scale.stops().as_ptr();
    axes.heatmap_intrinsics.set(cache).unwrap();
    theme.prepare_font_environment().unwrap();

    for _ in 0..2 {
        let result = prepare_intrinsics(prepared, axes, &theme, None);
        {
            let minimum = result.unwrap();
            assert_eq!(minimum.0.to_bits(), 13.0_f64.to_bits());
            assert_eq!(minimum.1.to_bits(), 17.0_f64.to_bits());
        }

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
