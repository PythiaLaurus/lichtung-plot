//! Validation and rendering for supplemental numeric geometry.

use crate::Error;

use crate::layout::{AxisSide, AxisTick};
use crate::render::backend::{with_clip, Backend, Point2D};
use crate::spec::{AxisScale, PlotKind, PlotRenderResult};
use crate::{GuideSpec, Plot};

const MAX_EXPLICIT_TICKS: usize = 10_000;

impl Plot {
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
) -> Result<(), Error> {
    let projection = &result.layout.projection;
    if let Some(ticks) = chart.explicit_x_ticks() {
        let ticks = projected_x_ticks(chart, ticks)?;
        validate_ticks(&ticks, projection.x_range(), "x-axis")?;
        result.layout.x_axis.ticks = ticks;
        result.layout.x_axis.validate_explicit_tick_collisions(
            projection,
            AxisSide::Bottom,
            &chart.palette(),
            "x-axis",
        )?;
    }
    if let Some(ticks) = chart.explicit_y_ticks() {
        validate_ticks(ticks, projection.y_range(), "y-axis")?;
        result.layout.y_axis.ticks = ticks.to_vec();
        result.layout.y_axis.validate_explicit_tick_collisions(
            projection,
            AxisSide::Left,
            &chart.palette(),
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
}

fn projected_x_ticks(chart: &Plot, ticks: &[AxisTick]) -> Result<Vec<AxisTick>, Error> {
    if chart.axis_scale_spec() == AxisScale::Linear {
        return Ok(ticks.to_vec());
    }
    ticks
        .iter()
        .enumerate()
        .map(|(index, tick)| {
            if tick.position <= 0.0 {
                return Err(Error::Data(format!(
                    "x-axis explicit tick {index} position must be strictly positive on a logarithmic scale; got {}",
                    tick.position
                )));
            }
            Ok(AxisTick::new(tick.position.log10(), tick.label.clone()))
        })
        .collect()
}

fn validate_ticks(ticks: &[AxisTick], range: (f64, f64), axis: &str) -> Result<(), Error> {
    if ticks.is_empty() {
        return Err(Error::Data(format!(
            "{axis} explicit numeric ticks must not be empty"
        )));
    }
    if ticks.len() > MAX_EXPLICIT_TICKS {
        return Err(Error::Data(format!(
            "{axis} explicit numeric tick count {} exceeds {MAX_EXPLICIT_TICKS}",
            ticks.len()
        )));
    }
    for (index, tick) in ticks.iter().enumerate() {
        if !tick.position.is_finite() {
            return Err(Error::Data(format!(
                "{axis} explicit tick {index} position must be finite; got {}",
                tick.position
            )));
        }
        if tick.position < range.0 || tick.position > range.1 {
            return Err(Error::Data(format!(
                "{axis} explicit tick {index} position {} is outside resolved range {}..{}",
                tick.position, range.0, range.1
            )));
        }
        if index > 0 && ticks[index - 1].position >= tick.position {
            return Err(Error::Data(format!(
                "{axis} explicit tick positions must be strictly ascending"
            )));
        }
    }
    Ok(())
}

fn project_guide(
    guide: &GuideSpec,
    index: usize,
    projection: &crate::layout::Projection,
    x_range: (f64, f64),
    y_range: (f64, f64),
) -> Result<([Point2D; 2], crate::render::backend::LineStyle), Error> {
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
    let start = projection.project(start.0, start.1);
    let end = projection.project(end.0, end.1);
    Ok((
        [Point2D::new(start.0, start.1), Point2D::new(end.0, end.1)],
        style.line_style(),
    ))
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
