//! Font-independent scientific domains and categorical axis sources.

mod prepared;
pub(crate) use prepared::{
    prepare_line_plot, prepare_plot, prepare_reference_plot, prepare_scatter_plot, PreparedAxes,
    PreparedReference,
};

use crate::spec::PlotKind;
use crate::{Error, Plot};

#[cfg(test)]
thread_local! {
    pub(crate) static DOMAIN_PREPARATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    pub(crate) static AXIS_PREPARATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub(crate) fn domain(
    chart: &Plot,
    resources: &crate::font::canonical::OperationContext,
) -> Result<(f64, f64), Error> {
    #[cfg(test)]
    DOMAIN_PREPARATIONS.with(|count| count.set(count.get() + 1));
    let data = crate::data::PlotData::from(chart).with_operation(resources);
    domain_with_data(chart, data)
}

pub(super) fn domain_with_data(
    chart: &Plot,
    data: crate::data::PlotData<'_>,
) -> Result<(f64, f64), Error> {
    let range = match chart.kind() {
        PlotKind::Scatter | PlotKind::Regression => {
            let points = crate::data::aligned_finite_f64(
                data,
                chart
                    .x_col()
                    .ok_or_else(|| Error::Data("shared x: x column not set".into()))?,
                chart
                    .y_col()
                    .ok_or_else(|| Error::Data("shared x: y column not set".into()))?,
                "shared x",
            )?;

            crate::layout::continuous_auto_range(
                crate::data::min_max(points.iter().map(|point| point.0), "shared x")?,
                chart.x_axis_spec(),
            )?
        }
        PlotKind::Histogram => super::histogram::shared_x_domain(chart, data)?,
        PlotKind::Density => super::density::shared_x_domain(chart, data)?,
        PlotKind::Qq => super::qq::shared_x_domain(chart, data)?,
        PlotKind::Line | PlotKind::Trajectory | PlotKind::LinePath => {
            super::line::shared_x_domain(chart, data)?
        }
        PlotKind::PointInterval => super::point_interval::shared_x_domain(chart, data)?,
        PlotKind::Coefficient | PlotKind::Forest => {
            super::interval_estimate::shared_x_domain(chart, data)?
        }
        PlotKind::Dumbbell => super::dumbbell::shared_x_domain(chart, data)?,
        PlotKind::HorizontalBar | PlotKind::StackedHorizontalBar => {
            let mode = if chart.kind() == PlotKind::StackedHorizontalBar {
                super::bar_data::BarMode::Stacked
            } else if chart.group_col().is_some() {
                super::bar_data::BarMode::Grouped
            } else {
                super::bar_data::BarMode::Single
            };
            let range = super::bar_data::prepare(chart, data, mode)?.value_range;
            crate::layout::anchored_auto_range(range, chart.x_axis_spec(), 0.0)?
        }
        PlotKind::Marginal => super::marginal::shared_x_domain(chart, data)?,
        PlotKind::Beeswarm => {
            let values = crate::data::strict_finite_f64(
                data,
                chart
                    .x_col()
                    .ok_or_else(|| Error::Data("beeswarm: x column not set".into()))?,
                "beeswarm",
            )?;

            crate::layout::continuous_auto_range(
                crate::data::min_max(values.iter().copied(), "beeswarm")?,
                chart.x_axis_spec(),
            )?
        }
        PlotKind::Waterfall => {
            let marks = super::waterfall::prepare(chart, data)?;
            crate::layout::continuous_auto_range(marks.range, chart.x_axis_spec())?
        }
        PlotKind::Interaction
        | PlotKind::Heatmap
        | PlotKind::Bar
        | PlotKind::StackedBar
        | PlotKind::Boxplot
        | PlotKind::Violin
        | PlotKind::Raincloud
        | PlotKind::Matrix => {
            return Err(Error::Data(
                "canvas x-range group requires a continuous single x axis".into(),
            ));
        }
    };
    if !range.0.is_finite() || !range.1.is_finite() || range.0 >= range.1 {
        return Err(Error::Data(
            "canvas x-range group did not produce a finite ascending range".into(),
        ));
    }
    Ok(range)
}
