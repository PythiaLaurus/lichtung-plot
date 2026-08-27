use super::{
    configured_bar_chart, Backend, BarValues, Error, FillStyle, LegendPolicy, LegendPosition,
    LineStyle, NullBackend, PanelBox, Plot, PlotKind, Point2D, RecordingBackend,
};
use crate::{CategoryValues, NumericValues};

#[test]
fn line_rejects_null_x_before_begin() {
    let chart = Plot::line(
        NumericValues::nullable("mz", [Some(1.0), None, Some(3.0)]),
        NumericValues::new("intensity", [2.0, 4.0, 6.0]),
    );

    let mut backend = RecordingBackend::default();
    let error = chart.render_kind(PlotKind::Line, &mut backend).unwrap_err();

    assert!(error
        .to_string()
        .contains("value \"mz\" contains null at source row 1"));
    assert_eq!(backend.begins, 0);
}

#[test]
fn explicit_inside_legend_with_fixed_limits_rejects_overlap_before_begin() {
    let hue = CategoryValues::new("hue", ["a", "a", "b", "b"]).unwrap();
    let chart = Plot::line(
        NumericValues::new("x", [8.0, 10.0, 8.0, 10.0]),
        NumericValues::new("y", [5.0; 4]),
    )
    .group(hue.clone())
    .hue(hue)
    .x_limits(0.0, 10.0)
    .y_limits(0.0, 10.0)
    .legend(LegendPolicy::Visible(LegendPosition::InsideRight));
    let mut backend = RecordingBackend::default();

    let error = chart.render_with_backend(&mut backend).unwrap_err();

    assert!(matches!(error, Error::Render(_)));
    assert!(error
        .to_string()
        .contains("explicit inside legend InsideRight cannot fit"));
    assert_eq!(backend.begins, 0);
}

#[test]
fn oversized_explicit_inside_legend_rejects_before_begin() {
    let chart = Plot::scatter(
        NumericValues::new("x", [1.0, 2.0]),
        NumericValues::new("y", [1.0, 2.0]),
    )
    .hue(
        CategoryValues::new(
            "hue",
            [
                "an exceptionally long first legend label",
                "an exceptionally long second legend label",
            ],
        )
        .unwrap(),
    )
    .x_label("")
    .y_label("")
    .legend(LegendPolicy::Visible(LegendPosition::InsideTopLeft))
    .size(180, 140);
    let mut backend = RecordingBackend::default();

    let error = chart.render_with_backend(&mut backend).unwrap_err();

    assert!(matches!(error, Error::Render(_)));
    assert!(error
        .to_string()
        .contains("explicit inside legend InsideTopLeft cannot fit"));
    assert_eq!(backend.begins, 0);
}

#[test]
fn line_rejects_null_y_before_begin() {
    let chart = Plot::line(
        NumericValues::new("mz", [1.0, 2.0, 3.0]),
        NumericValues::nullable("intensity", [Some(2.0), None, Some(6.0)]),
    );

    let error = chart
        .render_kind(PlotKind::Line, &mut RecordingBackend::default())
        .unwrap_err();

    assert!(error
        .to_string()
        .contains("value \"intensity\" contains null at source row 1"));
}

#[test]
fn line_rejects_non_finite_scientific_values_before_begin() {
    for (column, value) in [("mz", f64::NAN), ("intensity", f64::INFINITY)] {
        let mz = if column == "mz" {
            [1.0, value, 3.0]
        } else {
            [1.0, 2.0, 3.0]
        };
        let intensity = if column == "intensity" {
            [2.0, value, 6.0]
        } else {
            [2.0, 4.0, 6.0]
        };
        let chart = Plot::line(
            NumericValues::new("mz", mz),
            NumericValues::new("intensity", intensity),
        );

        let error = chart
            .render_kind(PlotKind::Line, &mut RecordingBackend::default())
            .unwrap_err();

        assert!(error.to_string().contains(&format!(
            "value \"{column}\" contains non-finite value {value} at source row 1"
        )));
    }
}

#[test]
fn violin_rejects_null_x_exactly() {
    let chart = Plot::violin(
        CategoryValues::nullable("group", [Some("a"), None, Some("a")]).unwrap(),
        NumericValues::new("value", [1.0, 2.0, 3.0]),
    );

    let mut backend = RecordingBackend::default();
    let error = chart
        .render_kind(PlotKind::Violin, &mut backend)
        .unwrap_err();

    assert_eq!(
        error.to_string(),
        "data error: violin: value \"group\" contains null at source row 1"
    );
    assert_eq!(backend.begins, 0);
}

#[test]
fn violin_rejects_null_y_exactly() {
    let chart = Plot::violin(
        CategoryValues::new("group", ["a", "a", "a"]).unwrap(),
        NumericValues::nullable("value", [Some(1.0), None, Some(3.0)]),
    );

    let error = chart
        .render_kind(PlotKind::Violin, &mut RecordingBackend::default())
        .unwrap_err();

    assert_eq!(
        error.to_string(),
        "data error: violin: value \"value\" contains null at source row 1"
    );
}

#[test]
fn violin_rejects_null_positioned_series_exactly() {
    let chart = Plot::violin(
        CategoryValues::new("group", ["a", "a", "a"]).unwrap(),
        NumericValues::new("value", [1.0, 2.0, 3.0]),
    )
    .series(CategoryValues::nullable("series", [Some("x"), None, Some("y")]).unwrap());

    let error = chart
        .render_kind(PlotKind::Violin, &mut RecordingBackend::default())
        .unwrap_err();

    assert_eq!(
        error.to_string(),
        "data error: violin: value \"series\" contains null at source row 1"
    );
}

#[test]
fn violin_rejects_non_finite_y_exactly() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let chart = Plot::violin(
            CategoryValues::new("group", ["a", "a", "a"]).unwrap(),
            NumericValues::new("value", [1.0, value, 3.0]),
        );

        let error = chart
            .render_kind(PlotKind::Violin, &mut RecordingBackend::default())
            .unwrap_err();

        assert_eq!(
            error.to_string(),
            format!(
                "data error: violin: value \"value\" contains non-finite value {value} at source row 1"
            )
        );
    }
}

#[test]
fn line_rejects_non_finite_supplemental_point_before_begin() {
    let layer = crate::PointLayer::new(
        crate::NumericValues::new("peak m/z", [2.0]),
        crate::NumericValues::new("peak intensity", [f64::INFINITY]),
    )
    .color((180, 45, 40))
    .opacity(0.9)
    .unwrap();
    let chart = Plot::line(
        NumericValues::new("mz", [1.0, 2.0, 3.0]),
        NumericValues::new("intensity", [2.0, 4.0, 6.0]),
    )
    .point_layer(layer);
    let mut backend = RecordingBackend::default();

    let error = chart.render_kind(PlotKind::Line, &mut backend).unwrap_err();

    assert!(error.to_string().contains("non-finite value inf"));
    assert_eq!(backend.begins, 0);
}

#[test]
fn boxplot_rejects_null_category_before_positioned_series_with_source_rows() {
    let category = CategoryValues::nullable("category", [Some("a"), None]).unwrap();
    let series = CategoryValues::nullable("series", [Some("x"), None]).unwrap();
    let value = NumericValues::new("value", [1.0, 2.0]);
    for with_series in [false, true] {
        let mut chart = Plot::boxplot(category.clone(), value.clone());
        if with_series {
            chart = chart.series(series.clone());
        }
        let mut backend = RecordingBackend::default();

        let error = chart
            .render_kind(PlotKind::Boxplot, &mut backend)
            .unwrap_err();

        assert_eq!(
            error.to_string(),
            "data error: boxplot: value \"category\" contains null at source row 1"
        );
        assert_eq!(backend.begins, 0);
    }
}

#[test]
fn boxplot_rejects_null_positioned_series_after_valid_categories() {
    let chart = Plot::boxplot(
        CategoryValues::new("category", ["a", "a"]).unwrap(),
        NumericValues::new("value", [1.0, 2.0]),
    )
    .series(CategoryValues::nullable("series", [Some("x"), None]).unwrap());
    let mut backend = RecordingBackend::default();

    let error = chart
        .render_kind(PlotKind::Boxplot, &mut backend)
        .unwrap_err();

    assert_eq!(
        error.to_string(),
        "data error: boxplot: value \"series\" contains null at source row 1"
    );
    assert_eq!(backend.begins, 0);
}

#[test]
fn boxplot_rejects_nan_and_infinity_with_source_rows_before_begin() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let chart = Plot::boxplot(
            CategoryValues::new("group", ["a", "a"]).unwrap(),
            NumericValues::new("value", [1.0, value]),
        );
        let mut backend = RecordingBackend::default();

        let error = chart
            .render_kind(PlotKind::Boxplot, &mut backend)
            .unwrap_err();

        assert_eq!(
            error.to_string(),
            format!(
                "data error: boxplot: value \"value\" contains non-finite value {value} at source row 1"
            )
        );
        assert_eq!(backend.begins, 0);
    }
}

#[test]
fn scatter_rejects_null_hue_even_when_numeric_row_would_be_dropped() {
    let chart = Plot::scatter(
        NumericValues::nullable("x", [Some(1.0), None]),
        NumericValues::nullable("y", [Some(2.0), None]),
    )
    .hue(CategoryValues::nullable("hue", [Some("a"), None]).unwrap());
    let mut backend = RecordingBackend::default();

    let error = chart
        .render_kind(PlotKind::Scatter, &mut backend)
        .unwrap_err();

    assert_eq!(
        error.to_string(),
        "data error: scatter: value \"hue\" contains null at source row 1"
    );
    assert_eq!(backend.begins, 0);
}

#[test]
fn matrix_rejects_every_missing_or_non_finite_numeric_value_before_begin() {
    for value in [
        None,
        Some(f64::NAN),
        Some(f64::INFINITY),
        Some(f64::NEG_INFINITY),
    ] {
        let chart = Plot::matrix(vec![
            NumericValues::nullable("x", [Some(1.0), value]),
            NumericValues::new("y", [2.0, 3.0]),
        ]);
        let mut backend = RecordingBackend::default();

        let error = chart
            .render_kind(PlotKind::Matrix, &mut backend)
            .unwrap_err();

        assert!(error.to_string().contains("source row 1"));
        assert_eq!(backend.begins, 0);
    }
}

#[test]
fn matrix_rejects_constant_ungrouped_column_before_backend_begin() {
    let chart = Plot::matrix(vec![
        NumericValues::new("constant", [2.0, 2.0, 2.0]),
        NumericValues::new("varying", [1.0, 2.0, 3.0]),
    ]);
    let mut backend = RecordingBackend::default();

    let error = chart
        .render_kind(PlotKind::Matrix, &mut backend)
        .unwrap_err();

    assert_eq!(
        error.to_string(),
        "data error: matrix: column \"constant\" in the ungrouped data has constant values; Scott bandwidth requires nonzero scale"
    );
    assert_eq!(backend.begins, 0);
}

#[test]
fn matrix_rejects_degenerate_hue_group_with_column_and_group_context_before_begin() {
    let hue = CategoryValues::new("cohort", ["singleton", "valid", "valid"]).unwrap();
    let chart = Plot::matrix(vec![
        NumericValues::new("x", [1.0, 2.0, 3.0]),
        NumericValues::new("y", [2.0, 3.0, 4.0]),
    ])
    .group(hue.clone())
    .hue(hue);
    let mut backend = RecordingBackend::default();

    let error = chart
        .render_kind(PlotKind::Matrix, &mut backend)
        .unwrap_err();

    assert_eq!(
        error.to_string(),
        "data error: matrix: column \"x\" in graphical group \"cohort\"=\"singleton\" requires at least 2 observations for Scott bandwidth"
    );
    assert_eq!(backend.begins, 0);
}

#[test]
fn matrix_rejects_unrepresentable_extreme_kde_support_before_begin() {
    let lower = f64::MAX.next_down();
    let chart = Plot::matrix(vec![
        NumericValues::new("extreme", [lower, f64::MAX]),
        NumericValues::new("ordinary", [1.0, 2.0]),
    ]);
    let mut backend = RecordingBackend::default();

    let error = chart
        .render_kind(PlotKind::Matrix, &mut backend)
        .unwrap_err();

    assert!(error
        .to_string()
        .contains("column \"extreme\" in the ungrouped data"));
    assert_eq!(backend.begins, 0);
}

#[test]
fn bar_mean_of_repeated_maximum_values_remains_renderable() {
    let data = BarValues {
        category: CategoryValues::new("group", ["a", "a"]).unwrap(),
        value: NumericValues::new("value", [f64::MAX, f64::MAX]),
        series: None,
    };
    for kind in [PlotKind::Bar, PlotKind::HorizontalBar] {
        let chart = configured_bar_chart(&data, kind);
        let mut backend = RecordingBackend::default();

        chart.render_kind(kind, &mut backend).unwrap();

        assert_eq!((backend.begins, backend.ends), (1, 1), "{kind:?}");
    }
}

#[test]
fn student_t_bars_reject_unrepresentable_extreme_confidence_geometry_before_begin() {
    let category = CategoryValues::new("group", ["a", "a"]).unwrap();
    let value = NumericValues::new("value", [-f64::MAX, f64::MAX]);
    for kind in [PlotKind::Bar, PlotKind::HorizontalBar] {
        let input = crate::BarInput::samples(value.clone())
            .error_bar(crate::ErrorBar::confidence_interval().student_t());
        let chart = match kind {
            PlotKind::Bar => Plot::bar(category.clone(), input).erase(),
            PlotKind::HorizontalBar => Plot::horizontal_bar(category.clone(), input).erase(),
            _ => unreachable!(),
        };
        let mut backend = RecordingBackend::default();

        let error = chart.render_kind(kind, &mut backend).unwrap_err();

        assert!(error.to_string().contains("not numerically representable"));
        assert_eq!(backend.begins, 0, "{kind:?}");
    }
}

#[test]
fn stacked_bars_reject_unrepresentable_offsets_before_begin() {
    let data = BarValues {
        category: CategoryValues::new("group", ["a", "a"]).unwrap(),
        value: NumericValues::new("value", [f64::MAX, f64::MAX]),
        series: Some(CategoryValues::new("hue", ["first", "second"]).unwrap()),
    };
    for kind in [PlotKind::StackedBar, PlotKind::StackedHorizontalBar] {
        let chart = configured_bar_chart(&data, kind);
        let mut backend = RecordingBackend::default();

        let error = chart.render_kind(kind, &mut backend).unwrap_err();

        assert!(error.to_string().contains("stack geometry"));
        assert_eq!(backend.begins, 0, "{kind:?}");
    }
}

#[test]
fn null_backend_rejects_non_finite_geometry_and_styles() {
    let mut backend = NullBackend::default();
    backend.begin(100, 100).unwrap();
    let point_error = backend
        .draw_polygon(
            &[
                Point2D::new(0.0, 0.0),
                Point2D::new(f64::NAN, 1.0),
                Point2D::new(1.0, 0.0),
            ],
            &FillStyle {
                color: (0, 0, 0),
                stroke: (0, 0, 0),
                stroke_width: 1.0,
                opacity: 1.0,
            },
        )
        .unwrap_err();
    let style_error = backend
        .draw_line(
            &[Point2D::new(0.0, 0.0), Point2D::new(1.0, 1.0)],
            &LineStyle::solid((0, 0, 0), f32::NAN),
        )
        .unwrap_err();
    let clip_error = backend
        .begin_clip(PanelBox::new(0.0, f64::INFINITY, 0.0, 1.0))
        .unwrap_err();
    let derived_clip_error = backend
        .begin_clip(PanelBox::new(-f64::MAX, f64::MAX, 0.0, 1.0))
        .unwrap_err();
    backend.end().unwrap();

    assert!(point_error.to_string().contains("finite coordinates"));
    assert!(style_error.to_string().contains("line width"));
    assert!(clip_error.to_string().contains("clip region"));
    assert!(derived_clip_error
        .to_string()
        .contains("finite positive width"));
}

#[test]
fn every_bar_family_rejects_missing_and_non_finite_values_before_begin() {
    for value in [
        None,
        Some(f64::NAN),
        Some(f64::INFINITY),
        Some(f64::NEG_INFINITY),
    ] {
        let data = BarValues {
            category: CategoryValues::new("group", ["a", "b"]).unwrap(),
            value: NumericValues::nullable("value", [Some(1.0), value]),
            series: Some(CategoryValues::new("hue", ["h1", "h2"]).unwrap()),
        };
        for kind in [
            PlotKind::Bar,
            PlotKind::HorizontalBar,
            PlotKind::StackedBar,
            PlotKind::StackedHorizontalBar,
        ] {
            let chart = configured_bar_chart(&data, kind);
            let mut backend = RecordingBackend::default();

            let error = chart.render_kind(kind, &mut backend).unwrap_err();

            assert!(error.to_string().contains("source row 1"), "{kind:?}");
            assert_eq!(backend.begins, 0, "{kind:?}");
        }
    }
}

#[test]
fn ungrouped_marginal_rejects_unrepresentable_finite_extreme_kde_before_begin() {
    let lower = f64::MAX.next_down();
    let chart = Plot::marginal(
        NumericValues::new("x", [lower, f64::MAX]),
        NumericValues::new("y", [1.0, 2.0]),
    );
    let mut backend = RecordingBackend::default();

    let error = chart
        .render_kind(PlotKind::Marginal, &mut backend)
        .unwrap_err();
    let message = error.to_string();

    assert!(message.contains("marginal: x distribution for ungrouped series"));
    assert!(
        message.contains("not numerically representable"),
        "{message}"
    );
    assert_eq!((backend.begins, backend.ends), (0, 0));
}

#[test]
fn grouped_marginal_rejects_unrepresentable_finite_extreme_kde_before_begin() {
    let lower = f64::MAX.next_down();
    let hue = CategoryValues::new("hue", ["extreme", "extreme", "ordinary", "ordinary"]).unwrap();
    let chart = Plot::marginal(
        NumericValues::new("x", [lower, f64::MAX, 1.0, 2.0]),
        NumericValues::new("y", [1.0, 2.0, 3.0, 4.0]),
    )
    .group(hue.clone())
    .hue(hue);
    let mut backend = RecordingBackend::default();

    let error = chart
        .render_kind(PlotKind::Marginal, &mut backend)
        .unwrap_err();

    assert!(error
        .to_string()
        .contains("marginal: x distribution for graphical group \"extreme\""));
    assert!(error.to_string().contains("not numerically representable"));
    assert_eq!((backend.begins, backend.ends), (0, 0));
}

#[test]
fn numeric_category_labels_remain_supported_by_beeswarm_and_waterfall() {
    let category = CategoryValues::new("category", ["1", "2"]).unwrap();

    let beeswarm = Plot::beeswarm(
        category.clone(),
        NumericValues::new("x", [1.0, 2.0]),
        NumericValues::new("color", [0.2, 0.8]),
    );
    beeswarm
        .render_kind(PlotKind::Beeswarm, &mut RecordingBackend::default())
        .unwrap();

    let waterfall = Plot::waterfall(category, NumericValues::new("delta", [2.0, -1.0]), 10.0);
    waterfall
        .render_kind(PlotKind::Waterfall, &mut RecordingBackend::default())
        .unwrap();
}
