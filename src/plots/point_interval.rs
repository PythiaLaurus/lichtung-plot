//! Point estimates with caller-provided interval bounds on categorical rows.

#![allow(clippy::cast_precision_loss)]

use crate::Error;

use crate::layout::{AxisSpec, AxisTick};
use crate::render::backend::{with_clip, Backend, LineStyle, Point2D};
use crate::spec::{CategoryOrder, PlotRenderContext, PlotRenderResult, ValueSort};
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
) -> Result<PlotRenderResult, crate::layout::fit::PlanError> {
    let rows = prepare(chart, context.data)?;
    let row_count = rows.len() as f64;
    let panel = context.panel;
    let layout = context.axis_layout(|| {
        let estimates = rows.iter().map(|row| row.estimate).collect::<Vec<_>>();
        let x_data_range = crate::plots::interval_estimate::estimate_data_range(&estimates);
        let facet_fallback = if x_data_range.is_none() && chart.x_axis_spec().limits.is_none() {
            crate::plots::interval_estimate::fixed_facet_estimate_tuples(
                chart,
                context,
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
                        "point_interval: all rows are not_estimable; set explicit x-axis limits"
                            .into(),
                    )
                    .into())
                }
            },
        };
        let x_axis_spec = crate::layout::display_axis_spec(chart.x_axis_spec(), x_range);
        let y_axis_spec = AxisSpec::default().limits(0.0, row_count);
        let labels = rows.iter().map(|row| row.label.clone()).collect::<Vec<_>>();
        let y_ticks = labels
            .iter()
            .enumerate()
            .map(|(index, label)| AxisTick::new(row_y(index, rows.len()), label.clone()))
            .collect();
        context.layout_with_ticks(
            x_range,
            (0.0, row_count),
            panel,
            chart.x_label_text().unwrap_or("").to_string(),
            chart.y_label_text().unwrap_or("").to_string(),
            Vec::new(),
            y_ticks,
            &x_axis_spec,
            &y_axis_spec,
            context.theme,
        )
    })?;
    let x_range = layout.projection.x_range();
    let theme = context.theme;
    let interval_style = LineStyle::solid(theme.foreground, 1.4);
    let zero_line_x = (chart.zero_line_visible() && x_range.0 <= 0.0 && x_range.1 >= 0.0)
        .then(|| layout.projection.project(0.0, 0.0).0);
    let not_estimable_plan = rows
        .iter()
        .position(|row| row.estimate.is_none())
        .map(|index| {
            crate::plots::interval_estimate::plan_not_estimable(
                theme,
                &layout,
                zero_line_x,
                &context
                    .fit_scope()
                    .child(crate::layout::AddressRole::CategoricalValueLabel, index)?,
            )
        })
        .transpose()?;

    with_clip(backend, layout.projection.panel_box(), |backend| {
        if let Some(x) = zero_line_x {
            let (_, y0) = layout.projection.project(0.0, 0.0);
            let (_, y1) = layout.projection.project(0.0, row_count);
            backend.draw_line(
                &[Point2D::new(x, y0), Point2D::new(x, y1)],
                &LineStyle::solid(theme.foreground, 0.8),
            )?;
        }

        for (index, row) in rows.iter().enumerate() {
            let y = row_y(index, rows.len());
            let Some(tuple) = row.estimate else {
                let (plan, x) = not_estimable_plan
                    .as_ref()
                    .expect("unavailable rows have a measured status plan");
                crate::plots::interval_estimate::draw_planned_not_estimable(
                    plan, *x, &layout, y, backend,
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
                    &chart.point_style(theme.colors[0], 4),
                )?;
            }
        }
        Ok(())
    })?;

    Ok(PlotRenderResult::new(layout, Vec::new()))
}

pub(crate) fn shared_x_domain(
    chart: &Plot,
    data: crate::data::PlotData<'_>,
) -> Result<(f64, f64), Error> {
    let rows = prepare(chart, data)?;
    if let Some(limits) = chart.x_axis_spec().limits {
        return Ok(limits);
    }
    data.claim_source_visits(rows.len())?;
    let range =
        crate::plots::interval_estimate::estimate_data_range(rows.iter().map(|row| &row.estimate))
            .ok_or_else(|| {
                Error::Data(
                    "point_interval: all rows are not_estimable; set explicit x-axis limits".into(),
                )
            })?;
    point_interval_display_range(range, chart.x_axis_spec())
}

pub(crate) fn axis_categories(
    chart: &Plot,
    data: crate::data::PlotData<'_>,
) -> Result<Vec<String>, Error> {
    data.collect_source(prepare(chart, data)?.into_iter().map(|row| Ok(row.label)))
}

fn prepare<'a>(
    chart: &Plot,
    data: impl Into<crate::data::PlotData<'a>>,
) -> Result<Vec<PointIntervalRow>, Error> {
    let data = data.into();
    if chart.y_axis_spec() != &AxisSpec::default() {
        return Err(Error::Data(
            "point_interval: categorical y-axis limits are not supported".into(),
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

    let mut rows: Vec<PointIntervalRow> = Vec::new();
    data.reserve_source(&mut rows, length)?;
    for (index, (label, estimate)) in labels.into_iter().zip(estimates).enumerate() {
        let label = data.normalize_source_string(&label)?;
        let source_row = data.source_row(index);
        if label.is_empty() {
            return Err(Error::Data(format!(
                "point_interval: category label is empty at source row {source_row}"
            )));
        }
        data.claim_source_visits(rows.len())?;
        if rows.iter().any(|row| row.label == label) {
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

    apply_category_order(&mut rows, chart.category_order_spec(), data)?;
    Ok(rows)
}

fn apply_category_order(
    rows: &mut [PointIntervalRow],
    order: &CategoryOrder,
    data: crate::data::PlotData<'_>,
) -> Result<(), Error> {
    if matches!(order, CategoryOrder::Value(_)) {
        data.claim_source_visits(rows.len())?;
    }
    match order {
        CategoryOrder::Observed => {}
        CategoryOrder::Explicit(requested) => {
            let requested = data.collect_source(
                requested
                    .iter()
                    .map(|label| data.normalize_source_string(label)),
            )?;
            for (index, label) in requested.iter().enumerate() {
                data.claim_source_visits(index)?;
                if requested[..index].contains(label) {
                    return Err(Error::Data(format!("point_interval category order: explicit category order contains duplicate level {label:?}")));
                }
                data.claim_source_visits(rows.len())?;
                if data.is_full_plot() && !rows.iter().any(|row| &row.label == label) {
                    return Err(Error::Data(format!("point_interval category order: explicit category order contains unknown level {label:?}")));
                }
            }
            let mut order = data.collect_source(rows.iter().enumerate().map(|(index, row)| {
                data.claim_source_visits(requested.len())?;
                let rank = requested.iter().position(|label| label == &row.label).ok_or_else(|| Error::Data(format!("point_interval category order: explicit category order omits observed level {:?}", row.label)))?;
                Ok((rank, index))
            }))?;
            sort_rows(rows, &mut order, data)?;
        }
        CategoryOrder::Value(_) if rows.iter().any(|row| row.estimate.is_none()) => {
            return Err(Error::Data(
                "point_interval: estimate-status rows do not support value-derived category order"
                    .into(),
            ));
        }
        CategoryOrder::Value(sort) => {
            let mut order = data.collect_source((0..rows.len()).map(|index| Ok((index, index))))?;
            // Original indices break ties, preserving the stable scientific order
            // without the standard stable sort's hidden scratch allocation.
            let work = rows
                .len()
                .checked_mul(rows.len().max(1).ilog2() as usize + 1)
                .and_then(|work| work.checked_mul(4))
                .ok_or_else(|| Error::Render("axis source sort work overflow".into()))?;
            data.claim_source_visits(work)?;
            order.sort_unstable_by(|left, right| {
                let left_value = rows[left.0]
                    .estimate
                    .expect("estimated rows validated")
                    .estimate;
                let right_value = rows[right.0]
                    .estimate
                    .expect("estimated rows validated")
                    .estimate;
                (match sort {
                    ValueSort::Ascending => left_value.total_cmp(&right_value),
                    ValueSort::Descending => right_value.total_cmp(&left_value),
                })
                .then(left.0.cmp(&right.0))
            });
            for (rank, entry) in order.iter_mut().enumerate() {
                entry.0 = rank;
            }
            sort_rows(rows, &mut order, data)?;
        }
    }
    Ok(())
}

fn sort_rows(
    rows: &mut [PointIntervalRow],
    order: &mut [(usize, usize)],
    data: crate::data::PlotData<'_>,
) -> Result<(), Error> {
    let work = order
        .len()
        .checked_mul(order.len().max(1).ilog2() as usize + 1)
        .and_then(|work| work.checked_mul(4))
        .ok_or_else(|| Error::Render("axis source sort work overflow".into()))?;
    data.claim_source_visits(work)?;
    order.sort_unstable();
    for (rank, entry) in order.iter_mut().enumerate() {
        entry.0 = rank;
    }
    data.claim_source_visits(work)?;
    order.sort_unstable_by_key(|entry| entry.1);
    // Each source now owns its destination; cycle swaps need no scratch vector.
    for index in 0..order.len() {
        while order[index].0 != index {
            data.claim_source_visits(1)?;
            let source = order[index].0;
            rows.swap(index, source);
            order.swap(index, source);
        }
    }
    Ok(())
}

fn row_y(index: usize, row_count: usize) -> f64 {
    row_count as f64 - index as f64 - 0.5
}

pub(crate) fn point_interval_display_range(
    data_range: (f64, f64),
    spec: &AxisSpec,
) -> Result<(f64, f64), Error> {
    #[cfg(test)]
    crate::orchestrator::measurements::count(
        crate::orchestrator::measurements::Counter::AxisRangePreparation,
    );
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
    use super::*;
    use crate::{CategoryValues, NumericValues};

    struct PointIntervalValues {
        category: CategoryValues,
        estimate: NumericValues,
        lower: NumericValues,
        upper: NumericValues,
    }

    fn valid_data() -> PointIntervalValues {
        PointIntervalValues {
            category: CategoryValues::new("category", ["first", "second"]).unwrap(),
            estimate: NumericValues::new("estimate", [2.0, 1.0]),
            lower: NumericValues::new("lower", [1.5, 0.0]),
            upper: NumericValues::new("upper", [4.0, 1.5]),
        }
    }

    fn chart(data: &PointIntervalValues) -> Plot<crate::kind::PointInterval> {
        Plot::point_interval(
            data.category.clone(),
            data.estimate.clone(),
            crate::Bounds::new(data.lower.clone(), data.upper.clone()),
        )
    }

    fn prepare_chart(
        chart: &Plot<crate::kind::PointInterval>,
    ) -> Result<Vec<PointIntervalRow>, Error> {
        let chart = chart.clone().erase();
        prepare(&chart, &chart)
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
    fn category_order_admits_before_normalization_or_sort_and_preserves_cycles_and_ties() {
        use crate::font::canonical::OperationGenerationResources;
        let plot = chart(&valid_data()).erase();
        let make_rows = || {
            [("a", 3.0), ("b", 1.0), ("c", 1.0), ("d", 2.0)]
                .into_iter()
                .map(|(label, estimate)| PointIntervalRow {
                    label: label.into(),
                    estimate: Some(crate::plots::interval_estimate::EstimateTuple {
                        estimate,
                        lower: Some(0.0),
                        upper: Some(4.0),
                    }),
                })
                .collect::<Vec<_>>()
        };
        for (order, expected) in [
            (
                CategoryOrder::explicit([" b ", "c", "a", "d"]),
                ["b", "c", "a", "d"],
            ),
            (
                CategoryOrder::Value(ValueSort::Ascending),
                ["b", "c", "d", "a"],
            ),
            (
                CategoryOrder::Value(ValueSort::Descending),
                ["a", "d", "b", "c"],
            ),
        ] {
            for (visits, bytes) in [(0, 4096), (4096, 0)] {
                let resources = OperationGenerationResources::with_test_limits(visits, bytes);
                let mut rows = make_rows();
                assert!(matches!(
                    apply_category_order(
                        &mut rows,
                        &order,
                        plot.data().with_axis_resources(&resources)
                    ),
                    Err(Error::Render(_))
                ));
                assert_eq!(
                    rows.iter()
                        .map(|row| row.label.as_str())
                        .collect::<Vec<_>>(),
                    ["a", "b", "c", "d"]
                );
            }
            let resources = OperationGenerationResources::new();
            let mut rows = make_rows();
            apply_category_order(
                &mut rows,
                &order,
                plot.data().with_axis_resources(&resources),
            )
            .unwrap();
            assert_eq!(
                rows.iter()
                    .map(|row| row.label.as_str())
                    .collect::<Vec<_>>(),
                expected
            );
        }
    }

    #[test]
    fn estimate_tuple_storage_refuses_after_numeric_channels() {
        let plot = chart(&valid_data()).erase();
        let resources =
            crate::font::canonical::OperationGenerationResources::with_test_limits(1000, 3 * 2 * 8);
        let error = crate::plots::interval_estimate::prepare_estimate_tuples(
            &plot,
            plot.data().with_axis_resources(&resources),
            "point_interval",
            true,
        )
        .unwrap_err();
        assert!(
            error.to_string().contains("axis source storage admission"),
            "{error}"
        );
    }

    #[test]
    fn prepare_sorts_rows_by_estimate() {
        let data = valid_data();

        let rows = prepare_chart(&chart(&data).category_order(ValueSort::Ascending)).unwrap();

        assert_eq!(
            rows.iter()
                .map(|row| row.label.as_str())
                .collect::<Vec<_>>(),
            ["second", "first"]
        );
    }

    #[test]
    fn prepare_applies_explicit_category_order() {
        let data = valid_data();

        let rows = prepare_chart(
            &chart(&data).category_order(crate::CategoryOrder::explicit(["second", "first"])),
        )
        .unwrap();

        assert_eq!(
            rows.iter()
                .map(|row| row.label.as_str())
                .collect::<Vec<_>>(),
            ["second", "first"]
        );
    }

    #[test]
    fn prepare_rejects_empty_data() {
        let data = PointIntervalValues {
            category: CategoryValues::new("category", Vec::<String>::new()).unwrap(),
            estimate: NumericValues::new("estimate", []),
            lower: NumericValues::new("lower", []),
            upper: NumericValues::new("upper", []),
        };

        assert!(prepare_chart(&chart(&data)).is_err());
    }

    #[test]
    fn prepare_rejects_empty_category_labels() {
        let data = PointIntervalValues {
            category: CategoryValues::new("category", ["first", " "]).unwrap(),
            estimate: NumericValues::new("estimate", [1.0, 2.0]),
            lower: NumericValues::new("lower", [0.0, 1.0]),
            upper: NumericValues::new("upper", [2.0, 3.0]),
        };

        assert!(prepare_chart(&chart(&data))
            .unwrap_err()
            .to_string()
            .contains("label is empty"));
    }

    #[test]
    fn prepare_rejects_duplicate_rendered_labels() {
        let data = PointIntervalValues {
            category: CategoryValues::new("category", ["same label", " same  label "]).unwrap(),
            estimate: NumericValues::new("estimate", [1.0, 2.0]),
            lower: NumericValues::new("lower", [0.0, 1.0]),
            upper: NumericValues::new("upper", [2.0, 3.0]),
        };

        assert!(prepare_chart(&chart(&data))
            .unwrap_err()
            .to_string()
            .contains("duplicate rendered"));
    }

    #[test]
    fn prepare_rejects_null_categories() {
        let data = PointIntervalValues {
            category: CategoryValues::nullable("category", [Some("first"), None]).unwrap(),
            estimate: NumericValues::new("estimate", [1.0, 2.0]),
            lower: NumericValues::new("lower", [0.0, 1.0]),
            upper: NumericValues::new("upper", [2.0, 3.0]),
        };

        assert!(prepare_chart(&chart(&data)).is_err());
    }

    #[test]
    fn prepare_rejects_null_and_non_finite_numbers() {
        for estimate in [
            NumericValues::nullable("estimate", [Some(1.0), None]),
            NumericValues::new("estimate", [1.0, f64::INFINITY]),
        ] {
            let data = PointIntervalValues {
                category: CategoryValues::new("category", ["first", "second"]).unwrap(),
                estimate,
                lower: NumericValues::new("lower", [0.0, 1.0]),
                upper: NumericValues::new("upper", [2.0, 3.0]),
            };
            assert!(prepare_chart(&chart(&data)).is_err());
        }
    }

    #[test]
    fn prepare_rejects_bounds_that_do_not_contain_estimate() {
        let data = PointIntervalValues {
            category: CategoryValues::new("category", ["first"]).unwrap(),
            estimate: NumericValues::new("estimate", [2.0]),
            lower: NumericValues::new("lower", [0.0]),
            upper: NumericValues::new("upper", [1.0]),
        };

        assert!(prepare_chart(&chart(&data))
            .unwrap_err()
            .to_string()
            .contains("lower <= estimate <= upper"));
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
