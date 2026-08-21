use polars::prelude::NamedFrom;

use super::{
    configured_bar_chart, configured_chart, fixture, Backend, Chart, DataFrame, Error, FillStyle,
    LegendPosition, LineStyle, NullBackend, PanelBox, PlotKind, Point2D, RecordingBackend, Series,
};

#[test]
fn mass_spectrum_rejects_null_mz_exactly() {
    let mz = Series::new("mz".into(), [Some(1.0_f64), None, Some(3.0)]);
    let intensity = Series::new("intensity".into(), [2.0_f64, 4.0, 6.0]);
    let data = DataFrame::new(mz.len(), vec![mz.into(), intensity.into()]).unwrap();
    let mut chart = Chart::new(&data);
    let _ = chart.x("mz");
    let _ = chart.y("intensity");
    let _ = chart.select(PlotKind::MassSpectrum);

    let mut backend = RecordingBackend::default();
    let error = chart
        .render_kind(PlotKind::MassSpectrum, &mut backend)
        .unwrap_err();

    assert_eq!(
        error.to_string(),
        "data error: mass_spectrum: column \"mz\" contains null at source row 1"
    );
    assert_eq!(backend.begins, 0);
}

#[test]
fn explicit_inside_legend_with_fixed_limits_rejects_overlap_before_begin() {
    let data = DataFrame::new(
        4,
        vec![
            Series::new("x".into(), [8.0_f64, 10.0, 8.0, 10.0]).into(),
            Series::new("y".into(), [5.0_f64; 4]).into(),
            Series::new("hue".into(), ["a", "a", "b", "b"]).into(),
        ],
    )
    .unwrap();
    let mut chart = Chart::new(&data)
        .group("hue")
        .hue("hue")
        .x_limits(0.0, 10.0)
        .y_limits(0.0, 10.0);
    chart
        .legend_position(LegendPosition::InsideRight)
        .x("x")
        .y("y")
        .line();
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
    let data = DataFrame::new(
        2,
        vec![
            Series::new("x".into(), [1.0_f64, 2.0]).into(),
            Series::new("y".into(), [1.0_f64, 2.0]).into(),
            Series::new(
                "hue".into(),
                [
                    "an exceptionally long first legend label",
                    "an exceptionally long second legend label",
                ],
            )
            .into(),
        ],
    )
    .unwrap();
    let mut chart = Chart::new(&data).hue("hue").x_label("").y_label("");
    chart
        .legend_position(LegendPosition::InsideTopLeft)
        .x("x")
        .y("y")
        .size(180, 140)
        .scatter();
    let mut backend = RecordingBackend::default();

    let error = chart.render_with_backend(&mut backend).unwrap_err();

    assert!(matches!(error, Error::Render(_)));
    assert!(error
        .to_string()
        .contains("explicit inside legend InsideTopLeft cannot fit"));
    assert_eq!(backend.begins, 0);
}

#[test]
fn mass_spectrum_rejects_null_intensity_exactly() {
    let mz = Series::new("mz".into(), [1.0_f64, 2.0, 3.0]);
    let intensity = Series::new("intensity".into(), [Some(2.0_f64), None, Some(6.0)]);
    let data = DataFrame::new(mz.len(), vec![mz.into(), intensity.into()]).unwrap();
    let mut chart = Chart::new(&data);
    let _ = chart.x("mz");
    let _ = chart.y("intensity");
    let _ = chart.select(PlotKind::MassSpectrum);

    let error = chart
        .render_kind(PlotKind::MassSpectrum, &mut RecordingBackend::default())
        .unwrap_err();

    assert_eq!(
        error.to_string(),
        "data error: mass_spectrum: column \"intensity\" contains null at source row 1"
    );
}

#[test]
fn mass_spectrum_rejects_non_finite_scientific_values_exactly() {
    for (column, value) in [("mz", f64::NAN), ("intensity", f64::INFINITY)] {
        let mz = Series::new(
            "mz".into(),
            if column == "mz" {
                [1.0_f64, value, 3.0]
            } else {
                [1.0_f64, 2.0, 3.0]
            },
        );
        let intensity = Series::new(
            "intensity".into(),
            if column == "intensity" {
                [2.0_f64, value, 6.0]
            } else {
                [2.0_f64, 4.0, 6.0]
            },
        );
        let data = DataFrame::new(mz.len(), vec![mz.into(), intensity.into()]).unwrap();
        let mut chart = Chart::new(&data);
        let _ = chart.x("mz");
        let _ = chart.y("intensity");
        let _ = chart.select(PlotKind::MassSpectrum);

        let error = chart
            .render_kind(PlotKind::MassSpectrum, &mut RecordingBackend::default())
            .unwrap_err();

        assert_eq!(
            error.to_string(),
            format!(
                "data error: mass_spectrum: column \"{column}\" contains non-finite value {value} at source row 1"
            )
        );
    }
}

#[test]
fn violin_rejects_null_x_exactly() {
    let group = Series::new("group".into(), [Some("a"), None, Some("a")]);
    let value = Series::new("value".into(), [1.0_f64, 2.0, 3.0]);
    let data = DataFrame::new(group.len(), vec![group.into(), value.into()]).unwrap();
    let mut chart = Chart::new(&data);
    let _ = chart.x("group");
    let _ = chart.y("value");
    let _ = chart.select(PlotKind::Violin);

    let mut backend = RecordingBackend::default();
    let error = chart
        .render_kind(PlotKind::Violin, &mut backend)
        .unwrap_err();

    assert_eq!(
        error.to_string(),
        "data error: violin: column \"group\" contains null at source row 1"
    );
    assert_eq!(backend.begins, 0);
}

#[test]
fn violin_rejects_null_y_exactly() {
    let group = Series::new("group".into(), ["a", "a", "a"]);
    let value = Series::new("value".into(), [Some(1.0_f64), None, Some(3.0)]);
    let data = DataFrame::new(group.len(), vec![group.into(), value.into()]).unwrap();
    let mut chart = Chart::new(&data);
    let _ = chart.x("group");
    let _ = chart.y("value");
    let _ = chart.select(PlotKind::Violin);

    let error = chart
        .render_kind(PlotKind::Violin, &mut RecordingBackend::default())
        .unwrap_err();

    assert_eq!(
        error.to_string(),
        "data error: violin: column \"value\" contains null at source row 1"
    );
}

#[test]
fn violin_rejects_null_hue_exactly() {
    let group = Series::new("group".into(), ["a", "a", "a"]);
    let hue = Series::new("hue".into(), [Some("x"), None, Some("y")]);
    let value = Series::new("value".into(), [1.0_f64, 2.0, 3.0]);
    let data = DataFrame::new(group.len(), vec![group.into(), hue.into(), value.into()]).unwrap();
    let mut chart = Chart::new(&data).hue("hue");
    let _ = chart.x("group");
    let _ = chart.y("value");
    let _ = chart.select(PlotKind::Violin);

    let error = chart
        .render_kind(PlotKind::Violin, &mut RecordingBackend::default())
        .unwrap_err();

    assert_eq!(
        error.to_string(),
        "data error: violin: column \"hue\" contains null at source row 1"
    );
}

#[test]
fn violin_rejects_non_finite_y_exactly() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let group = Series::new("group".into(), ["a", "a", "a"]);
        let measurement = Series::new("value".into(), [1.0_f64, value, 3.0]);
        let data = DataFrame::new(group.len(), vec![group.into(), measurement.into()]).unwrap();
        let mut chart = Chart::new(&data);
        let _ = chart.x("group");
        let _ = chart.y("value");
        let _ = chart.select(PlotKind::Violin);

        let error = chart
            .render_kind(PlotKind::Violin, &mut RecordingBackend::default())
            .unwrap_err();

        assert_eq!(
            error.to_string(),
            format!(
                "data error: violin: column \"value\" contains non-finite value {value} at source row 1"
            )
        );
    }
}

#[test]
fn mass_spectrum_rejects_non_finite_supplemental_peak_before_begin() {
    let mz = Series::new("mz".into(), [1.0_f64, 2.0, 3.0]);
    let intensity = Series::new("intensity".into(), [2.0_f64, 4.0, 6.0]);
    let data = DataFrame::new(mz.len(), vec![mz.into(), intensity.into()]).unwrap();
    let mut chart = Chart::new(&data);
    let _ = chart.add_peaks(vec![(2.0, f64::INFINITY)]);
    let _ = chart.x("mz");
    let _ = chart.y("intensity");
    let _ = chart.select(PlotKind::MassSpectrum);
    let mut backend = RecordingBackend::default();

    let error = chart
        .render_kind(PlotKind::MassSpectrum, &mut backend)
        .unwrap_err();

    assert_eq!(
        error.to_string(),
        "data error: mass_spectrum: supplemental peak 0 must contain finite m/z and intensity values, got (2, inf)"
    );
    assert_eq!(backend.begins, 0);
}

#[test]
fn boxplot_rejects_numeric_string_y_before_begin() {
    let group = Series::new("group".into(), ["a", "a"]);
    let value = Series::new("value".into(), ["1.0", "2.0"]);
    let data = DataFrame::new(group.len(), vec![group.into(), value.into()]).unwrap();
    let mut chart = Chart::new(&data);
    let _ = chart.x("group");
    let _ = chart.y("value");
    let _ = chart.select(PlotKind::Boxplot);
    let mut backend = RecordingBackend::default();

    let error = chart
        .render_kind(PlotKind::Boxplot, &mut backend)
        .unwrap_err();

    assert_eq!(
        error.to_string(),
        "data error: boxplot: column \"value\" must have a numeric dtype, got String"
    );
    assert_eq!(backend.begins, 0);
}

#[test]
fn boxplot_rejects_null_category_and_hue_with_source_rows() {
    let group = Series::new("group".into(), [Some("a"), None]);
    let hue = Series::new("hue".into(), [Some("x"), None]);
    let value = Series::new("value".into(), [1.0_f64, 2.0]);
    let data = DataFrame::new(group.len(), vec![group.into(), hue.into(), value.into()]).unwrap();
    for (hue_column, expected_column) in [(None, "group"), (Some("hue"), "group")] {
        let mut chart =
            hue_column.map_or_else(|| Chart::new(&data), |hue| Chart::new(&data).hue(hue));
        let _ = chart.x("group");
        let _ = chart.y("value");
        let _ = chart.select(PlotKind::Boxplot);
        let mut backend = RecordingBackend::default();

        let error = chart
            .render_kind(PlotKind::Boxplot, &mut backend)
            .unwrap_err();

        assert_eq!(
            error.to_string(),
            format!(
                "data error: boxplot: column \"{expected_column}\" contains null at source row 1"
            )
        );
        assert_eq!(backend.begins, 0);
    }
}

#[test]
fn boxplot_rejects_null_hue_after_valid_categories() {
    let group = Series::new("group".into(), ["a", "a"]);
    let hue = Series::new("hue".into(), [Some("x"), None]);
    let value = Series::new("value".into(), [1.0_f64, 2.0]);
    let data = DataFrame::new(group.len(), vec![group.into(), hue.into(), value.into()]).unwrap();
    let mut chart = Chart::new(&data).hue("hue");
    let _ = chart.x("group");
    let _ = chart.y("value");
    let _ = chart.select(PlotKind::Boxplot);
    let mut backend = RecordingBackend::default();

    let error = chart
        .render_kind(PlotKind::Boxplot, &mut backend)
        .unwrap_err();

    assert_eq!(
        error.to_string(),
        "data error: boxplot: column \"hue\" contains null at source row 1"
    );
    assert_eq!(backend.begins, 0);
}

#[test]
fn boxplot_rejects_nan_and_infinity_with_source_rows_before_begin() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let group = Series::new("group".into(), ["a", "a"]);
        let measurement = Series::new("value".into(), [1.0_f64, value]);
        let data = DataFrame::new(group.len(), vec![group.into(), measurement.into()]).unwrap();
        let mut chart = Chart::new(&data);
        let _ = chart.x("group");
        let _ = chart.y("value");
        let _ = chart.select(PlotKind::Boxplot);
        let mut backend = RecordingBackend::default();

        let error = chart
            .render_kind(PlotKind::Boxplot, &mut backend)
            .unwrap_err();

        assert_eq!(
            error.to_string(),
            format!(
                "data error: boxplot: column \"value\" contains non-finite value {value} at source row 1"
            )
        );
        assert_eq!(backend.begins, 0);
    }
}

#[test]
fn scatter_rejects_null_hue_even_when_numeric_row_would_be_dropped() {
    let x = Series::new("x".into(), [Some(1.0_f64), None]);
    let y = Series::new("y".into(), [Some(2.0_f64), None]);
    let hue = Series::new("hue".into(), [Some("a"), None]);
    let data = DataFrame::new(x.len(), vec![x.into(), y.into(), hue.into()]).unwrap();
    let mut chart = Chart::new(&data).hue("hue");
    let _ = chart.x("x");
    let _ = chart.y("y");
    let _ = chart.select(PlotKind::Scatter);
    let mut backend = RecordingBackend::default();

    let error = chart
        .render_kind(PlotKind::Scatter, &mut backend)
        .unwrap_err();

    assert_eq!(
        error.to_string(),
        "data error: scatter: required categorical column \"hue\" contains null at source row 1"
    );
    assert_eq!(backend.begins, 0);
}

#[test]
fn matrix_rejects_numeric_strings_before_backend_begin() {
    let numeric_string = Series::new("numeric_string".into(), ["1.0", "2.0"]);
    let numeric = Series::new("numeric".into(), [1.0_f64, 2.0]);
    let data = DataFrame::new(2, vec![numeric_string.into(), numeric.into()]).unwrap();
    let mut chart = Chart::new(&data);
    let _ = chart.matrix_variables(vec!["numeric_string", "numeric"]);
    let _ = chart.select(PlotKind::Matrix);
    let mut backend = RecordingBackend::default();

    let error = chart
        .render_kind(PlotKind::Matrix, &mut backend)
        .unwrap_err();

    assert_eq!(
        error.to_string(),
        "data error: matrix: column \"numeric_string\" must have a numeric dtype, got String"
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
        let x = Series::new("x".into(), [Some(1.0_f64), value]);
        let y = Series::new("y".into(), [2.0_f64, 3.0]);
        let data = DataFrame::new(2, vec![x.into(), y.into()]).unwrap();
        let mut chart = Chart::new(&data);
        let _ = chart.matrix_variables(vec!["x", "y"]);
        let _ = chart.select(PlotKind::Matrix);
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
    let x = Series::new("constant".into(), [2.0_f64, 2.0, 2.0]);
    let y = Series::new("varying".into(), [1.0_f64, 2.0, 3.0]);
    let data = DataFrame::new(3, vec![x.into(), y.into()]).unwrap();
    let mut chart = Chart::new(&data);
    let _ = chart.matrix_variables(vec!["constant", "varying"]);
    let _ = chart.select(PlotKind::Matrix);
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
    let x = Series::new("x".into(), [1.0_f64, 2.0, 3.0]);
    let y = Series::new("y".into(), [2.0_f64, 3.0, 4.0]);
    let hue = Series::new("cohort".into(), ["singleton", "valid", "valid"]);
    let data = DataFrame::new(3, vec![x.into(), y.into(), hue.into()]).unwrap();
    let mut chart = Chart::new(&data).group("cohort").hue("cohort");
    let _ = chart.matrix_variables(vec!["x", "y"]);
    let _ = chart.select(PlotKind::Matrix);
    let mut backend = RecordingBackend::default();

    let error = chart
        .render_kind(PlotKind::Matrix, &mut backend)
        .unwrap_err();

    assert_eq!(
        error.to_string(),
        "data error: matrix: column \"x\" in graphical group \"cohort\"=\"singleton\" with hue \"cohort\"=\"singleton\" requires at least 2 observations for Scott bandwidth"
    );
    assert_eq!(backend.begins, 0);
}

#[test]
fn matrix_rejects_unrepresentable_extreme_kde_support_before_begin() {
    let lower = f64::MAX.next_down();
    let x = Series::new("extreme".into(), [lower, f64::MAX]);
    let y = Series::new("ordinary".into(), [1.0_f64, 2.0]);
    let data = DataFrame::new(2, vec![x.into(), y.into()]).unwrap();
    let mut chart = Chart::new(&data);
    let _ = chart.matrix_variables(vec!["extreme", "ordinary"]);
    let _ = chart.select(PlotKind::Matrix);
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
    let group = Series::new("group".into(), ["a", "a"]);
    let value = Series::new("value".into(), [f64::MAX, f64::MAX]);
    let data = DataFrame::new(2, vec![group.into(), value.into()]).unwrap();
    for kind in [PlotKind::Bar, PlotKind::HorizontalBar] {
        let chart = configured_bar_chart(&data, kind);
        let mut backend = RecordingBackend::default();

        chart.render_kind(kind, &mut backend).unwrap();

        assert_eq!((backend.begins, backend.ends), (1, 1), "{kind:?}");
    }
}

#[test]
fn bars_reject_unrepresentable_extreme_confidence_geometry_before_begin() {
    let group = Series::new("group".into(), ["a", "a"]);
    let value = Series::new("value".into(), [-f64::MAX, f64::MAX]);
    let data = DataFrame::new(2, vec![group.into(), value.into()]).unwrap();
    for kind in [PlotKind::Bar, PlotKind::HorizontalBar] {
        let chart = configured_bar_chart(&data, kind);
        let mut backend = RecordingBackend::default();

        let error = chart.render_kind(kind, &mut backend).unwrap_err();

        assert!(error
            .to_string()
            .contains("confidence interval is not numerically representable"));
        assert_eq!(backend.begins, 0, "{kind:?}");
    }
}

#[test]
fn stacked_bars_reject_unrepresentable_offsets_before_begin() {
    let group = Series::new("group".into(), ["a", "a"]);
    let hue = Series::new("hue".into(), ["first", "second"]);
    let value = Series::new("value".into(), [f64::MAX, f64::MAX]);
    let data = DataFrame::new(2, vec![group.into(), hue.into(), value.into()]).unwrap();
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
    let mut backend = NullBackend;
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

    assert!(point_error.to_string().contains("finite coordinates"));
    assert!(style_error.to_string().contains("line width"));
    assert!(clip_error.to_string().contains("clip region"));
    assert!(derived_clip_error
        .to_string()
        .contains("finite positive width"));
}

#[test]
fn every_bar_family_rejects_numeric_strings_before_backend_begin() {
    let group = Series::new("group".into(), ["a", "b"]);
    let hue = Series::new("hue".into(), ["h1", "h2"]);
    let value = Series::new("value".into(), ["1.0", "2.0"]);
    let data = DataFrame::new(2, vec![group.into(), hue.into(), value.into()]).unwrap();

    for kind in [
        PlotKind::Bar,
        PlotKind::HorizontalBar,
        PlotKind::StackedBar,
        PlotKind::StackedHorizontalBar,
    ] {
        let chart = configured_bar_chart(&data, kind);
        let mut backend = RecordingBackend::default();

        let error = chart.render_kind(kind, &mut backend).unwrap_err();

        assert_eq!(
            error.to_string(),
            "data error: bar: column \"value\" must have a numeric dtype, got String"
        );
        assert_eq!(backend.begins, 0, "{kind:?}");
    }
}

#[test]
fn every_bar_family_rejects_missing_and_non_finite_values_before_begin() {
    for value in [
        None,
        Some(f64::NAN),
        Some(f64::INFINITY),
        Some(f64::NEG_INFINITY),
    ] {
        let group = Series::new("group".into(), ["a", "b"]);
        let hue = Series::new("hue".into(), ["h1", "h2"]);
        let measurement = Series::new("value".into(), [Some(1.0_f64), value]);
        let data = DataFrame::new(2, vec![group.into(), hue.into(), measurement.into()]).unwrap();
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
fn injected_marginal_coordinates_jointly_drop_nan_pairs() {
    let data = fixture();
    let mut chart = configured_chart(&data, PlotKind::Marginal);
    let _ = chart.add_marginal_coords(vec![(f64::NAN, 1.0), (2.0, f64::NAN), (3.0, 4.0)]);
    let mut backend = RecordingBackend::default();

    chart.render_kind(PlotKind::Marginal, &mut backend).unwrap();

    assert_eq!((backend.begins, backend.scatter_points.len()), (1, 1));
}

#[test]
fn injected_marginal_coordinates_reject_either_infinity_before_begin() {
    for coords in [
        vec![(1.0, 2.0), (f64::INFINITY, 3.0)],
        vec![(1.0, 2.0), (3.0, f64::NEG_INFINITY)],
        vec![(f64::NAN, f64::INFINITY)],
    ] {
        let data = fixture();
        let mut chart = configured_chart(&data, PlotKind::Marginal);
        let _ = chart.add_marginal_coords(coords);
        let mut backend = RecordingBackend::default();

        let error = chart
            .render_kind(PlotKind::Marginal, &mut backend)
            .unwrap_err();

        assert!(error.to_string().contains("contains infinity"));
        assert_eq!(backend.begins, 0);
    }
}

#[test]
fn injected_marginal_coordinates_error_when_joint_drop_removes_every_point() {
    let data = fixture();
    let mut chart = configured_chart(&data, PlotKind::Marginal);
    let _ = chart.add_marginal_coords(vec![(f64::NAN, 1.0), (2.0, f64::NAN)]);
    let mut backend = RecordingBackend::default();

    let error = chart
        .render_kind(PlotKind::Marginal, &mut backend)
        .unwrap_err();

    assert_eq!(
        error.to_string(),
        "data error: marginal: no injected points remain after jointly dropping NaN pairs"
    );
    assert_eq!(backend.begins, 0);
}

#[test]
fn ungrouped_marginal_rejects_unrepresentable_finite_extreme_kde_before_begin() {
    let lower = f64::MAX.next_down();
    let data = DataFrame::new(
        2,
        vec![
            Series::new("x".into(), [lower, f64::MAX]).into(),
            Series::new("y".into(), [1.0_f64, 2.0]).into(),
        ],
    )
    .unwrap();
    let mut chart = Chart::new(&data);
    let _ = chart.x("x").y("y").marginal();
    let mut backend = RecordingBackend::default();

    let error = chart
        .render_kind(PlotKind::Marginal, &mut backend)
        .unwrap_err();
    let message = error.to_string();

    assert!(message.contains("marginal: x distribution for group \"\""));
    assert!(
        message.contains("support/grid is not representable") || message.contains("kde:"),
        "{message}"
    );
    assert_eq!((backend.begins, backend.ends), (0, 0));
}

#[test]
fn grouped_marginal_rejects_unrepresentable_finite_extreme_kde_before_begin() {
    let lower = f64::MAX.next_down();
    let data = DataFrame::new(
        4,
        vec![
            Series::new("x".into(), [lower, f64::MAX, 1.0, 2.0]).into(),
            Series::new("y".into(), [1.0_f64, 2.0, 3.0, 4.0]).into(),
            Series::new("hue".into(), ["extreme", "extreme", "ordinary", "ordinary"]).into(),
        ],
    )
    .unwrap();
    let mut chart = Chart::new(&data).group("hue").hue("hue");
    let _ = chart.x("x").y("y").marginal();
    let mut backend = RecordingBackend::default();

    let error = chart
        .render_kind(PlotKind::Marginal, &mut backend)
        .unwrap_err();

    assert!(error
        .to_string()
        .contains("marginal: x distribution for group \"extreme\""));
    assert!(error
        .to_string()
        .contains("support/grid is not representable"));
    assert_eq!((backend.begins, backend.ends), (0, 0));
}

#[test]
fn numeric_categories_remain_supported_by_beeswarm_and_waterfall() {
    let category = Series::new("category".into(), [1_i64, 2]);
    let x = Series::new("x".into(), [1.0_f64, 2.0]);
    let color = Series::new("color".into(), [0.2_f64, 0.8]);
    let delta = Series::new("delta".into(), [2.0_f64, -1.0]);
    let base = Series::new("base".into(), [10.0_f64, 10.0]);
    let terminal = Series::new("terminal".into(), [11.0_f64, 11.0]);
    let data = DataFrame::new(
        category.len(),
        vec![
            category.into(),
            x.into(),
            color.into(),
            delta.into(),
            base.into(),
            terminal.into(),
        ],
    )
    .unwrap();

    let mut beeswarm = Chart::new(&data).color("color");
    let _ = beeswarm.x("x");
    let _ = beeswarm.y("category");
    let _ = beeswarm.select(PlotKind::Beeswarm);
    beeswarm
        .render_kind(PlotKind::Beeswarm, &mut RecordingBackend::default())
        .unwrap();

    let mut waterfall = Chart::new(&data).waterfall_totals("base", "terminal");
    let _ = waterfall.x("category");
    let _ = waterfall.y("delta");
    let _ = waterfall.select(PlotKind::Waterfall);
    waterfall
        .render_kind(PlotKind::Waterfall, &mut RecordingBackend::default())
        .unwrap();
}
