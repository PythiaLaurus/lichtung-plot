//! Point estimates with caller-provided interval bounds on categorical rows.

#![allow(clippy::cast_precision_loss)]

use crate::alpha::{marks::ObservationAlpha, AlphaTarget, AlphaUnitKind};
use crate::Error;

use crate::layout::AxisSpec;
use crate::spec::{CategoryOrder, ValueSort};
use crate::Plot;

#[derive(Debug, PartialEq)]
pub(crate) struct PointIntervalRow {
    pub(crate) label: String,
    pub(crate) estimate: Option<crate::plots::interval_estimate::EstimateTuple>,
    pub(crate) alpha: ObservationAlpha,
}

pub(crate) fn shared_x_domain(
    chart: &Plot,
    data: crate::data::PlotData<'_>,
) -> Result<(f64, f64), Error> {
    let rows = prepare(chart, data)?;
    if let Some(limits) = chart.x_axis_spec().limits {
        return Ok(limits);
    }

    let range =
        crate::plots::interval_estimate::estimate_data_range(rows.iter().map(|row| &row.estimate))
            .ok_or_else(|| {
                Error::Data(
                    "point_interval: all rows are not_estimable; set explicit x-axis limits".into(),
                )
            })?;
    point_interval_display_range(range, chart.x_axis_spec())
}

pub(crate) fn prepare<'a>(
    chart: &Plot,
    data: impl Into<crate::data::PlotData<'a>>,
) -> Result<Vec<PointIntervalRow>, Error> {
    let data = data.into();
    let alpha = chart.prepare_alpha(AlphaTarget::Point, AlphaUnitKind::Observation, &[])?;
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
    crate::data::source::reserve_source(&mut rows, length)?;
    for (index, (label, estimate)) in labels.into_iter().zip(estimates).enumerate() {
        let label = crate::data::source::normalize_source_string(&label)?;
        let source_row = data.source_row(index);
        if label.is_empty() {
            return Err(Error::Data(format!(
                "point_interval: category label is empty at source row {source_row}"
            )));
        }

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
        rows.push(PointIntervalRow {
            label,
            estimate,
            alpha: ObservationAlpha::resolve(data, &alpha, index)?,
        });
    }

    apply_category_order(&mut rows, chart.category_order_spec(), data)?;
    Ok(rows)
}

fn apply_category_order(
    rows: &mut [PointIntervalRow],
    order: &CategoryOrder,
    data: crate::data::PlotData<'_>,
) -> Result<(), Error> {
    match order {
        CategoryOrder::Observed => {}
        CategoryOrder::Explicit(requested) => {
            let requested = crate::data::source::collect_source(
                requested
                    .iter()
                    .map(|label| crate::data::source::normalize_source_string(label)),
            )?;
            for (index, label) in requested.iter().enumerate() {
                if requested[..index].contains(label) {
                    return Err(Error::Data(format!("point_interval category order: explicit category order contains duplicate level {label:?}")));
                }

                if data.is_full_plot() && !rows.iter().any(|row| &row.label == label) {
                    return Err(Error::Data(format!("point_interval category order: explicit category order contains unknown level {label:?}")));
                }
            }
            let mut order = crate::data::source::collect_source(rows.iter().enumerate().map(|(index, row)| {

                let rank = requested.iter().position(|label| label == &row.label).ok_or_else(|| Error::Data(format!("point_interval category order: explicit category order omits observed level {:?}", row.label)))?;
                Ok((rank, index))
            }))?;
            sort_rows(rows, &mut order);
        }
        CategoryOrder::Value(_) if rows.iter().any(|row| row.estimate.is_none()) => {
            return Err(Error::Data(
                "point_interval: estimate-status rows do not support value-derived category order"
                    .into(),
            ));
        }
        CategoryOrder::Value(sort) => {
            let mut order = crate::data::source::collect_source(
                (0..rows.len()).map(|index| Ok((index, index))),
            )?;
            // Original indices break ties, preserving the stable scientific order
            // without the standard stable sort's hidden scratch allocation.

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
            sort_rows(rows, &mut order);
        }
    }
    Ok(())
}

fn sort_rows(rows: &mut [PointIntervalRow], order: &mut [(usize, usize)]) {
    order.sort_unstable();
    for (rank, entry) in order.iter_mut().enumerate() {
        entry.0 = rank;
    }

    order.sort_unstable_by_key(|entry| entry.1);
    // Each source now owns its destination; cycle swaps need no scratch vector.
    for index in 0..order.len() {
        while order[index].0 != index {
            let source = order[index].0;
            rows.swap(index, source);
            order.swap(index, source);
        }
    }
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
    let display = crate::layout::continuous_auto_range(data_range, spec)?;
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
