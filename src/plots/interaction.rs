//! Root-prepared interaction cells and immutable selected trace operands.

use crate::data::{ObservedCollectives, PlotData};
use crate::decorations::legend::LegendEntry;
use crate::render::backend::{Backend, LineStyle, Point2D, PointStyle};
use crate::spec::{LinePattern, PlotRenderContext, PlotRenderResult, PointShape};
use crate::{Error, Plot};

#[cfg(test)]
pub(crate) mod extraction;

#[cfg(test)]
mod native;

#[cfg(test)]
thread_local! {
    pub(crate) static STAGES: std::cell::Cell<[usize; 3]> = const { std::cell::Cell::new([0; 3]) };
}

#[cfg(test)]
fn count(stage: usize) {
    STAGES.with(|counts| {
        let mut next = counts.get();
        next[stage] += 1;
        counts.set(next);
    });
}

#[derive(Debug, Clone, Copy, Default)]
struct TraceStyle {
    point_shape: PointShape,
    line_pattern: LinePattern,
}

#[derive(Debug)]
struct Cell {
    condition: usize,
    estimate: f64,
    lower: f64,
    upper: f64,
}

#[derive(Debug)]
struct Trace {
    identity: crate::data::CollectivePair,
    color_index: usize,
    style: TraceStyle,
    cells: Vec<Cell>,
}

#[derive(Debug)]
pub(crate) struct PreparedInteraction {
    pub(crate) conditions: Vec<String>,
    pub(crate) range: (f64, f64),
    traces: Vec<Trace>,
}

impl PreparedInteraction {
    pub(crate) fn hue_levels(&self) -> impl Iterator<Item = &str> {
        self.traces
            .iter()
            .filter_map(|trace| trace.identity.hue.as_deref())
    }

    pub(crate) fn has_guide_entries(&self, chart: &Plot) -> bool {
        self.traces
            .iter()
            .any(|trace| trace.identity.hue.is_some() || chart.interaction_styles_spec().is_some())
    }

    pub(crate) fn guide_entries(
        &self,
        chart: &Plot,
        theme: &crate::theme::ResolvedTheme,
        data: PlotData<'_>,
    ) -> Result<Vec<LegendEntry>, Error> {
        let mut entries = Vec::new();
        data.reserve_source(&mut entries, self.traces.len())?;
        for trace in &self.traces {
            if trace.identity.hue.is_none() && chart.interaction_styles_spec().is_none() {
                continue;
            }
            let key = trace
                .identity
                .hue
                .as_deref()
                .or(trace.identity.group.as_deref())
                .ok_or_else(|| Error::Render("interaction trace has no color identity".into()))?;
            let color = chart.categorical_color("interaction", key, trace.color_index, theme)?;
            let identity = chart.collective_legend_identity(&trace.identity, data)?;
            let point = chart
                .point_style(theme, color, 4)
                .with_shape(trace.style.point_shape);
            let line = trace.style.line_pattern.line_style(color, 1.8);
            entries.push(
                match (
                    chart.markers_visible(),
                    chart.interaction_connectors_visible(),
                ) {
                    (true, true) => LegendEntry::point_line(identity, point, line),
                    (true, false) => LegendEntry::point(identity, point),
                    (false, true) => LegendEntry::line(identity, line),
                    (false, false) => {
                        return Err(Error::Render(
                            "interaction legend has no visible glyph component".into(),
                        ));
                    }
                },
            );
        }
        Ok(entries)
    }
}

#[derive(Debug)]
pub(crate) struct PreparedInteractionInputs {
    pub(crate) panels: Vec<(Option<String>, PreparedInteraction)>,
}

#[expect(
    clippy::too_many_lines,
    reason = "one admitted semantic preparation boundary"
)]
pub(crate) fn prepare(
    chart: &Plot,
    data: PlotData<'_>,
    parent: Option<&ObservedCollectives>,
) -> Result<PreparedInteraction, Error> {
    data.claim_mark_rows()?;
    #[cfg(test)]
    count(0);
    let x_col = chart
        .x_col()
        .ok_or_else(|| Error::Data("interaction: x factor column not set".into()))?;
    let estimate_col = chart
        .y_col()
        .ok_or_else(|| Error::Data("interaction: estimate column not set".into()))?;
    chart
        .group_col()
        .ok_or_else(|| Error::Data("interaction: graphical group column not set".into()))?;
    let (lo, hi) = chart.interval_bound_cols().ok_or_else(|| {
        Error::Data("interaction: complete precomputed interval columns are required".into())
    })?;
    let x = crate::data::strict_strings(data, x_col, "interaction")?;
    let channels = crate::data::observed_collectives(
        data,
        chart.group_col(),
        chart.group_name(),
        chart.hue_col(),
        chart.hue_name(),
        None,
        "interaction",
    )?;
    let estimate = crate::data::strict_finite_f64(data, estimate_col, "interaction")?;
    let lower = crate::data::strict_finite_f64(data, lo, "interaction")?;
    let upper = crate::data::strict_finite_f64(data, hi, "interaction")?;
    if x.is_empty()
        || [
            channels.rows.len(),
            estimate.len(),
            lower.len(),
            upper.len(),
        ]
        .iter()
        .any(|length| *length != x.len())
    {
        return Err(Error::Data(
            "interaction: all channels must have equal non-zero lengths".into(),
        ));
    }
    let conditions = data.source_levels(&x)?;
    let domain = parent.unwrap_or(&channels);
    let groups = chart.global_group_levels().unwrap_or(&domain.group_levels);
    let hues = chart.global_hue_levels().unwrap_or(&domain.hue_levels);
    let identities = chart
        .global_collective_pairs()
        .unwrap_or(&domain.identities);
    let styles = resolve_trace_styles(chart, groups, data)?;
    data.claim_source_visits(x.len())?;
    for row in 0..x.len() {
        if lower[row] > estimate[row] || estimate[row] > upper[row] {
            return Err(Error::Data(format!(
                "interaction: source row {} must satisfy lower <= estimate <= upper",
                data.source_row(row)
            )));
        }
        data.claim_source_visits(row)?;
        if (0..row).any(|prior| x[prior] == x[row] && channels.rows[prior] == channels.rows[row]) {
            return Err(Error::Data(format!("interaction: duplicate (condition, group, hue) cell ({:?}, {:?}, {:?}) at source row {}", x[row], channels.rows[row].group, channels.rows[row].hue, data.source_row(row))));
        }
    }
    let mut traces = Vec::new();
    data.reserve_source(&mut traces, identities.len())?;
    for identity in identities {
        data.claim_source_visits(channels.rows.len())?;
        if !channels.rows.contains(identity) {
            continue;
        }
        let group = identity.group.as_deref().ok_or_else(|| {
            Error::Data("interaction: graphical group channel did not resolve".into())
        })?;
        data.claim_source_visits(groups.len())?;
        let index = groups
            .iter()
            .position(|level| level == group)
            .ok_or_else(|| {
                Error::Data(format!(
                    "interaction: graphical group {group:?} is absent from the resolved order"
                ))
            })?;
        let color_index = if let Some(hue) = &identity.hue {
            data.claim_source_visits(hues.len())?;
            hues.iter().position(|level| level == hue).ok_or_else(|| {
                Error::Data(format!(
                    "interaction: hue {hue:?} is absent from the resolved order"
                ))
            })?
        } else {
            0
        };
        let mut cells = Vec::new();
        data.claim_source_visits(channels.rows.len())?;
        let cell_count = channels.rows.iter().filter(|row| *row == identity).count();
        data.reserve_source(&mut cells, cell_count)?;
        for (condition, level) in conditions.iter().enumerate() {
            data.claim_source_visits(x.len())?;
            if let Some(row) =
                (0..x.len()).find(|row| &x[*row] == level && &channels.rows[*row] == identity)
            {
                cells.push(Cell {
                    condition,
                    estimate: estimate[row],
                    lower: lower[row],
                    upper: upper[row],
                });
            }
        }
        traces.push(Trace {
            identity: data.copy_source_pair(identity)?,
            color_index,
            style: styles
                .as_ref()
                .map_or_else(TraceStyle::default, |styles| styles[index]),
            cells,
        });
    }
    data.claim_source_visits(lower.len())?;
    data.claim_source_visits(upper.len())?;
    Ok(PreparedInteraction {
        conditions,
        range: crate::data::min_max(lower.into_iter().chain(upper), "interaction")?,
        traces,
    })
}

#[derive(Debug)]
pub(crate) struct InteractionPayload {
    clip: crate::layout::PanelBox,
    traces: Vec<TracePayload>,
}

#[derive(Debug)]
struct TracePayload {
    intervals: Vec<[Point2D; 2]>,
    points: Vec<Point2D>,
    interval: LineStyle,
    connector: Option<LineStyle>,
    marker: Option<PointStyle>,
}

impl InteractionPayload {
    pub(crate) fn draw(&self, backend: &mut dyn Backend) -> Result<(), Error> {
        #[cfg(test)]
        count(2);
        crate::render::backend::with_clip(backend, self.clip, |backend| {
            for trace in &self.traces {
                for interval in &trace.intervals {
                    backend.draw_line(interval, &trace.interval)?;
                }
                if let Some(style) = &trace.connector {
                    backend.draw_line(&trace.points, style)?;
                }
                if let Some(style) = &trace.marker {
                    backend.draw_scatter(&trace.points, style)?;
                }
            }
            Ok(())
        })
    }
}

#[expect(
    clippy::cast_precision_loss,
    reason = "selected bounded category projection and frozen trace styles"
)]
pub(crate) fn measure_marks(
    chart: &Plot,
    context: PlotRenderContext<'_>,
    layout: crate::layout::Layout,
) -> Result<PlotRenderResult, crate::layout::fit::PlanError> {
    #[cfg(test)]
    count(1);
    let marks = chart
        .prepared_interaction
        .as_ref()
        .and_then(|inputs| {
            inputs
                .panels
                .iter()
                .find(|(scope, _)| scope.as_deref() == chart.axis_scope())
        })
        .map(|(_, marks)| marks)
        .ok_or_else(|| {
            Error::Render("interaction measurement requires root-prepared semantics".into())
        })?;
    let resources = context
        .theme
        .prepared_font_environment()?
        .generation_resources();
    let data = context.data.with_axis_resources(&resources);
    let mut payload = InteractionPayload {
        clip: layout.projection.panel_box(),
        traces: Vec::new(),
    };
    let legends = marks.guide_entries(chart, context.theme, data)?;
    data.reserve_source(&mut payload.traces, marks.traces.len())?;
    let mut geometry = crate::render::geometry::GeometrySnapshot::new(context.theme)?;
    let clip = crate::render::backend::validate_clip_region(payload.clip)?.panel();
    for trace in &marks.traces {
        let key = trace
            .identity
            .hue
            .as_deref()
            .or(trace.identity.group.as_deref())
            .ok_or_else(|| Error::Render("interaction trace has no color identity".into()))?;
        let color =
            chart.categorical_color("interaction", key, trace.color_index, context.theme)?;
        let point = chart
            .point_style(context.theme, color, 4)
            .with_shape(trace.style.point_shape);
        let line = trace.style.line_pattern.line_style(color, 1.8);
        let mut selected = TracePayload {
            intervals: Vec::new(),
            points: Vec::new(),
            interval: LineStyle::solid(color, 1.2),
            connector: (chart.interaction_connectors_visible() && trace.cells.len() > 1)
                .then(|| line.clone()),
            marker: chart.markers_visible().then(|| point.clone()),
        };
        data.reserve_source(&mut selected.intervals, trace.cells.len())?;
        data.reserve_source(&mut selected.points, trace.cells.len())?;
        for cell in &trace.cells {
            let x = cell.condition as f64 + 0.5;
            let (px, py) = layout.projection.project(x, cell.estimate);
            let (_, low) = layout.projection.project(x, cell.lower);
            let (_, high) = layout.projection.project(x, cell.upper);
            layout.projection.validate_span(
                crate::layout::AxisSide::Left,
                (cell.lower, cell.upper),
                (low, high),
                cell.lower < cell.upper,
                "interaction interval",
            )?;
            let interval = [Point2D::new(px, low), Point2D::new(px, high)];
            geometry.line(&interval, &selected.interval, Some(clip))?;
            selected.intervals.push(interval);
            selected.points.push(Point2D::new(px, py));
        }
        if let Some(style) = &selected.connector {
            geometry.line(&selected.points, style, Some(clip))?;
        }
        if let Some(style) = &selected.marker {
            geometry.scatter(&selected.points, style, clip)?;
        }
        payload.traces.push(selected);
    }
    let mut result = PlotRenderResult::new(layout, legends);
    result.x_domain = crate::spec::AxisDomain::Categorical;
    result.marks = Some(data.source_arc(crate::layout::interior::DeferredMarkPlan {
        payload: crate::layout::interior::MarkPayload::Interaction(payload),
        geometry,
    })?);
    Ok(result)
}

fn resolve_trace_styles(
    chart: &Plot,
    levels: &[String],
    data: PlotData<'_>,
) -> Result<Option<Vec<TraceStyle>>, Error> {
    let Some(configured) = chart.interaction_styles_spec() else {
        return Ok(None);
    };
    let mut styles = data.collect_source(levels.iter().map(|_| Ok(None)))?;
    for configured in configured {
        let index = data
            .source_position(levels, |level| level == configured.group())?
            .ok_or_else(|| {
                Error::Data(format!(
                    "interaction: style references unknown graphical group {:?}",
                    configured.group()
                ))
            })?;
        if styles[index].is_some() {
            return Err(Error::Data(format!(
                "interaction: graphical group {:?} has more than one configured style",
                configured.group()
            )));
        }
        styles[index] = Some(TraceStyle {
            point_shape: configured.point_shape(),
            line_pattern: configured.line_pattern(),
        });
    }
    data.collect_source(styles.into_iter().zip(levels).map(|(style, trace)| {
        data.claim_source_visits(1)?;
        style.ok_or_else(|| {
            Error::Data(format!(
                "interaction: graphical group {trace:?} has no configured style"
            ))
        })
    }))
    .map(Some)
}
