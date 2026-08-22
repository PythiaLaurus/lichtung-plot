//! Point estimates with caller-provided interval bounds on categorical rows.

#![allow(clippy::cast_precision_loss)]

use std::collections::HashSet;

use crate::Error;

use crate::layout::{AxisSpec, AxisTick, Layout};
use crate::render::backend::{with_clip, Backend, LineStyle, Point2D};
use crate::spec::{PlotRenderContext, PlotRenderResult, ValueSort};
use crate::Plot;

const CAP_HALF_HEIGHT_PX: f64 = 4.0;

#[derive(Debug, PartialEq)]
struct PointIntervalRow {
    label: String,
    estimate: Option<crate::plots::interval_estimate::EstimateTuple>,
}

#[expect(
    clippy::too_many_lines,
    reason = "point-interval range, layout, and estimated/unavailable row geometry share one renderer boundary"
)]
pub(crate) fn render(
    chart: &Plot,
    context: PlotRenderContext<'_>,
    backend: &mut dyn Backend,
) -> Result<PlotRenderResult, Error> {
    let rows = prepare(chart, context.data)?;
    let estimates = rows.iter().map(|row| row.estimate).collect::<Vec<_>>();
    let x_data_range = crate::plots::interval_estimate::estimate_data_range(&estimates);
    let facet_fallback = if x_data_range.is_none() && chart.x_axis_spec().limits.is_none() {
        crate::plots::interval_estimate::fixed_facet_estimate_tuples(
            chart,
            context.data,
            "point_interval",
            true,
        )?
        .and_then(|estimates| crate::plots::interval_estimate::estimate_data_range(&estimates))
    } else {
        None
    };
    let x_range = match x_data_range {
        Some(range) => point_interval_display_range(range, chart.x_axis_spec())?,
        None => match (chart.x_axis_spec().limits, facet_fallback) {
            (Some(limits), _) => limits,
            (None, Some(range)) if range.0 < range.1 => range,
            (None, Some(range)) => point_interval_display_range(range, chart.x_axis_spec())?,
            (None, None) => {
                return Err(Error::Data(
                    "point_interval: all rows are not_estimable; set explicit x-axis limits".into(),
                ))
            }
        },
    };
    let x_axis_spec = crate::layout::display_axis_spec(chart.x_axis_spec(), x_range);
    let row_count = rows.len() as f64;
    let y_axis_spec = AxisSpec::default().limits(0.0, row_count);
    let labels = rows.iter().map(|row| row.label.clone()).collect::<Vec<_>>();
    let display_labels =
        chart.resolve_label_domain("y-axis categories", labels.iter().map(String::as_str))?;
    let y_ticks = labels
        .iter()
        .enumerate()
        .map(|(index, label)| AxisTick::new(row_y(index, rows.len()), label.clone()))
        .collect();
    let panel = crate::plots::horizontal_bar::panel_box_for_y_tick_labels(
        chart,
        context.panel,
        &display_labels,
    )?;
    let layout = Layout::new_with_ticks_and_axis_specs_in_panel_box(
        x_range,
        (0.0, row_count),
        panel,
        chart.x_label_text().unwrap_or("").to_string(),
        chart.y_label_text().unwrap_or("").to_string(),
        Vec::new(),
        y_ticks,
        &x_axis_spec,
        &y_axis_spec,
    );
    let palette = chart.palette();
    let interval_style = LineStyle::solid(palette.foreground, 1.4);
    let zero_line_x = (chart.zero_line_visible() && x_range.0 <= 0.0 && x_range.1 >= 0.0)
        .then(|| layout.projection.project(0.0, 0.0).0);

    with_clip(backend, layout.projection.panel_box(), |backend| {
        if let Some(x) = zero_line_x {
            let (_, y0) = layout.projection.project(0.0, 0.0);
            let (_, y1) = layout.projection.project(0.0, row_count);
            backend.draw_line(
                &[Point2D::new(x, y0), Point2D::new(x, y1)],
                &LineStyle::solid(palette.foreground, 0.8),
            )?;
        }

        for (index, row) in rows.iter().enumerate() {
            let y = row_y(index, rows.len());
            let Some(tuple) = row.estimate else {
                crate::plots::interval_estimate::draw_not_estimable(
                    chart,
                    &layout,
                    y,
                    zero_line_x,
                    backend,
                )?;
                continue;
            };
            let lower = tuple.lower.ok_or_else(|| {
                Error::Render("point_interval: prepared lower bound is absent".into())
            })?;
            let upper = tuple.upper.ok_or_else(|| {
                Error::Render("point_interval: prepared upper bound is absent".into())
            })?;
            let visible_lower = lower.max(x_range.0);
            let visible_upper = upper.min(x_range.1);
            let (_, center_y) = layout
                .projection
                .project(tuple.estimate.clamp(x_range.0, x_range.1), y);
            if visible_lower <= visible_upper {
                let (lower_x, _) = layout.projection.project(visible_lower, y);
                let (upper_x, _) = layout.projection.project(visible_upper, y);
                backend.draw_line(
                    &[
                        Point2D::new(lower_x, center_y),
                        Point2D::new(upper_x, center_y),
                    ],
                    &interval_style,
                )?;
            }
            for bound in [lower, upper] {
                if bound >= x_range.0 && bound <= x_range.1 {
                    let (x, _) = layout.projection.project(bound, y);
                    backend.draw_line(
                        &[
                            Point2D::new(x, center_y - CAP_HALF_HEIGHT_PX),
                            Point2D::new(x, center_y + CAP_HALF_HEIGHT_PX),
                        ],
                        &interval_style,
                    )?;
                }
            }
            if tuple.estimate >= x_range.0 && tuple.estimate <= x_range.1 {
                let (estimate_x, _) = layout.projection.project(tuple.estimate, y);
                backend.draw_scatter(
                    &[Point2D::new(estimate_x, center_y)],
                    &chart.point_style(palette.colors[0], 4),
                )?;
            }
        }
        Ok(())
    })?;

    Ok(PlotRenderResult::new(layout, Vec::new()))
}

fn prepare<'a>(
    chart: &Plot,
    data: impl Into<crate::data::PlotData<'a>>,
) -> Result<Vec<PointIntervalRow>, Error> {
    let data = data.into();
    if chart.y_axis_spec() != &AxisSpec::default() {
        return Err(Error::Data(
            "point_interval: categorical y-axis limits and tick-count overrides are not supported"
                .into(),
        ));
    }
    if let Some(hue_col) = chart.hue_col() {
        return Err(Error::Data(format!(
            "point_interval: hue column {hue_col:?} is not supported"
        )));
    }
    if let Some(color_col) = chart.color_col() {
        return Err(Error::Data(format!(
            "point_interval: color column {color_col:?} is not supported"
        )));
    }
    let category_col = chart
        .y_col()
        .ok_or_else(|| Error::Data("point_interval: y category column not set".into()))?;
    let labels = crate::data::strict_strings(data, category_col, "point_interval")?;
    let estimates = crate::plots::interval_estimate::prepare_estimate_tuples(
        chart,
        data,
        "point_interval",
        true,
    )?;
    let length = labels.len();
    if length == 0 || estimates.len() != length {
        return Err(Error::Data(
            "point_interval: category/estimate/lower/upper columns must have equal non-zero lengths"
                .into(),
        ));
    }

    let mut rendered_labels = HashSet::with_capacity(length);
    let mut rows = Vec::with_capacity(length);
    for (index, (label, estimate)) in labels.into_iter().zip(estimates).enumerate() {
        let label = crate::data::collapse_whitespace(&label);
        let source_row = data.source_row(index);
        if label.is_empty() {
            return Err(Error::Data(format!(
                "point_interval: category label is empty at source row {source_row}"
            )));
        }
        if !rendered_labels.insert(label.clone()) {
            return Err(Error::Data(format!(
                "point_interval: duplicate rendered category label {label:?} at source row {source_row}"
            )));
        }
        if let Some(tuple) = estimate {
            let lower = tuple.lower.ok_or_else(|| {
                Error::Render("point_interval: prepared lower bound is absent".into())
            })?;
            let upper = tuple.upper.ok_or_else(|| {
                Error::Render("point_interval: prepared upper bound is absent".into())
            })?;
            if lower > tuple.estimate || tuple.estimate > upper {
                return Err(Error::Data(format!(
                    "point_interval: source row {source_row} must satisfy lower <= estimate <= upper; got {lower} <= {} <= {upper}",
                    tuple.estimate
                )));
            }
        }
        rows.push(PointIntervalRow { label, estimate });
    }

    if rows.iter().any(|row| row.estimate.is_none()) && chart.value_sort_spec() != ValueSort::None {
        return Err(Error::Data(
            "point_interval: estimate-status rows retain source order and do not support value sorting"
                .into(),
        ));
    }
    match chart.value_sort_spec() {
        ValueSort::None => {}
        ValueSort::Ascending => {
            rows.sort_by(|left, right| match (left.estimate, right.estimate) {
                (Some(left), Some(right)) => left.estimate.total_cmp(&right.estimate),
                _ => std::cmp::Ordering::Equal,
            });
        }
        ValueSort::Descending => {
            rows.sort_by(|left, right| match (left.estimate, right.estimate) {
                (Some(left), Some(right)) => right.estimate.total_cmp(&left.estimate),
                _ => std::cmp::Ordering::Equal,
            });
        }
    }
    Ok(rows)
}

fn row_y(index: usize, row_count: usize) -> f64 {
    row_count as f64 - index as f64 - 0.5
}

fn point_interval_display_range(
    data_range: (f64, f64),
    spec: &AxisSpec,
) -> Result<(f64, f64), Error> {
    if let Some(limits) = spec.limits {
        return Ok(limits);
    }
    let display = crate::layout::continuous_auto_range(data_range, spec);
    if display.0.is_finite() && display.1.is_finite() && display.0 < display.1 {
        return Ok(display);
    }
    let same_value = data_range.0.total_cmp(&data_range.1).is_eq()
        || (data_range.0.abs().to_bits() == 0 && data_range.1.abs().to_bits() == 0);
    if same_value {
        let value = data_range.0;
        let lower = value.next_down();
        let upper = value.next_up();
        if lower.is_finite() && lower < value {
            return Ok(if upper.is_finite() && value < upper {
                (lower, upper)
            } else {
                (lower, value)
            });
        }
        if upper.is_finite() && value < upper {
            return Ok((value, upper));
        }
        return Err(Error::Data(format!(
            "point_interval: constant value {} cannot produce a finite representable automatic x-axis range; set explicit limits",
            data_range.0
        )));
    }
    if data_range.0.is_finite() && data_range.1.is_finite() && data_range.0 < data_range.1 {
        Ok(data_range)
    } else {
        Err(Error::Data(
            "point_interval: automatic x-axis range is not finite and ascending; set explicit limits"
                .into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use polars::prelude::{DataFrame, NamedFrom, Series};

    use super::*;

    fn data(labels: Series, estimates: Series, lowers: Series, uppers: Series) -> DataFrame {
        DataFrame::new(
            labels.len(),
            vec![
                labels.into(),
                estimates.into(),
                lowers.into(),
                uppers.into(),
            ],
        )
        .unwrap()
    }

    fn valid_data() -> DataFrame {
        data(
            Series::new("category".into(), ["first", "second"]),
            Series::new("estimate".into(), [2.0_f64, 1.0]),
            Series::new("lower".into(), [1.5_f64, 0.0]),
            Series::new("upper".into(), [4.0_f64, 1.5]),
        )
    }

    fn chart(data: &DataFrame) -> Plot {
        Plot::point_interval(
            crate::test_support::category(data, "category"),
            crate::test_support::numeric(data, "estimate"),
            crate::test_support::numeric(data, "lower"),
            crate::test_support::numeric(data, "upper"),
        )
    }

    fn prepare_chart(chart: &Plot) -> Result<Vec<PointIntervalRow>, Error> {
        prepare(chart, chart)
    }

    #[test]
    fn prepare_preserves_rows_by_default() {
        let data = valid_data();

        let rows = prepare_chart(&chart(&data)).unwrap();

        assert_eq!(
            rows.iter()
                .map(|row| row.label.as_str())
                .collect::<Vec<_>>(),
            ["first", "second"]
        );
    }

    #[test]
    fn prepare_sorts_rows_by_estimate() {
        let data = valid_data();

        let rows = prepare_chart(&chart(&data).value_sort(ValueSort::Ascending)).unwrap();

        assert_eq!(
            rows.iter()
                .map(|row| row.label.as_str())
                .collect::<Vec<_>>(),
            ["second", "first"]
        );
    }

    #[test]
    fn prepare_rejects_empty_data() {
        let data = data(
            Series::new("category".into(), Vec::<String>::new()),
            Series::new("estimate".into(), Vec::<f64>::new()),
            Series::new("lower".into(), Vec::<f64>::new()),
            Series::new("upper".into(), Vec::<f64>::new()),
        );

        assert!(prepare_chart(&chart(&data)).is_err());
    }

    #[test]
    fn prepare_rejects_empty_category_labels() {
        let data = data(
            Series::new("category".into(), ["first", " "]),
            Series::new("estimate".into(), [1.0_f64, 2.0]),
            Series::new("lower".into(), [0.0_f64, 1.0]),
            Series::new("upper".into(), [2.0_f64, 3.0]),
        );

        assert!(prepare_chart(&chart(&data))
            .unwrap_err()
            .to_string()
            .contains("label is empty"));
    }

    #[test]
    fn prepare_rejects_duplicate_rendered_labels() {
        let data = data(
            Series::new("category".into(), ["same label", " same  label "]),
            Series::new("estimate".into(), [1.0_f64, 2.0]),
            Series::new("lower".into(), [0.0_f64, 1.0]),
            Series::new("upper".into(), [2.0_f64, 3.0]),
        );

        assert!(prepare_chart(&chart(&data))
            .unwrap_err()
            .to_string()
            .contains("duplicate rendered"));
    }

    #[test]
    fn prepare_rejects_null_categories() {
        let data = data(
            Series::new("category".into(), [Some("first"), None]),
            Series::new("estimate".into(), [1.0_f64, 2.0]),
            Series::new("lower".into(), [0.0_f64, 1.0]),
            Series::new("upper".into(), [2.0_f64, 3.0]),
        );

        assert!(prepare_chart(&chart(&data)).is_err());
    }

    #[test]
    fn prepare_rejects_null_and_non_finite_numbers() {
        for estimates in [
            Series::new("estimate".into(), [Some(1.0_f64), None]),
            Series::new("estimate".into(), [1.0_f64, f64::INFINITY]),
        ] {
            let data = data(
                Series::new("category".into(), ["first", "second"]),
                estimates,
                Series::new("lower".into(), [0.0_f64, 1.0]),
                Series::new("upper".into(), [2.0_f64, 3.0]),
            );
            assert!(prepare_chart(&chart(&data)).is_err());
        }
    }

    #[test]
    fn prepare_rejects_bounds_that_do_not_contain_estimate() {
        let data = data(
            Series::new("category".into(), ["first"]),
            Series::new("estimate".into(), [2.0_f64]),
            Series::new("lower".into(), [0.0_f64]),
            Series::new("upper".into(), [1.0_f64]),
        );

        assert!(prepare_chart(&chart(&data))
            .unwrap_err()
            .to_string()
            .contains("lower <= estimate <= upper"));
    }

    #[test]
    fn prepare_rejects_hue_and_color() {
        let data = valid_data();
        let hue_error =
            prepare_chart(&chart(&data).hue(crate::test_support::category(&data, "category")))
                .unwrap_err();
        let color_error =
            prepare_chart(&chart(&data).color(crate::test_support::numeric(&data, "estimate")))
                .unwrap_err();

        assert!(
            hue_error.to_string().contains("hue column")
                && color_error.to_string().contains("color column")
        );
    }

    #[test]
    fn prepare_rejects_categorical_y_axis_overrides() {
        let data = valid_data();

        let error = prepare_chart(&chart(&data).y_limits(0.0, 1.0))
            .unwrap_err()
            .to_string();

        assert!(error.contains("categorical y-axis"));
    }

    #[test]
    fn display_range_falls_back_to_finite_extreme_endpoints() {
        let spec = AxisSpec::default();

        assert_eq!(
            point_interval_display_range((-f64::MAX, f64::MAX), &spec).unwrap(),
            (-f64::MAX, f64::MAX)
        );
        assert_eq!(
            point_interval_display_range((f64::MAX, f64::MAX), &spec).unwrap(),
            (f64::MAX.next_down(), f64::MAX)
        );
        let signed_zero = point_interval_display_range((-0.0, 0.0), &spec).unwrap();
        assert!(signed_zero.0.is_finite() && signed_zero.0 < 0.0);
        assert!(signed_zero.1.is_finite() && signed_zero.1 > 0.0);
    }
}
