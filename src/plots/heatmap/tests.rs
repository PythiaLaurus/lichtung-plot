use super::*;
use crate::layout::deterministic_text_width;
use crate::test_support::{into_column, Fixture, FixtureColumn};
use crate::{CategoryOrder, ValueSort};

fn data(x: FixtureColumn, y: FixtureColumn, fill: FixtureColumn) -> Fixture {
    Fixture::from_columns(vec![("x", x), ("y", y), ("fill", fill)]).unwrap()
}

fn valid_data() -> Fixture {
    data(
        into_column([" b ", "a", "b"]),
        into_column(["first", "first", " second  row "]),
        into_column([1.0_f64, 2.0, 3.0]),
    )
}

fn chart(data: &Fixture) -> Plot<crate::kind::Heatmap> {
    Plot::heatmap(
        crate::test_support::category(data, "x"),
        crate::test_support::category(data, "y"),
        crate::test_support::numeric(data, "fill"),
    )
}

fn prepare_chart(chart: &Plot<crate::kind::Heatmap>) -> Result<HeatmapData, Error> {
    let chart = chart.clone().erase();
    prepare(&chart, &chart)
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
fn prepare_applies_independent_explicit_orders_to_shuffled_cells() {
    let data = data(
        into_column(["b", "a", "b", "a"]),
        into_column(["south", "north", "north", "south"]),
        into_column([4.0_f64, 1.0, 2.0, 3.0]),
    );
    let chart = chart(&data)
        .x_category_order(CategoryOrder::explicit([" a ", "b"]))
        .y_category_order(CategoryOrder::explicit([" north ", "south"]));

    let prepared = prepare_chart(&chart).unwrap();

    assert_eq!(prepared.x_levels, ["a", "b"]);
    assert_eq!(prepared.y_levels, ["north", "south"]);
    assert_eq!(prepared.cells, [Some(1.0), Some(2.0), Some(3.0), Some(4.0)]);
}

#[test]
fn prepare_rejects_inexact_explicit_axis_orders() {
    let data = valid_data();
    let orders = [vec!["b", "b"], vec!["b"], vec!["b", "a", "unknown"]];

    let errors = orders
        .into_iter()
        .map(|order| {
            prepare_chart(&chart(&data).x_category_order(CategoryOrder::explicit(order)))
                .unwrap_err()
                .to_string()
        })
        .collect::<Vec<_>>();

    assert!(errors[0].contains("duplicate level"));
    assert!(errors[1].contains("omits observed level"));
    assert!(errors[2].contains("unknown level"));
}

#[test]
fn prepare_reorients_triangular_pairs_under_explicit_family_order() {
    let data = data(
        into_column(["a", "a", "b"]),
        into_column(["b", "c", "c"]),
        into_column([1.0_f64, 2.0, 3.0]),
    );
    let chart = chart(&data)
        .layout(HeatmapLayout::LowerTriangle { diagonal: false })
        .x_category_order(CategoryOrder::explicit(["c", "b", "a"]));

    let prepared = prepare_chart(&chart).unwrap();

    assert_eq!(prepared.x_levels, ["c", "b", "a"]);
    assert_eq!(prepared.y_levels, ["c", "b", "a"]);
    assert_eq!(prepared.cells[3], Some(3.0));
    assert_eq!(prepared.cells[6], Some(2.0));
    assert_eq!(prepared.cells[7], Some(1.0));
}

#[test]
fn prepare_builds_cartesian_grid_with_absent_and_null_cells() {
    let data = data(
        into_column(["a", "b", "a"]),
        into_column(["one", "one", "two"]),
        into_column([Some(1.0_f64), None, Some(3.0)]),
    );

    let prepared = prepare_chart(&chart(&data)).unwrap();

    assert_eq!(prepared.cells, [Some(1.0), None, Some(3.0), None]);
    assert!(prepared.has_missing);
}

#[test]
fn prepare_rejects_duplicate_normalized_pairs() {
    let data = data(
        into_column(["a b", " a  b "]),
        into_column(["one", "one"]),
        into_column([1.0_f64, 2.0]),
    );

    let error = prepare_chart(&chart(&data)).unwrap_err().to_string();

    assert!(error.contains("duplicate normalized"));
}

#[test]
fn prepare_rejects_null_and_empty_category_labels() {
    for x in [
        into_column([Some("a"), None]),
        into_column([Some("a"), Some(" \t ")]),
    ] {
        let data = data(x, into_column(["one", "two"]), into_column([1.0_f64, 2.0]));
        assert!(prepare_chart(&chart(&data)).is_err());
    }
}

#[test]
fn prepare_rejects_non_finite_and_all_null_fill() {
    for fill in [
        into_column([Some(1.0_f64), Some(f64::NAN)]),
        into_column([Some(1.0_f64), Some(f64::INFINITY)]),
        into_column([None::<f64>, None]),
    ] {
        let data = data(into_column(["a", "b"]), into_column(["one", "one"]), fill);
        assert!(prepare_chart(&chart(&data)).is_err());
    }
}

#[test]
fn prepare_rejects_cartesian_grid_above_safety_limit() {
    let levels = (0..=1_000)
        .map(|index| format!("v{index}"))
        .collect::<Vec<_>>();
    let data = data(
        into_column(levels.clone()),
        into_column(levels),
        into_column(vec![1.0_f64; 1_001]),
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
        chart(&data).x_category_order(ValueSort::Ascending).erase(),
        chart(&data)
            .scale(HeatmapScale::Diverging { center: f64::NAN })
            .erase(),
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
