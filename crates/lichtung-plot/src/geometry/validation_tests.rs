use super::*;

fn histogram() -> HistogramGeometry {
    HistogramGeometry {
        value_dimension: "x".into(),
        density_dimension: "y".into(),
        bin_lower: NumericValues::new("lower", [-2.0]),
        bin_upper: NumericValues::new("upper", [2.0]),
        bin_density: NumericValues::new("density", [1.0]),
        kde_x: NumericValues::new("x", [-2.0, 2.0]),
        kde_y: NumericValues::new("density", [0.5, 0.5]),
        rug_x: NumericValues::new("x", [0.0]),
        rug_baseline: NumericValues::new("baseline", [0.0]),
        rug_top: NumericValues::new("top", [0.1]),
    }
}

fn raincloud() -> RaincloudGeometry {
    RaincloudGeometry {
        category_dimension: "x".into(),
        value_dimension: "y".into(),
        hue_dimension: Some("cohort".into()),
        category: CategoryValues::new("x", ["a", "b"]).unwrap(),
        hue: Some(CategoryValues::new("cohort", ["same"; 2]).unwrap()),
        q1: NumericValues::new("q1", [-1.0; 2]),
        median: NumericValues::new("median", [0.0; 2]),
        q3: NumericValues::new("q3", [1.0; 2]),
        density_category: CategoryValues::new("density category", ["b", "a", "b", "a"]).unwrap(),
        density_hue: Some(CategoryValues::new("cohort", ["same"; 4]).unwrap()),
        density_y: NumericValues::new("y", [-2.0, -2.0, 2.0, 2.0]),
        normalized_width: NumericValues::new("width", [0.5; 4]),
        point_category: CategoryValues::new("point category", ["b", "a"]).unwrap(),
        point_hue: Some(CategoryValues::new("cohort", ["same"; 2]).unwrap()),
        point_y: NumericValues::new("y", [0.0; 2]),
        normalized_strip: NumericValues::new("strip", [0.5; 2]),
    }
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one supplied-input witness verifies independently keyed summary/observation paints and shared guide membership"
)]
fn supplied_geometry_preserves_support_and_rejects_nonfinite_values() {
    let histogram = histogram();
    let raincloud = raincloud();
    assert_eq!(histogram.validate().unwrap(), (-2.0, 2.0));
    assert_eq!(raincloud.membership().unwrap().range, (-2.0, 2.0));
    let supplied = crate::Plot::raincloud(crate::RaincloudInput::geometry(raincloud.clone()))
        .with_alpha(
            crate::alpha::AlphaTarget::Point,
            crate::alpha::AlphaEncoding::observations(
                crate::alpha::AlphaValues::linear(
                    NumericValues::new("confidence", [0.0, 230.0]),
                    (0.0, 255.0),
                    (0, 255),
                )
                .unwrap(),
            ),
        )
        .with_alpha(
            crate::alpha::AlphaTarget::Line,
            crate::alpha::AlphaEncoding::keyed(
                [
                    CategoryValues::new("x", ["b", "a"]).unwrap(),
                    CategoryValues::new("cohort", ["same"; 2]).unwrap(),
                ],
                crate::alpha::AlphaValues::linear(
                    NumericValues::new("summary confidence", [64.0, 128.0]),
                    (0.0, 255.0),
                    (0, 255),
                )
                .unwrap(),
            )
            .unwrap(),
        )
        .with_alpha(
            crate::alpha::AlphaTarget::Fill,
            crate::alpha::AlphaEncoding::keyed(
                [
                    CategoryValues::new("x", ["b", "a"]).unwrap(),
                    CategoryValues::new("cohort", ["same"; 2]).unwrap(),
                ],
                crate::alpha::AlphaValues::linear(
                    NumericValues::new("body confidence", [0.0, 191.0]),
                    (0.0, 255.0),
                    (0, 255),
                )
                .unwrap(),
            )
            .unwrap(),
        );
    assert_eq!(supplied.description().row_count(), 0);
    let theme = crate::Theme::publication().resolve().unwrap();
    let text = crate::text::TextService::default();
    let theme = crate::text::TextRequestContext::new(&theme, &text);
    crate::orchestrator::with_prepared_plot_for_test(
        &supplied.erase(),
        &theme,
        |_| Ok(()),
        |chart, inputs, theme, _operation| {
            let chart = chart.clone();
            let operation = theme.text.prepared_font_environment().unwrap().operation();
            let selected = chart
                .measure_selected_interior(
                    crate::decorations::legend::discrete::scoped::GuideUse::Local,
                    crate::spec::PlotRenderContext::new(
                        inputs.plots[0].data(&operation).unwrap(),
                        crate::layout::PixelRect::new(20.0, 600.0, 20.0, 420.0),
                        theme,
                    )
                    .with_registration(&inputs.plots[0], inputs.intrinsic(&inputs.plots[0])),
                )
                .unwrap();
            let crate::layout::interior::MarkPayload::Distribution(payload) =
                &selected.marks.as_ref().unwrap().payload
            else {
                panic!("Raincloud payload")
            };
            let alphas = payload
                .marks
                .iter()
                .filter_map(|mark| match mark {
                    crate::plots::categorical_distribution::Mark::Scatter(points, style) => {
                        Some((points.len(), style.color.get_alpha()))
                    }
                    _ => None,
                })
                .collect::<Vec<_>>();
            // Summary order is a/b; original point rows are b/a. Alpha follows point rows.
            assert_eq!(alphas, [(1, 230), (1, 0)]);
            let inner_alpha = theme
                .marks
                .get_fills()
                .get_bodies()
                .get_inner_summary()
                .backend_style()
                .color
                .get_alpha();
            assert_eq!(
                payload
                    .marks
                    .iter()
                    .filter_map(|mark| match mark {
                        crate::plots::categorical_distribution::Mark::Polygon(_, style) =>
                            Some(style.color.get_alpha()),
                        _ => None,
                    })
                    .collect::<Vec<_>>(),
                [191, inner_alpha, 0, inner_alpha]
            );
            assert_eq!(
                payload
                    .marks
                    .iter()
                    .filter_map(|mark| match mark {
                        crate::plots::categorical_distribution::Mark::Line(_, style) =>
                            Some(style.color.get_alpha()),
                        _ => None,
                    })
                    .collect::<Vec<_>>(),
                [128, 64]
            );
            let sections = selected
                .discrete_guide
                .as_ref()
                .unwrap()
                .sections
                .iter()
                .filter_map(|section| match &section.body {
                    crate::decorations::legend::discrete::GuideSectionBody::Alpha(alpha) => {
                        Some(alpha)
                    }
                    _ => None,
                })
                .collect::<Vec<_>>();
            assert_eq!(sections.len(), 3);
            assert!(sections.iter().all(|section| section.entries.len() == 5
                && section.sources[0].glyphs[0].members.len() == 2));
            let mut invalid_histogram = histogram;
            invalid_histogram.bin_lower = NumericValues::new("lower", [f64::INFINITY]);
            let mut invalid_raincloud = raincloud;
            invalid_raincloud.density_y = NumericValues::new("y", [f64::INFINITY, -2.0, 2.0, 2.0]);
            assert!(matches!(invalid_histogram.validate(), Err(Error::Data(_))));
            assert!(matches!(
                invalid_raincloud.membership(),
                Err(Error::Data(_))
            ));
        },
    )
    .unwrap();
}

#[test]
fn raincloud_membership_matches_raw_keys_and_retains_child_order() {
    let mut input = raincloud();
    let prepared = input.membership().unwrap();
    assert_eq!(prepared.groups[0].density, [1, 3]);
    assert_eq!(prepared.groups[1].density, [0, 2]);
    assert_eq!(prepared.groups[0].points, [1]);
    assert_eq!(prepared.groups[1].points, [0]);
    input.point_category = CategoryValues::new("point category", ["unknown", "a"]).unwrap();
    assert!(input
        .membership()
        .unwrap_err()
        .to_string()
        .contains("unknown summary group"));
    input.point_category = CategoryValues::new("point category", ["a", "a"]).unwrap();
    assert!(input
        .membership()
        .unwrap_err()
        .to_string()
        .contains("at least one raw point"));
}

#[test]
fn column_domains_reject_misalignment_missing_values_and_incomplete_hue() {
    let mut histogram = histogram();
    histogram.kde_y = NumericValues::new("density", [0.5]);
    assert!(matches!(histogram.validate(), Err(Error::Data(_))));
    histogram.kde_y = NumericValues::nullable("density", [Some(0.5), None]);
    assert!(matches!(histogram.validate(), Err(Error::Data(_))));
    let mut raincloud = raincloud();
    raincloud.density_hue = None;
    assert!(matches!(raincloud.membership(), Err(Error::Data(_))));
    raincloud.density_hue = Some(CategoryValues::new("cohort", ["same"; 4]).unwrap());
    raincloud.q1 = NumericValues::new("q1", [-1.0]);
    assert!(matches!(raincloud.membership(), Err(Error::Data(_))));
}

#[test]
fn geometry_equality_uses_column_contents_names_and_validity() {
    let mut left = histogram();
    let mut right = histogram();
    assert_eq!(left, right);
    right.rug_x = NumericValues::new("different scientific name", [0.0]);
    assert_ne!(left, right);
    left.kde_y = NumericValues::nullable("density", [None, Some(0.5)]);
    right = left.clone();
    assert_eq!(left, right);
    right.kde_y = NumericValues::new("density", [f64::NAN, 0.5]);
    assert_ne!(left, right);
    assert_ne!(right, right.clone());
    assert_eq!(raincloud(), raincloud());
}
