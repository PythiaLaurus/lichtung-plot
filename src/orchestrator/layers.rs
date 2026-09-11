//! Validation and rendering for supplemental numeric geometry.

use crate::Error;

use crate::layout::AxisTick;
use crate::render::backend::{with_clip, Backend, Point2D};
use crate::spec::{AxisScale, PlotKind, PlotRenderResult};
use crate::{GuideSpec, Plot};

impl Plot {
    pub(crate) fn projected_explicit_x_ticks(
        &self,
        data: crate::data::PlotData<'_>,
    ) -> Result<Option<Vec<AxisTick>>, Error> {
        self.explicit_x_ticks()
            .map(|ticks| projected_x_ticks(self, ticks, data))
            .transpose()
    }
    pub(crate) fn validate_supplemental_configuration(&self, kind: PlotKind) -> Result<(), Error> {
        let configured = self.explicit_x_ticks().is_some()
            || self.explicit_y_ticks().is_some()
            || !self.guides().is_empty()
            || !self.point_layers().is_empty()
            || !self.bar_point_layers().is_empty();
        if configured && self.has_facets() {
            return Err(Error::Data(
                "explicit numeric ticks, guides, and point layers are not supported with faceting in this focused phase"
                    .into(),
            ));
        }

        let (numeric_x, numeric_y) = numeric_axes(self, kind);
        if self.explicit_x_ticks().is_some() && !numeric_x {
            return Err(Error::Data(format!(
                "{kind:?}: explicit numeric x ticks require a numeric x axis"
            )));
        }
        if self.explicit_y_ticks().is_some() && !numeric_y {
            return Err(Error::Data(format!(
                "{kind:?}: explicit numeric y ticks require a numeric y axis"
            )));
        }
        for guide in self.guides() {
            match guide {
                GuideSpec::Horizontal { .. } if !numeric_y => {
                    return Err(Error::Data(format!(
                        "{kind:?}: a horizontal guide requires a numeric y axis"
                    )));
                }
                GuideSpec::Vertical { .. } if !numeric_x => {
                    return Err(Error::Data(format!(
                        "{kind:?}: a vertical guide requires a numeric x axis"
                    )));
                }
                GuideSpec::Segment { .. } if !numeric_x || !numeric_y => {
                    return Err(Error::Data(format!(
                        "{kind:?}: a finite segment guide requires numeric x and y axes"
                    )));
                }
                _ => {}
            }
        }

        if !self.point_layers().is_empty() && kind != PlotKind::Line {
            return Err(Error::Data(format!(
                "{kind:?}: borrowed point layers are supported only by Line"
            )));
        }
        if !self.bar_point_layers().is_empty()
            && !matches!(kind, PlotKind::Bar | PlotKind::HorizontalBar)
        {
            return Err(Error::Data(format!(
                "{kind:?}: bar point layers are supported only by ordinary Bar and HorizontalBar"
            )));
        }
        Ok(())
    }
}

pub(crate) fn apply(
    chart: &Plot,
    result: &mut PlotRenderResult,
    backend: &mut dyn Backend,
) -> Result<(), crate::layout::fit::PlanError> {
    let projection = &result.layout.projection;
    if chart.explicit_x_ticks().is_some() {
        crate::layout::Axis::validate_ticks(
            &result.layout.x_axis.ticks,
            projection.x_range(),
            "x-axis",
        )?;
    }
    if chart.explicit_y_ticks().is_some() {
        crate::layout::Axis::validate_ticks(
            &result.layout.y_axis.ticks,
            projection.y_range(),
            "y-axis",
        )?;
    }
    if chart.guides().is_empty() {
        return Ok(());
    }

    let x_range = projection.x_range();
    let y_range = projection.y_range();
    let projected = chart
        .guides()
        .iter()
        .enumerate()
        .map(|(index, guide)| project_guide(guide, index, projection, x_range, y_range))
        .collect::<Result<Vec<_>, _>>()?;
    with_clip(backend, projection.panel_box(), |backend| {
        for (points, style) in projected {
            backend.draw_line(&points, &style)?;
        }
        Ok(())
    })
    .map_err(Into::into)
}

#[derive(Debug)]
pub(crate) struct GuideSnapshot {
    clip: crate::layout::PanelBox,
    lines: Vec<([Point2D; 2], crate::render::backend::LineStyle)>,
}

impl GuideSnapshot {
    pub(crate) fn validate_sources(
        chart: &Plot,
        ranges: [(f64, f64); 2],
        data: crate::data::PlotData<'_>,
    ) -> Result<(), Error> {
        data.claim_source_visits(chart.guides().len())?;
        for (index, guide) in chart.guides().iter().enumerate() {
            guide_endpoints(guide, index, ranges[0], ranges[1])?;
        }
        Ok(())
    }
    pub(crate) fn measure(
        chart: &Plot,
        result: &PlotRenderResult,
        data: crate::data::PlotData<'_>,
        geometry: &mut crate::render::geometry::GeometrySnapshot,
    ) -> Result<Self, Error> {
        let projection = &result.layout.projection;
        if chart.explicit_x_ticks().is_some() {
            crate::layout::Axis::validate_ticks(
                &result.layout.x_axis.ticks,
                projection.x_range(),
                "x-axis",
            )?;
        }
        if chart.explicit_y_ticks().is_some() {
            crate::layout::Axis::validate_ticks(
                &result.layout.y_axis.ticks,
                projection.y_range(),
                "y-axis",
            )?;
        }
        let clip = projection.panel_box();
        let canonical_clip = crate::render::backend::validate_clip_region(clip)?.panel();
        let lines =
            data.collect_source(chart.guides().iter().enumerate().map(|(index, guide)| {
                project_guide(
                    guide,
                    index,
                    projection,
                    projection.x_range(),
                    projection.y_range(),
                )
            }))?;
        for (points, style) in &lines {
            geometry.line(points, style, Some(canonical_clip))?;
        }
        Ok(Self { clip, lines })
    }

    pub(crate) fn draw(&self, backend: &mut dyn Backend) -> Result<(), Error> {
        if self.lines.is_empty() {
            return Ok(());
        }
        with_clip(backend, self.clip, |backend| {
            for (points, style) in &self.lines {
                backend.draw_line(points, style)?;
            }
            Ok(())
        })
    }
}

fn projected_x_ticks(
    chart: &Plot,
    ticks: &[AxisTick],
    data: crate::data::PlotData<'_>,
) -> Result<Vec<AxisTick>, Error> {
    data.collect_source(ticks
        .iter()
        .enumerate()
        .map(|(index, tick)| {
            let position = if chart.axis_scale_spec() == AxisScale::Linear {
                tick.position
            } else {
                if tick.position <= 0.0 {
                return Err(Error::Data(format!(
                    "x-axis explicit tick {index} position must be strictly positive on a logarithmic scale; got {}",
                    tick.position
                )));
                }
                tick.position.log10()
            };
            Ok(AxisTick::new(position, data.copy_source_string(&tick.label)?))
        }))
}

#[cfg(test)]
#[test]
fn projected_exact_ticks_admit_records_and_labels_before_transformation() {
    let chart = Plot::forest(
        crate::CategoryValues::new("term", ["a"]).unwrap(),
        crate::NumericValues::new("value", [1.0]),
        crate::Bounds::new(
            crate::NumericValues::new("lower", [0.1]),
            crate::NumericValues::new("upper", [10.0]),
        ),
        1.0,
    )
    .scale(AxisScale::Log10)
    .erase();
    for (visits, bytes) in [(0, 100), (100, 0)] {
        let resources =
            crate::font::canonical::OperationGenerationResources::with_test_limits(visits, bytes);
        let error = projected_x_ticks(
            &chart,
            &[AxisTick::new(0.0, "invalid")],
            chart.data().with_axis_resources(&resources),
        )
        .unwrap_err();
        assert!(matches!(error, Error::Render(_)), "{error}");
    }
    let resources = crate::font::canonical::OperationGenerationResources::new();
    let ticks = [AxisTick::new(0.1, "lower"), AxisTick::new(10.0, "upper")];
    assert_eq!(
        projected_x_ticks(&chart, &ticks, chart.data().with_axis_resources(&resources)).unwrap(),
        [AxisTick::new(-1.0, "lower"), AxisTick::new(1.0, "upper")]
    );
}

fn project_guide(
    guide: &GuideSpec,
    index: usize,
    projection: &crate::layout::Projection,
    x_range: (f64, f64),
    y_range: (f64, f64),
) -> Result<([Point2D; 2], crate::render::backend::LineStyle), Error> {
    let (start, end, style) = guide_endpoints(guide, index, x_range, y_range)?;
    let start = projection.project(start.0, start.1);
    let end = projection.project(end.0, end.1);
    Ok((
        [Point2D::new(start.0, start.1), Point2D::new(end.0, end.1)],
        style,
    ))
}

#[expect(
    clippy::type_complexity,
    reason = "validated scientific endpoints and their current line style"
)]
fn guide_endpoints(
    guide: &GuideSpec,
    index: usize,
    x_range: (f64, f64),
    y_range: (f64, f64),
) -> Result<((f64, f64), (f64, f64), crate::render::backend::LineStyle), Error> {
    let (start, end, style) = match guide {
        GuideSpec::Horizontal { y, style } => ((x_range.0, *y), (x_range.1, *y), style),
        GuideSpec::Vertical { x, style } => ((*x, y_range.0), (*x, y_range.1), style),
        GuideSpec::Segment { start, end, style } => (*start, *end, style),
    };
    validate_guide_point(start, index, "start", x_range, y_range)?;
    validate_guide_point(end, index, "end", x_range, y_range)?;
    if same_coordinate(start.0, end.0) && same_coordinate(start.1, end.1) {
        return Err(Error::Data(format!(
            "guide {index} endpoints must be distinct"
        )));
    }
    Ok((start, end, style.line_style()))
}

fn validate_guide_point(
    point: (f64, f64),
    index: usize,
    endpoint: &str,
    x_range: (f64, f64),
    y_range: (f64, f64),
) -> Result<(), Error> {
    if !point.0.is_finite() || !point.1.is_finite() {
        return Err(Error::Data(format!(
            "guide {index} {endpoint} must contain finite coordinates; got ({}, {})",
            point.0, point.1
        )));
    }
    if point.0 < x_range.0 || point.0 > x_range.1 || point.1 < y_range.0 || point.1 > y_range.1 {
        return Err(Error::Data(format!(
            "guide {index} {endpoint} ({}, {}) is outside resolved ranges {}..{} and {}..{}",
            point.0, point.1, x_range.0, x_range.1, y_range.0, y_range.1
        )));
    }
    Ok(())
}

fn same_coordinate(left: f64, right: f64) -> bool {
    left.to_bits() == right.to_bits() || (left == 0.0 && right == 0.0)
}

fn numeric_axes(chart: &Plot, kind: PlotKind) -> (bool, bool) {
    match kind {
        PlotKind::Scatter
        | PlotKind::Histogram
        | PlotKind::Density
        | PlotKind::Qq
        | PlotKind::Regression
        | PlotKind::Marginal
        | PlotKind::Line
        | PlotKind::LinePath => (true, true),
        PlotKind::Trajectory => {
            let numeric_x = chart
                .x_col()
                .and_then(|column| chart.data().column(column).ok())
                .is_some_and(crate::data::DataColumn::is_numeric);
            (numeric_x, true)
        }
        PlotKind::PointInterval
        | PlotKind::Coefficient
        | PlotKind::Forest
        | PlotKind::Dumbbell
        | PlotKind::HorizontalBar
        | PlotKind::StackedHorizontalBar
        | PlotKind::Beeswarm
        | PlotKind::Waterfall => (true, false),
        PlotKind::Interaction
        | PlotKind::Bar
        | PlotKind::StackedBar
        | PlotKind::Boxplot
        | PlotKind::Violin
        | PlotKind::Raincloud => (false, true),
        PlotKind::Heatmap | PlotKind::Matrix => (false, false),
    }
}
