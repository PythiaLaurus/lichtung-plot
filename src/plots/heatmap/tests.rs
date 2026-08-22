use polars::prelude::{DataFrame, NamedFrom, Series};

use super::*;
use crate::layout::deterministic_text_width;

fn data(x: Series, y: Series, fill: Series) -> DataFrame {
    DataFrame::new(x.len(), vec![x.into(), y.into(), fill.into()]).unwrap()
}

fn valid_data() -> DataFrame {
    data(
        Series::new("x".into(), [" b ", "a", "b"]),
        Series::new("y".into(), ["first", "first", " second  row "]),
        Series::new("fill".into(), [1.0_f64, 2.0, 3.0]),
    )
}

fn chart(data: &DataFrame) -> Plot {
    Plot::heatmap(
        crate::test_support::category(data, "x"),
        crate::test_support::category(data, "y"),
        crate::test_support::numeric(data, "fill"),
    )
}

fn prepare_chart(chart: &Plot) -> Result<HeatmapData, Error> {
    prepare(chart, chart)
}

#[test]
fn prepare_normalizes_categories_and_preserves_first_observed_axes() {
    let data = valid_data();

    let prepared = prepare_chart(&chart(&data)).unwrap();

    assert_eq!(
        (prepared.x_levels, prepared.y_levels),
        (
            vec!["b".to_string(), "a".to_string()],
            vec!["first".to_string(), "second row".to_string()]
        )
    );
}

#[test]
fn prepare_builds_cartesian_grid_with_absent_and_null_cells() {
    let data = data(
        Series::new("x".into(), ["a", "b", "a"]),
        Series::new("y".into(), ["one", "one", "two"]),
        Series::new("fill".into(), [Some(1.0_f64), None, Some(3.0)]),
    );

    let prepared = prepare_chart(&chart(&data)).unwrap();

    assert_eq!(prepared.cells, [Some(1.0), None, Some(3.0), None]);
    assert!(prepared.has_missing);
}

#[test]
fn prepare_rejects_duplicate_normalized_pairs() {
    let data = data(
        Series::new("x".into(), ["a b", " a  b "]),
        Series::new("y".into(), ["one", "one"]),
        Series::new("fill".into(), [1.0_f64, 2.0]),
    );

    let error = prepare_chart(&chart(&data)).unwrap_err().to_string();

    assert!(error.contains("duplicate normalized"));
}

#[test]
fn prepare_rejects_null_and_empty_category_labels() {
    for x in [
        Series::new("x".into(), [Some("a"), None]),
        Series::new("x".into(), ["a", " \t "]),
    ] {
        let data = data(
            x,
            Series::new("y".into(), ["one", "two"]),
            Series::new("fill".into(), [1.0_f64, 2.0]),
        );
        assert!(prepare_chart(&chart(&data)).is_err());
    }
}

#[test]
fn prepare_rejects_non_finite_and_all_null_fill() {
    for fill in [
        Series::new("fill".into(), [Some(1.0_f64), Some(f64::NAN)]),
        Series::new("fill".into(), [Some(1.0_f64), Some(f64::INFINITY)]),
        Series::new("fill".into(), [None::<f64>, None]),
    ] {
        let data = data(
            Series::new("x".into(), ["a", "b"]),
            Series::new("y".into(), ["one", "one"]),
            fill,
        );
        assert!(prepare_chart(&chart(&data)).is_err());
    }
}

#[test]
fn prepare_rejects_cartesian_grid_above_safety_limit() {
    let levels = (0..=1_000)
        .map(|index| format!("v{index}"))
        .collect::<Vec<_>>();
    let data = data(
        Series::new("x".into(), levels.clone()),
        Series::new("y".into(), levels),
        Series::new("fill".into(), vec![1.0_f64; 1_001]),
    );

    let error = prepare_chart(&chart(&data)).unwrap_err().to_string();

    assert!(error.contains("safety limit"));
}

#[test]
fn color_domain_uses_exact_explicit_limits_and_validates_diverging_center() {
    let exact = color_domain(&[1.0], HeatmapScale::Sequential, Some((-2.0, 4.0))).unwrap();
    let error = color_domain(
        &[1.0],
        HeatmapScale::Diverging { center: 0.0 },
        Some((1.0, 2.0)),
    )
    .unwrap_err();

    assert_eq!(
        exact,
        ColorDomain {
            minimum: -2.0,
            maximum: 4.0,
            center: None
        }
    );
    assert!(error.to_string().contains("must contain center"));
}

#[test]
fn diverging_auto_domain_is_symmetric_and_handles_constant_signed_zero_and_subnormal() {
    let symmetric =
        color_domain(&[-2.0, 3.0], HeatmapScale::Diverging { center: 1.0 }, None).unwrap();
    let zero = color_domain(&[-0.0, 0.0], HeatmapScale::Diverging { center: 0.0 }, None).unwrap();
    let subnormal = f64::from_bits(1);
    let tiny = color_domain(
        &[0.0, subnormal],
        HeatmapScale::Diverging { center: 0.0 },
        None,
    )
    .unwrap();

    assert_eq!(
        symmetric,
        ColorDomain {
            minimum: -2.0,
            maximum: 4.0,
            center: Some(1.0)
        }
    );
    assert_eq!((zero.minimum, zero.maximum), (0.0, 0.0));
    assert_eq!((tiny.minimum, tiny.maximum), (-subnormal, subnormal));
}

#[test]
fn diverging_auto_domain_accepts_opposite_extremes_around_zero() {
    let domain = color_domain(
        &[-f64::MAX, f64::MAX],
        HeatmapScale::Diverging { center: 0.0 },
        None,
    )
    .unwrap();

    assert_eq!((domain.minimum, domain.maximum), (-f64::MAX, f64::MAX));
    assert!(normalized_fraction(0.0, domain.minimum, domain.maximum).is_finite());
}

#[test]
fn diverging_auto_domain_reports_unrepresentable_symmetry() {
    let error = color_domain(
        &[-f64::MAX],
        HeatmapScale::Diverging { center: f64::MAX },
        None,
    )
    .unwrap_err();

    assert!(error.to_string().contains("finite symmetric"));
}

#[test]
fn constant_domains_map_to_midpoint_colors_and_explicit_domains_saturate() {
    let sequential = ColorDomain {
        minimum: 2.0,
        maximum: 2.0,
        center: None,
    };
    let diverging = ColorDomain {
        minimum: -1.0,
        maximum: 1.0,
        center: Some(0.0),
    };

    assert_eq!(
        heatmap_color(2.0, sequential),
        interpolate((247, 251, 255), (8, 48, 107), 0.5)
    );
    assert_eq!(heatmap_color(-2.0, diverging), (49, 130, 189));
    assert_eq!(heatmap_color(2.0, diverging), (222, 45, 38));
}

#[test]
fn colorbar_gradient_stops_preserve_sequential_diverging_and_constant_scales() {
    let sequential = colorbar_gradient_stops(ColorDomain {
        minimum: 0.0,
        maximum: 1.0,
        center: None,
    });
    let diverging = colorbar_gradient_stops(ColorDomain {
        minimum: -1.0,
        maximum: 3.0,
        center: Some(0.0),
    });
    let constant = colorbar_gradient_stops(ColorDomain {
        minimum: 2.0,
        maximum: 2.0,
        center: None,
    });

    assert_eq!(
        sequential,
        [
            GradientStop::new(0.0, (247, 251, 255)),
            GradientStop::new(1.0, (8, 48, 107)),
        ]
    );
    assert_eq!(
        diverging,
        [
            GradientStop::new(0.0, (49, 130, 189)),
            GradientStop::new(0.25, (247, 247, 247)),
            GradientStop::new(1.0, (222, 45, 38)),
        ]
    );
    assert_eq!(constant[0].color, constant[1].color);
}

#[test]
fn validate_options_rejects_heatmap_incompatible_configuration() {
    let data = valid_data();
    let charts = [
        chart(&data).x_limits(0.0, 1.0),
        chart(&data).y_tick_count(2),
        chart(&data).value_sort(ValueSort::Ascending),
        chart(&data).hue(crate::test_support::category(&data, "x")),
        chart(&data).heatmap_scale(HeatmapScale::Diverging { center: f64::NAN }),
    ];

    assert!(charts.iter().all(|chart| validate_options(chart).is_err()));
}

#[test]
fn fitted_label_elides_only_at_extended_grapheme_boundaries() {
    let expected = "e\u{301}...";
    let maximum_width = deterministic_text_width(expected, 12.0);

    let fitted = fitted_label("e\u{301}e\u{301}e\u{301}e\u{301}", maximum_width, 12);

    assert_eq!(fitted, expected);
    assert!(deterministic_text_width(&fitted, 12.0) <= maximum_width);
}

#[test]
fn fitted_unique_labels_rejects_space_too_narrow_for_an_ellipsis() {
    let error = fitted_unique_labels(&["category".into()], 1.0, 12, "x").unwrap_err();

    assert!(error.to_string().contains("no room for an ellipsis"));
}
