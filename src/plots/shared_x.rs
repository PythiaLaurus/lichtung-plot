//! Font-independent scientific domains and categorical axis sources.

mod prepared;
pub(crate) use prepared::{prepare_line_plot, prepare_plot, prepare_scatter_plot, PreparedAxes};

use crate::spec::PlotKind;
use crate::{Error, Plot};

#[cfg(test)]
thread_local! {
    pub(crate) static DOMAIN_PREPARATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    pub(crate) static AXIS_PREPARATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub(crate) fn domain(
    chart: &Plot,
    resources: &crate::font::canonical::OperationGenerationResources,
) -> Result<(f64, f64), Error> {
    #[cfg(test)]
    DOMAIN_PREPARATIONS.with(|count| count.set(count.get() + 1));
    let data = crate::data::PlotData::from(chart).with_axis_resources(resources);
    domain_with_data(chart, data)
}

pub(super) fn domain_with_data(
    chart: &Plot,
    data: crate::data::PlotData<'_>,
) -> Result<(f64, f64), Error> {
    data.claim_source_visits(data.height())?;
    // One scalar range result and at most five private nice-range candidates.
    data.claim_source_visits(6)?;
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
            )
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
            let (_, range) = super::bar_data::axis_inputs(chart, data, mode, None)?;
            crate::layout::anchored_auto_range(range, chart.x_axis_spec(), 0.0)
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
            )
        }
        PlotKind::Waterfall => super::waterfall::shared_x_domain(chart, data)?,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Bounds, CategoryValues, NumericValues};

    fn domain(chart: &Plot) -> Result<(f64, f64), Error> {
        super::domain(
            chart,
            &crate::font::canonical::OperationGenerationResources::new(),
        )
    }

    #[test]
    fn family_domains_are_scientific_coordinates_without_fonts() {
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        let histogram = Plot::histogram(crate::HistogramInput::samples(NumericValues::new(
            "value",
            [5.0, 5.0],
        )))
        .erase();
        assert_eq!(domain(&histogram).unwrap(), (4.5, 5.5));
        let qq = |values| Plot::qq(NumericValues::new("observed", values)).erase();
        assert_eq!(
            domain(&qq([1.0, 2.0, 3.0, 4.0])).unwrap(),
            domain(&qq([100.0, 200.0, 300.0, 400.0])).unwrap()
        );
        let forest = Plot::forest(
            CategoryValues::new("term", ["a", "b"]).unwrap(),
            NumericValues::new("estimate", [1.0, 100.0]),
            Bounds::new(
                NumericValues::new("lower", [0.1, 10.0]),
                NumericValues::new("upper", [10.0, 1000.0]),
            ),
            1.0,
        )
        .scale(crate::AxisScale::Log10)
        .erase();
        assert_eq!(
            domain(&forest).unwrap(),
            crate::layout::continuous_auto_range((-1.0, 3.0), &crate::layout::AxisSpec::default())
        );
        let waterfall = Plot::waterfall(
            CategoryValues::new("step", ["a", "b"]).unwrap(),
            NumericValues::new("delta", [20.0, -40.0]),
            10.0,
        )
        .erase();
        assert_eq!(
            domain(&waterfall).unwrap(),
            crate::layout::continuous_auto_range(
                (-10.0, 30.0),
                &crate::layout::AxisSpec::default()
            )
        );
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }

    #[test]
    fn shared_numeric_sources_refuse_before_fonts_and_materialization() {
        use crate::font::canonical::OperationGenerationResources;
        let plots = [
            Plot::scatter(
                NumericValues::new("x", [1.0, 2.0]),
                NumericValues::new("y", [2.0, 3.0]),
            )
            .erase(),
            Plot::regression(
                NumericValues::new("x", [1.0, 2.0]),
                NumericValues::new("y", [2.0, 3.0]),
            )
            .erase(),
            Plot::density(NumericValues::new("x", [1.0, 2.0])).erase(),
            Plot::qq(NumericValues::new("x", [1.0, 2.0])).erase(),
            Plot::waterfall(
                CategoryValues::new("step", ["a", "b"]).unwrap(),
                NumericValues::new("delta", [20.0, -40.0]),
                10.0,
            )
            .erase(),
        ];
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        for plot in &plots {
            for (visits, bytes, expected) in
                [(0, 1024, "work admission"), (100, 0, "storage admission")]
            {
                let resources = OperationGenerationResources::with_test_limits(visits, bytes);
                let error = super::domain(plot, &resources).unwrap_err();
                assert!(matches!(error, Error::Render(_)), "{error}");
                assert!(error.to_string().contains(expected), "{error}");
            }
        }
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }

    #[test]
    fn density_sample_storage_is_admitted_after_channel_preparation() {
        let plot = Plot::density(NumericValues::new("x", [1.0, 2.0, 3.0])).erase();
        let before_samples = 3
            * (std::mem::size_of::<f64>()
                + std::mem::size_of::<usize>()
                + 2 * std::mem::size_of::<crate::data::CollectivePair>());
        let resources = crate::font::canonical::OperationGenerationResources::with_test_limits(
            1000,
            u64::try_from(before_samples).unwrap(),
        );
        let error = super::domain(&plot, &resources).unwrap_err();
        assert!(
            error.to_string().contains("axis source storage admission"),
            "{error}"
        );
        assert_eq!(resources.usage().unwrap().1, 26);
    }
}
