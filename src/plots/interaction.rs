//! Root-prepared interaction cells and immutable selected trace operands.

use crate::alpha::{
    marks::{point_ink, push_point, ObservationAlpha},
    AlphaTarget, AlphaUnitKind,
};
use crate::data::{ObservedCollectives, PlotData};
use crate::decorations::legend::source::{LineRole, PointRole};
use crate::decorations::legend::GuideSourceKey;
use crate::render::backend::{Backend, LineStyle, Point2D, PointStyle};
use crate::spec::{LinePattern, PlotRenderContext, PlotRenderResult, PointShape};
use crate::theme::CategoryColors;
use crate::{Error, Plot};

#[cfg(test)]
pub(crate) mod extraction;

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
    alpha: ObservationAlpha,
}

#[derive(Debug)]
struct Trace {
    identity: crate::data::CollectivePair,
    style: TraceStyle,
    cells: Vec<Cell>,
}

#[derive(Debug)]
pub(crate) struct PreparedInteraction {
    pub(crate) conditions: Vec<String>,
    pub(crate) range: (f64, f64),
    traces: Vec<Trace>,
}

pub(crate) fn intrinsic_body_minimum(
    chart: &Plot,
    theme: &crate::theme::ResolvedTheme,
) -> Result<(f64, f64), Error> {
    let prepared = chart
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
            Error::Render("Interaction intrinsic requires root-prepared traces".into())
        })?;
    if prepared.conditions.is_empty() || prepared.traces.is_empty() {
        return Err(Error::Render(
            "Interaction has no categorical slots or traces".into(),
        ));
    }

    let mut pitch = 1.0_f64 / 1024.0;
    let mut numeric = 0.0_f64;
    let interval = crate::render::geometry::line_ink_width(
        &theme
            .marks
            .data_lines()
            .interaction_interval()
            .backend_style((0, 0, 0)),
    )?;
    for trace in &prepared.traces {
        let marker = if chart.markers_visible() {
            let point = chart
                .point_style(theme, (0, 0, 0), theme.marks.points().radii().interaction())
                .with_shape(trace.style.point_shape);
            point_ink(
                &point,
                trace
                    .cells
                    .iter()
                    .map(|cell| (0.0, cell.estimate, cell.alpha)),
            )?
            .map(|ink| ink.bounds)
        } else {
            None
        };
        let connector = if chart.interaction_connectors_visible() && trace.cells.len() > 1 {
            crate::render::geometry::line_ink_width(
                &theme
                    .marks
                    .data_lines()
                    .interaction_connector()
                    .backend_style((0, 0, 0)),
            )?
        } else {
            0.0
        };
        let stroke = interval.max(connector);
        numeric = numeric.max(stroke);
        pitch = pitch.max(stroke);
        for cell in &trace.cells {
            if cell.condition >= prepared.conditions.len() {
                return Err(Error::Render(
                    "Interaction occupied condition exceeds prepared slots".into(),
                ));
            }
            if let Some(ink) = marker {
                pitch = pitch.max(2.0 * (-ink.left).max(ink.right));
                numeric = numeric.max(ink.top - ink.bottom);
            }
        }
    }
    let count = u32::try_from(prepared.conditions.len())
        .map_err(|_| Error::Render("Interaction condition count exceeds spatial range".into()))?;
    let categorical = pitch * f64::from(count);
    if !categorical.is_finite() {
        return Err(Error::Render(
            "Interaction categorical hard span overflowed".into(),
        ));
    }
    Ok((
        categorical,
        if chart.user_y_limits().is_none() {
            numeric
        } else {
            0.0
        },
    ))
}

impl PreparedInteraction {
    pub(crate) fn color_keys(&self) -> impl Iterator<Item = &str> {
        let has_hue = self.traces.iter().any(|trace| trace.identity.hue.is_some());
        self.traces.iter().filter_map(move |trace| {
            if has_hue {
                trace.identity.hue.as_deref()
            } else {
                trace.identity.group.as_deref()
            }
        })
    }

    fn resolved_colors(
        &self,
        chart: &Plot,
        theme: &crate::theme::ResolvedTheme,
        data: PlotData<'_>,
    ) -> Result<CategoryColors, Error> {
        let has_hue = self.traces.iter().any(|trace| trace.identity.hue.is_some());
        let parent = if has_hue {
            chart.global_hue_levels()
        } else {
            chart.global_group_levels()
        };
        let keys = if let Some(levels) = parent {
            crate::data::source::collect_source(
                levels
                    .iter()
                    .map(|level| crate::data::source::copy_source_string(level)),
            )?
        } else {
            let mut observed = Vec::new();
            crate::data::source::reserve_source(&mut observed, self.traces.len())?;

            for raw in self.color_keys() {
                observed.push(crate::data::source::copy_source_string(raw)?);
            }
            crate::data::source::source_levels(&observed)?
        };
        CategoryColors::resolve(
            &keys,
            theme.color_scales.qualitative(),
            chart.categorical_color_scale_spec(),
            data.source_resources()?,
        )
    }

    pub(crate) fn checked_primary_mark_count(&self) -> Result<u128, Error> {
        self.traces.iter().try_fold(0_u128, |count, trace| {
            count
                .checked_add(trace.cells.len() as u128)
                .ok_or_else(|| Error::Render("Interaction occupied mark count overflowed".into()))
        })
    }

    pub(crate) fn guide_entries(
        &self,
        chart: &Plot,
        theme: &crate::theme::ResolvedTheme,
        data: PlotData<'_>,
    ) -> Result<Vec<GuideSourceKey>, Error> {
        let colors = self.resolved_colors(chart, theme, data)?;
        self.guide_entries_with_colors(chart, theme, &colors)
    }

    fn guide_entries_with_colors(
        &self,
        chart: &Plot,
        theme: &crate::theme::ResolvedTheme,

        colors: &CategoryColors,
    ) -> Result<Vec<GuideSourceKey>, Error> {
        let mut entries = Vec::new();
        crate::data::source::reserve_source(&mut entries, self.traces.len())?;
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
            let color = colors.color(key)?;
            let identity = chart.collective_legend_identity(&trace.identity)?;
            let point = chart
                .point_style(theme, color, theme.marks.points().radii().interaction())
                .with_shape(trace.style.point_shape);
            let mut line = theme
                .marks
                .data_lines()
                .interaction_connector()
                .backend_style(color);
            line.pattern = trace.style.line_pattern.stroke_pattern();
            let mapped_styles = chart.interaction_styles_spec().is_some();
            let mut key = GuideSourceKey::collective(identity, chart, mapped_styles)?;
            if chart.markers_visible() {
                key = key.with_point(point, PointRole::Primary);
            }
            if chart.interaction_connectors_visible() {
                key = key.with_line(line, LineRole::InteractionConnector);
            }
            if mapped_styles {
                key = key.with_interaction_group_style()?;
            }
            entries.push(key);
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
    data.validate_row_source()?;
    let alpha = chart.prepare_alpha(AlphaTarget::Point, AlphaUnitKind::Observation, &[])?;
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
    let conditions = crate::data::source::source_levels(&x)?;
    let domain = parent.unwrap_or(&channels);
    let groups = chart.global_group_levels().unwrap_or(&domain.group_levels);
    let hues = chart.global_hue_levels().unwrap_or(&domain.hue_levels);
    let identities = chart
        .global_collective_pairs()
        .unwrap_or(&domain.identities);
    let styles = resolve_trace_styles(chart, groups)?;

    for row in 0..x.len() {
        if lower[row] > estimate[row] || estimate[row] > upper[row] {
            return Err(Error::Data(format!(
                "interaction: source row {} must satisfy lower <= estimate <= upper",
                data.source_row(row)
            )));
        }

        if (0..row).any(|prior| x[prior] == x[row] && channels.rows[prior] == channels.rows[row]) {
            return Err(Error::Data(format!("interaction: duplicate (condition, group, hue) cell ({:?}, {:?}, {:?}) at source row {}", x[row], channels.rows[row].group, channels.rows[row].hue, data.source_row(row))));
        }
    }
    let mut traces = Vec::new();
    crate::data::source::reserve_source(&mut traces, identities.len())?;
    for identity in identities {
        if !channels.rows.contains(identity) {
            continue;
        }
        let group = identity.group.as_deref().ok_or_else(|| {
            Error::Data("interaction: graphical group channel did not resolve".into())
        })?;

        let index = groups
            .iter()
            .position(|level| level == group)
            .ok_or_else(|| {
                Error::Data(format!(
                    "interaction: graphical group {group:?} is absent from the resolved order"
                ))
            })?;
        if let Some(hue) = &identity.hue {
            if !hues.contains(hue) {
                return Err(Error::Data(format!(
                    "interaction: hue {hue:?} is absent from the resolved order"
                )));
            }
        }
        let mut cells = Vec::new();

        let cell_count = channels.rows.iter().filter(|row| *row == identity).count();
        crate::data::source::reserve_source(&mut cells, cell_count)?;
        for (condition, level) in conditions.iter().enumerate() {
            if let Some(row) =
                (0..x.len()).find(|row| &x[*row] == level && &channels.rows[*row] == identity)
            {
                cells.push(Cell {
                    condition,
                    estimate: estimate[row],
                    lower: lower[row],
                    upper: upper[row],
                    alpha: ObservationAlpha::resolve(data, &alpha, row)?,
                });
            }
        }
        traces.push(Trace {
            identity: crate::data::source::copy_source_pair(identity)?,
            style: styles
                .as_ref()
                .map_or_else(TraceStyle::default, |styles| styles[index]),
            cells,
        });
    }
    if let Some(exact) = chart.categorical_map_spec() {
        for trace in &traces {
            let key = trace
                .identity
                .hue
                .as_deref()
                .or(trace.identity.group.as_deref())
                .ok_or_else(|| Error::Render("Interaction color identity is missing".into()))?;
            if exact.color_in(key).is_none() {
                return Err(Error::Data(format!(
                    "interaction: category {key:?} has no caller-owned color"
                )));
            }
        }
    }

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
    pub(crate) domain_expansions: [Option<crate::layout::interior::DomainExpansion>; 2],
}

#[derive(Debug)]
struct TracePayload {
    intervals: Vec<[Point2D; 2]>,
    points: Vec<Point2D>,
    interval: LineStyle,
    connector: Option<LineStyle>,
    markers: Vec<(Vec<Point2D>, PointStyle)>,
}

impl InteractionPayload {
    fn measure_ink(
        &mut self,
        chart: &Plot,
        marks: &PreparedInteraction,
        projection: &crate::layout::Projection,
        context: PlotRenderContext<'_>,
    ) -> Result<(), crate::layout::fit::PlanError> {
        let mut full = crate::render::geometry::GeometrySnapshot::new();

        for trace in &self.traces {
            for interval in &trace.intervals {
                full.line(interval, &trace.interval, None)?;
            }
            if let Some(style) = &trace.connector {
                full.line(&trace.points, style, None)?;
            }
            for (points, style) in &trace.markers {
                full.scatter_with_clip(points, style, None)?;
            }
        }
        if let Some(bounds) = full
            .bounds()
            .iter()
            .copied()
            .reduce(crate::render::geometry::GeometryBox::union)
        {
            if bounds.left < self.clip.x0 || bounds.right > self.clip.x1 {
                return Err(context
                    .fit_scope()
                    .failure(
                        crate::layout::fit::FitKind::SpecializedGeometry,
                        "Interaction painted ink exceeds its assigned condition panel",
                    )
                    .into());
            }
            if chart.user_y_limits().is_none() {
                let count = u32::try_from(marks.conditions.len()).map_err(|_| {
                    Error::Render("Interaction condition count exceeds spatial range".into())
                })?;
                self.domain_expansions =
                    crate::layout::interior::automatic_absolute_ink_expansions(
                        [(0.0, f64::from(count)), marks.range],
                        projection,
                        bounds,
                        [false, true],
                        context.fit_scope(),
                    )?;
            }
        }
        Ok(())
    }

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
                for (points, style) in &trace.markers {
                    backend.draw_scatter(points, style)?;
                }
            }
            Ok(())
        })
    }
}

#[expect(
    clippy::cast_precision_loss,
    clippy::too_many_lines,
    reason = "selected bounded categories and ordered interval, connector and point paints share one frozen trace"
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
    let resources = context.theme.prepared_font_environment()?.operation();
    let data = context.data.with_operation(&resources);
    let mut payload = InteractionPayload {
        clip: layout.projection.panel_box(),
        traces: Vec::new(),
        domain_expansions: [None; 2],
    };
    let colors = marks.resolved_colors(chart, context.theme, data)?;
    crate::data::source::reserve_source(&mut payload.traces, marks.traces.len())?;
    let mut geometry = crate::render::geometry::GeometrySnapshot::new();
    let clip = crate::render::backend::validate_clip_region(payload.clip)?.panel();
    for trace in &marks.traces {
        let key = trace
            .identity
            .hue
            .as_deref()
            .or(trace.identity.group.as_deref())
            .ok_or_else(|| Error::Render("interaction trace has no color identity".into()))?;
        let color = colors.color(key)?;
        let point = chart
            .point_style(
                context.theme,
                color,
                context.theme.marks.points().radii().interaction(),
            )
            .with_shape(trace.style.point_shape);
        let mut line = context
            .theme
            .marks
            .data_lines()
            .interaction_connector()
            .backend_style(color);
        line.pattern = trace.style.line_pattern.stroke_pattern();
        let mut selected = TracePayload {
            intervals: Vec::new(),
            points: Vec::new(),
            interval: context
                .theme
                .marks
                .data_lines()
                .interaction_interval()
                .backend_style(color),
            connector: (chart.interaction_connectors_visible() && trace.cells.len() > 1)
                .then(|| line.clone()),
            markers: Vec::new(),
        };
        crate::data::source::reserve_source(&mut selected.intervals, trace.cells.len())?;
        crate::data::source::reserve_source(&mut selected.points, trace.cells.len())?;
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
            if chart.markers_visible() {
                push_point(
                    &mut selected.markers,
                    Point2D::new(px, py),
                    cell.alpha.point(point.clone()),
                )?;
            }
        }
        if let Some(style) = &selected.connector {
            geometry.line(&selected.points, style, Some(clip))?;
        }
        for (points, style) in &selected.markers {
            geometry.scatter(points, style, clip)?;
        }
        payload.traces.push(selected);
    }
    payload.measure_ink(chart, marks, &layout.projection, context)?;
    let mut result = PlotRenderResult::new(layout, chart.prepared_discrete_guide()?);
    result.x_domain = crate::spec::AxisDomain::Categorical;
    result.marks = Some(crate::data::source::source_arc(
        crate::layout::interior::SelectedMarkPlan {
            payload: crate::layout::interior::MarkPayload::Interaction(payload),
            geometry,
        },
    )?);
    Ok(result)
}

fn resolve_trace_styles(chart: &Plot, levels: &[String]) -> Result<Option<Vec<TraceStyle>>, Error> {
    let Some(configured) = chart.interaction_styles_spec() else {
        return Ok(None);
    };
    let mut styles = crate::data::source::collect_source(levels.iter().map(|_| Ok(None)))?;
    for configured in configured {
        let index =
            crate::data::source::source_position(levels, |level| level == configured.group())
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
    crate::data::source::collect_source(styles.into_iter().zip(levels).map(|(style, trace)| {
        style.ok_or_else(|| {
            Error::Data(format!(
                "interaction: graphical group {trace:?} has no configured style"
            ))
        })
    }))
    .map(Some)
}
