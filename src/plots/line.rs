//! Root-prepared ordered series, trajectories, and source-ordered paths.

use crate::data::{CollectivePair, PlotData};
use crate::decorations::legend::LegendEntry;
use crate::render::backend::{
    with_clip, Backend, FillStyle, LineStyle, PlacedText, Point2D, PointStyle, TextAnchor,
};
use crate::spec::{AxisDomain, LinePathStyle, PlotRenderContext, PlotRenderResult};
use crate::{Error, Plot, PlotKind};

#[cfg(test)]
thread_local! {
    pub(crate) static STAGES: std::cell::Cell<[[usize; 4]; 3]> = const { std::cell::Cell::new([[0; 4]; 3]) };
}

#[cfg(test)]
fn stage(kind: PlotKind, index: usize) {
    STAGES.with(|stages| {
        let mut counts = stages.get();
        let family = match kind {
            PlotKind::Line => 0,
            PlotKind::Trajectory => 1,
            PlotKind::LinePath => 2,
            _ => unreachable!(),
        };
        counts[family][index] += 1;
        stages.set(counts);
    });
}

#[derive(Debug)]
pub(crate) struct PreparedLineInputs {
    pub(crate) panels: Vec<(Option<String>, PreparedLine)>,
}

#[derive(Debug)]
pub(crate) struct PreparedLine {
    pub(crate) ranges: [(f64, f64); 2],
    pub(crate) conditions: Option<Vec<String>>,
    groups: Vec<LineGroup>,
    hues: Vec<String>,
    layers: Vec<PointLayer>,
    primary_color_slots: usize,
}

#[derive(Debug)]
struct LineGroup {
    identity: Option<CollectivePair>,
    hue_index: Option<usize>,
    points: Vec<LinePoint>,
}

#[derive(Debug)]
struct LinePoint {
    x: f64,
    y: f64,
    interval: Option<(f64, f64)>,
    hue_index: Option<usize>,
}

#[derive(Debug)]
struct PointLayer {
    points: Vec<(f64, f64)>,
    labels: Option<Vec<String>>,
    legend: Option<String>,
}

#[derive(Debug)]
pub(crate) struct LinePayload {
    #[cfg(test)]
    kind: PlotKind,
    clip: crate::layout::PanelBox,
    marks: Vec<LineMark>,
    layers: Vec<(Vec<Point2D>, PointStyle, Vec<PlacedText>)>,
}

#[derive(Debug)]
enum LineMark {
    Ribbon(Vec<Point2D>, FillStyle),
    Line(Vec<Point2D>, LineStyle),
    Points(Vec<Point2D>, PointStyle),
}

impl PreparedLine {
    pub(crate) fn hue_levels(&self) -> impl Iterator<Item = &str> {
        self.hues.iter().map(String::as_str)
    }

    pub(crate) fn has_guide_entries(&self, chart: &Plot) -> bool {
        self.groups
            .iter()
            .any(|group| group.identity.as_ref().is_some_and(|id| id.hue.is_some()))
            || (chart.kind() != PlotKind::Line
                && (chart.kind() == PlotKind::Trajectory || chart.markers_visible())
                && chart.line_path_style_spec() != LinePathStyle::Hue
                && !self.hues.is_empty())
            || self.layers.iter().any(|layer| layer.legend.is_some())
    }

    pub(crate) fn group_order(
        &self,
        chart: &Plot,
        data: PlotData<'_>,
    ) -> Result<Vec<usize>, Error> {
        let hue_paths = chart.kind() == PlotKind::LinePath
            && chart.line_path_style_spec() == LinePathStyle::Hue;
        let mut order = data.collect_source((0..self.groups.len()).map(Ok))?;
        crate::data::source::sort_source_by(&mut order, Some(data), |a, b| {
            let key = |index: usize| {
                let group = &self.groups[index];
                group
                    .identity
                    .as_ref()
                    .and_then(|identity| {
                        if chart.kind() == PlotKind::Line || hue_paths {
                            chart
                                .global_collective_pairs()
                                .and_then(|pairs| pairs.iter().position(|pair| pair == identity))
                        } else {
                            chart.global_group_levels().and_then(|levels| {
                                levels
                                    .iter()
                                    .position(|level| Some(level) == identity.group.as_ref())
                            })
                        }
                    })
                    .unwrap_or(index)
            };
            (key(*a), *a).cmp(&(key(*b), *b))
        })?;
        Ok(order)
    }

    /// Exact legend glyphs from prepared semantic groups; no assigned panel,
    /// source reprojection, or font environment is needed.
    pub(crate) fn guide_entries_in_order(
        &self,
        chart: &Plot,
        theme: &crate::theme::ResolvedTheme,
        data: PlotData<'_>,
        order: &[usize],
    ) -> Result<Vec<LegendEntry>, Error> {
        let mut entries = Vec::new();
        let hue_paths = chart.kind() == PlotKind::LinePath
            && chart.line_path_style_spec() == LinePathStyle::Hue;
        for &index in order {
            let group = &self.groups[index];
            let color = if chart.kind() == PlotKind::Line {
                let slot = group
                    .identity
                    .as_ref()
                    .and_then(|identity| identity.hue.as_ref())
                    .map_or(0, |hue| {
                        chart.hue_color_index(hue, group.hue_index.unwrap_or(0))
                    });
                theme.colors[slot % theme.colors.len()]
            } else if hue_paths {
                let hue = group.hue_index.ok_or_else(|| {
                    Error::Render("line path: hue-styled path has no resolved hue index".into())
                })?;
                chart.categorical_color(
                    "line path",
                    &self.hues[hue],
                    chart.hue_color_index(&self.hues[hue], hue),
                    theme,
                )?
            } else {
                theme.foreground
            };
            let Some(identity) = group
                .identity
                .as_ref()
                .filter(|identity| identity.hue.is_some())
            else {
                continue;
            };
            data.claim_source_visits(group.points.len())?;
            let has_fill = chart.kind() == PlotKind::Line
                && group.points.iter().all(|point| point.interval.is_some());
            let line = LineStyle::solid(
                color,
                if chart.kind() == PlotKind::Line {
                    1.8
                } else {
                    1.4
                },
            );
            let point =
                chart.point_style(color, if chart.kind() == PlotKind::Line { 3 } else { 4 });
            let fill = FillStyle {
                color,
                stroke: color,
                stroke_width: 0.0,
                opacity: 0.18,
            };
            data.reserve_source(&mut entries, 1)?;
            let identity = chart.collective_legend_identity(identity, data)?;
            entries.push(match (chart.markers_visible(), has_fill) {
                (true, true) => LegendEntry::point_line_fill(identity, point, line, fill),
                (true, false) => LegendEntry::point_line(identity, point, line),
                (false, true) => LegendEntry::line_fill(identity, line, fill),
                (false, false) => LegendEntry::line(identity, line),
            });
        }
        if chart.kind() != PlotKind::Line
            && (chart.kind() == PlotKind::Trajectory || chart.markers_visible())
            && !hue_paths
        {
            let levels = chart.global_hue_levels().unwrap_or(&self.hues);
            data.reserve_source(&mut entries, levels.len())?;
            for (index, label) in levels.iter().enumerate() {
                entries.push(LegendEntry::point(
                    chart.point_legend_identity(label, data)?,
                    chart.point_style(theme.colors[index % theme.colors.len()], 4),
                ));
            }
        }
        let slots = if chart.hue_col().is_some() {
            chart
                .global_hue_levels()
                .map_or(self.primary_color_slots, <[String]>::len)
        } else {
            self.primary_color_slots
        };
        for (index, layer) in self.layers.iter().enumerate() {
            if let Some(label) = &layer.legend {
                data.reserve_source(&mut entries, 1)?;
                let style = chart.point_layers()[index]
                    .point_style(theme.colors[(slots + index) % theme.colors.len()]);
                entries.push(
                    LegendEntry::point(chart.point_legend_identity(label, data)?, style)
                        .without_hue_level(),
                );
            }
        }
        Ok(entries)
    }

    #[expect(
        clippy::too_many_lines,
        clippy::float_cmp,
        reason = "one normalization preserves family order; duplicate scientific coordinates compare exactly including signed zero"
    )]
    pub(crate) fn new(
        chart: &Plot,
        data: PlotData<'_>,
        conditions: Option<&[String]>,
    ) -> Result<Self, Error> {
        data.claim_mark_rows()?;
        #[cfg(test)]
        stage(chart.kind(), 0);
        let kind = chart.kind();
        let family = match kind {
            PlotKind::Line => "line",
            PlotKind::Trajectory => "trajectory",
            PlotKind::LinePath => "line path",
            _ => {
                return Err(Error::Data(
                    "line preparation requires a line family".into(),
                ))
            }
        };
        let x_col = chart
            .x_col()
            .ok_or_else(|| Error::Data(format!("{family}: x column not set")))?;
        let y_col = chart
            .y_col()
            .ok_or_else(|| Error::Data(format!("{family}: y column not set")))?;
        if kind != PlotKind::Line && chart.group_col().is_none() {
            return Err(Error::Data(format!(
                "{family}: explicit graphical group column not set"
            )));
        }
        let channels = crate::data::observed_collectives(
            data,
            chart.group_col(),
            chart.group_name(),
            chart.hue_col(),
            chart.hue_name(),
            None,
            family,
        )?;
        let hues = data.collect_source(
            chart
                .global_hue_levels()
                .unwrap_or(&channels.hue_levels)
                .iter()
                .map(|label| data.copy_source_string(label)),
        )?;
        let y = crate::data::strict_finite_f64(data, y_col, family)?;
        let (x, conditions) = if data.column(x_col)?.is_numeric() {
            (crate::data::strict_finite_f64(data, x_col, family)?, None)
        } else if kind == PlotKind::Trajectory {
            let labels = crate::data::strict_normalized_strings(data, x_col, family, "condition")?;
            let levels =
                if let Some(levels) = conditions.or_else(|| chart.global_line_condition_levels()) {
                    data.collect_source(levels.iter().map(|label| data.copy_source_string(label)))?
                } else {
                    data.source_levels(&labels)?
                };
            let positions = data.collect_source(labels.iter().map(|label| {
                let index = data
                    .source_position(&levels, |level| level == label)?
                    .ok_or_else(|| {
                        Error::Data(format!(
                            "trajectory: condition {label:?} is absent from global order"
                        ))
                    })?;
                Ok(categorical_index(index)? + 0.5)
            }))?;
            (positions, Some(levels))
        } else {
            return Err(Error::Data(format!("{family}: x must be numeric")));
        };
        if x.is_empty() || x.len() != y.len() || channels.rows.len() != x.len() {
            return Err(Error::Data(format!(
                "{family}: aligned columns must have equal non-zero lengths"
            )));
        }
        let bounds = if kind == PlotKind::Line {
            chart
                .interval_bound_cols()
                .map(|(lower, upper)| {
                    Ok::<_, Error>((
                        crate::data::strict_finite_f64(data, lower, family)?,
                        crate::data::strict_finite_f64(data, upper, family)?,
                    ))
                })
                .transpose()?
        } else {
            None
        };
        let hue_paths =
            kind == PlotKind::LinePath && chart.line_path_style_spec() == LinePathStyle::Hue;
        if hue_paths && chart.hue_col().is_none() {
            return Err(Error::Data(
                "line path: LinePathStyle::Hue requires a categorical hue column".into(),
            ));
        }
        let identities = chart
            .global_collective_pairs()
            .unwrap_or(&channels.identities);
        let group_levels = chart
            .global_group_levels()
            .unwrap_or(&channels.group_levels);
        let collective = kind == PlotKind::Line || hue_paths;
        if kind == PlotKind::Trajectory {
            let mut rows = data.collect_source((0..x.len()).map(Ok))?;
            crate::data::source::sort_source_by(&mut rows, Some(data), |a, b| {
                let normalized = |row: usize| if x[row] == 0.0 { 0.0 } else { x[row] };
                channels.rows[*a]
                    .group
                    .cmp(&channels.rows[*b].group)
                    .then_with(|| normalized(*a).total_cmp(&normalized(*b)))
                    .then_with(|| a.cmp(b))
            })?;
            data.claim_source_visits(rows.len())?;
            if let Some(row) = rows
                .windows(2)
                .filter(|pair| {
                    channels.rows[pair[0]].group == channels.rows[pair[1]].group
                        && x[pair[0]] == x[pair[1]]
                })
                .map(|pair| pair[1])
                .min()
            {
                return Err(Error::Data(format!(
                    "trajectory: duplicate normalized (group, condition) at source row {}",
                    data.source_row(row)
                )));
            }
        }
        let count = if collective {
            identities.len()
        } else {
            group_levels.len()
        };
        let mut groups = Vec::new();
        data.reserve_source(&mut groups, count)?;
        for index in 0..count {
            let mut rows = Vec::new();
            let belongs = |row: usize| {
                if collective {
                    channels.rows[row] == identities[index]
                } else {
                    channels.rows[row].group.as_ref() == Some(&group_levels[index])
                }
            };
            data.claim_source_visits(x.len())?;
            let count = (0..x.len()).filter(|row| belongs(*row)).count();
            data.reserve_source(&mut rows, count)?;
            for row in 0..x.len() {
                data.claim_source_visits(1)?;
                if belongs(row) {
                    rows.push(row);
                }
            }
            if rows.is_empty() {
                continue;
            }
            if kind != PlotKind::Line && rows.len() < 2 {
                return Err(Error::Data(if hue_paths {
                    format!(
                        "line path: {} requires at least two retained points",
                        identities[index].description(channels.collapsed)
                    )
                } else {
                    format!("trajectory: graphical group {:?} requires at least two retained conditions", group_levels[index])
                }));
            }
            let identity = Some(if collective {
                data.copy_source_pair(&identities[index])?
            } else {
                CollectivePair {
                    group: Some(data.copy_source_string(&group_levels[index])?),
                    hue: None,
                }
            });
            let hue_index = identity
                .as_ref()
                .and_then(|identity| identity.hue.as_ref())
                .map(|hue| data.source_position(&hues, |level| level == hue))
                .transpose()?
                .flatten();
            if hue_paths && hue_index.is_none() {
                return Err(Error::Data(
                    "line path: hue is absent from the resolved hue order".into(),
                ));
            }
            let mut points = data.collect_source(rows.iter().map(|row| {
                let row = *row;
                let interval = if let Some((lower, upper)) = &bounds {
                    data.claim_source_visits(1)?;
                    let (lower, upper) = (lower[row], upper[row]);
                    if lower > y[row] || y[row] > upper {
                        return Err(Error::Data(format!(
                            "line: source row {} must satisfy lower <= y <= upper",
                            data.source_row(row)
                        )));
                    }
                    Some((lower, upper))
                } else {
                    None
                };
                Ok(LinePoint {
                    x: x[row],
                    y: y[row],
                    interval,
                    hue_index: channels.rows[row]
                        .hue
                        .as_ref()
                        .map(|hue| data.source_position(&hues, |level| level == hue))
                        .transpose()?
                        .flatten(),
                })
            }))?;
            if kind != PlotKind::LinePath {
                crate::data::source::sort_source_by(&mut points, Some(data), |a, b| {
                    a.x.total_cmp(&b.x)
                })?;
                data.claim_source_visits(points.len())?;
                if kind == PlotKind::Line && points.windows(2).any(|pair| pair[0].x == pair[1].x) {
                    return Err(Error::Data(format!(
                        "line: duplicate x value within series {}",
                        identities[index].description(channels.collapsed)
                    )));
                }
            }
            groups.push(LineGroup {
                identity,
                hue_index,
                points,
            });
        }
        let mut layers = Vec::new();
        data.reserve_source(&mut layers, chart.point_layers().len())?;
        for (index, layer) in chart.point_layers().iter().enumerate() {
            let mut buffer = [0_u8; 64];
            let mut cursor = std::io::Cursor::new(buffer.as_mut_slice());
            std::io::Write::write_fmt(&mut cursor, format_args!("line point layer {index}"))
                .expect("bounded layer index");
            let length = usize::try_from(cursor.position()).expect("bounded layer index");
            let context = std::str::from_utf8(&buffer[..length]).expect("ASCII layer index");
            let x = layer.x_values();
            let y = layer.y_values();
            if x.is_empty() || x.len() != y.len() {
                return Err(Error::Data(
                    "line point layer: x and y columns must have equal non-zero lengths".into(),
                ));
            }
            let points = data.collect_source((0..x.len()).map(|row| {
                data.claim_source_visits(1)?;
                Ok((
                    crate::data::strict_numeric_value(x, row, context)?,
                    crate::data::strict_numeric_value(y, row, context)?,
                ))
            }))?;
            let labels = layer
                .label_values()
                .map(|values| {
                    data.collect_source((0..values.len()).map(|row| {
                        data.claim_source_visits(1)?;
                        match values.get(row) {
                            Some(Some(label)) => data.copy_source_string(label),
                            _ => Err(Error::Data(format!(
                                "{context}: label contains null at source row {row}"
                            ))),
                        }
                    }))
                })
                .transpose()?;
            if labels
                .as_ref()
                .is_some_and(|labels| labels.len() != points.len())
            {
                return Err(Error::Data(
                    "line point layer: labels must align one-to-one with x and y".into(),
                ));
            }
            layers.push(PointLayer {
                points,
                labels,
                legend: layer
                    .legend_label_text()
                    .map(|label| data.copy_source_string(label))
                    .transpose()?,
            });
        }
        let mut ranges = [(f64::INFINITY, f64::NEG_INFINITY); 2];
        for group in &groups {
            data.claim_source_visits(group.points.len())?;
            for point in &group.points {
                ranges[0] = (ranges[0].0.min(point.x), ranges[0].1.max(point.x));
                let (lower, upper) = point.interval.unwrap_or((point.y, point.y));
                ranges[1] = (ranges[1].0.min(lower), ranges[1].1.max(upper));
            }
        }
        for layer in &layers {
            data.claim_source_visits(layer.points.len())?;
            for &(x, y) in &layer.points {
                ranges[0] = (ranges[0].0.min(x), ranges[0].1.max(x));
                ranges[1] = (ranges[1].0.min(y), ranges[1].1.max(y));
            }
        }
        ranges[0] = if let Some(levels) = &conditions {
            (0.0, categorical_index(levels.len())?)
        } else {
            crate::layout::continuous_auto_range(
                ranges[0],
                chart.x_axis_spec(),
                data.source_resources()?,
            )?
        };
        ranges[1] = crate::layout::continuous_auto_range(
            ranges[1],
            chart.y_axis_spec(),
            data.source_resources()?,
        )?;
        let primary_color_slots = if chart.hue_col().is_some() {
            hues.len()
        } else {
            1
        };
        Ok(Self {
            ranges,
            conditions,
            groups,
            hues,
            layers,
            primary_color_slots,
        })
    }
}

pub(crate) fn shared_x_domain(chart: &Plot, data: PlotData<'_>) -> Result<(f64, f64), Error> {
    let prepared = PreparedLine::new(chart, data, None)?;
    if prepared.conditions.is_some() {
        return Err(Error::Data(
            "canvas x-range group has a categorical x axis".into(),
        ));
    }
    Ok(prepared.ranges[0])
}

fn categorical_index(index: usize) -> Result<f64, Error> {
    u32::try_from(index).map(f64::from).map_err(|_| {
        Error::Data(
            "trajectory: categorical condition count exceeds the supported coordinate range".into(),
        )
    })
}

pub(crate) fn measure_marks(
    chart: &Plot,
    context: PlotRenderContext<'_>,
    layout: crate::layout::Layout,
) -> Result<PlotRenderResult, crate::layout::fit::PlanError> {
    #[cfg(test)]
    stage(chart.kind(), 1);
    let prepared = chart
        .prepared_line
        .as_ref()
        .and_then(|inputs| {
            inputs
                .panels
                .iter()
                .find(|(scope, _)| scope.as_deref() == chart.axis_scope())
        })
        .map(|(_, inputs)| inputs)
        .ok_or_else(|| {
            Error::Render(
                "line measurement requires root-prepared semantics for its component".into(),
            )
        })?;
    let theme = context.theme;
    let resources = theme.prepared_font_environment()?.generation_resources();
    let data = context.data.with_axis_resources(&resources);
    project(chart, prepared, layout, theme, data)
}

#[expect(
    clippy::too_many_lines,
    reason = "selected projection retains series, ribbon, marker and label order without a backend"
)]
fn project(
    chart: &Plot,
    prepared: &PreparedLine,
    layout: crate::layout::Layout,
    theme: &crate::theme::ResolvedTheme,
    data: PlotData<'_>,
) -> Result<PlotRenderResult, crate::layout::fit::PlanError> {
    #[cfg(test)]
    stage(chart.kind(), 2);
    let mut payload = LinePayload {
        #[cfg(test)]
        kind: chart.kind(),
        clip: layout.projection.panel_box(),
        marks: Vec::new(),
        layers: Vec::new(),
    };
    let project = |x, y| {
        let (x, y) = layout.projection.project(x, y);
        Point2D::new(x, y)
    };
    let hue_paths =
        chart.kind() == PlotKind::LinePath && chart.line_path_style_spec() == LinePathStyle::Hue;
    let order = prepared.group_order(chart, data)?;
    let legends = prepared.guide_entries_in_order(chart, theme, data, &order)?;
    for &index in &order {
        let group = &prepared.groups[index];
        let color = if chart.kind() == PlotKind::Line {
            let slot = group
                .identity
                .as_ref()
                .and_then(|identity| identity.hue.as_ref())
                .map_or(0, |hue| {
                    chart.hue_color_index(hue, group.hue_index.unwrap_or(0))
                });
            theme.colors[slot % theme.colors.len()]
        } else if hue_paths {
            let index = group.hue_index.ok_or_else(|| {
                Error::Render("line path: hue-styled path has no resolved hue index".into())
            })?;
            chart.categorical_color(
                "line path",
                &prepared.hues[index],
                chart.hue_color_index(&prepared.hues[index], index),
                theme,
            )?
        } else {
            theme.foreground
        };
        let fill = FillStyle {
            color,
            stroke: color,
            stroke_width: 0.0,
            opacity: 0.18,
        };
        data.claim_source_visits(group.points.len())?;
        let has_fill = chart.kind() == PlotKind::Line
            && group.points.iter().all(|point| point.interval.is_some());
        if has_fill {
            let mut polygon = Vec::new();
            let count = group
                .points
                .len()
                .checked_mul(2)
                .ok_or_else(|| Error::Render("line ribbon point count overflow".into()))?;
            data.reserve_source(&mut polygon, count)?;
            let mut has_area = false;
            for point in &group.points {
                let (lower, upper) = point.interval.expect("complete ribbon");
                has_area |= lower < upper;
                polygon.push(project(point.x, upper));
            }
            if let Some((first, last)) = group.points.first().zip(group.points.last()) {
                layout.projection.validate_span(
                    crate::layout::AxisSide::Bottom,
                    (first.x, last.x),
                    (polygon[0].x, polygon[polygon.len() - 1].x),
                    has_area && first.x < last.x,
                    "line ribbon polygon width",
                )?;
            }
            for (index, point) in group.points.iter().enumerate().rev() {
                let (lower, upper) = point.interval.expect("complete ribbon");
                let projected = project(point.x, lower);
                layout.projection.validate_span(
                    crate::layout::AxisSide::Left,
                    (lower, upper),
                    (projected.y, polygon[index].y),
                    lower < upper,
                    "line ribbon polygon",
                )?;
                polygon.push(projected);
            }
            data.reserve_source(&mut payload.marks, 1)?;
            payload.marks.push(LineMark::Ribbon(polygon, fill.clone()));
        }
        let points = data.collect_source(group.points.iter().map(|point| {
            data.claim_source_visits(1)?;
            Ok(project(point.x, point.y))
        }))?;
        if points.len() >= 2 {
            // Match canonical_line_points' global distinction rule, not just
            // adjacent segments. Keep source order and normalize signed zeros.
            let mut indices = data.collect_source((0..points.len()).map(Ok))?;
            crate::data::source::sort_source_by(&mut indices, Some(data), |a, b| {
                let key = |index: usize| {
                    let point = points[index];
                    let bits = |value: f64| if value == 0.0 { 0 } else { value.to_bits() };
                    (bits(point.x), bits(point.y), index)
                };
                key(*a).cmp(&key(*b))
            })?;
            data.claim_source_visits(indices.len() - 1)?;
            for pair in indices.windows(2) {
                let (a, b) = (pair[0], pair[1]);
                if points[a] != points[b] {
                    continue;
                }
                let (first, second) = (&group.points[a], &group.points[b]);
                for (axis, source, projected) in [
                    (
                        crate::layout::AxisSide::Bottom,
                        (first.x, second.x),
                        (points[a].x, points[b].x),
                    ),
                    (
                        crate::layout::AxisSide::Left,
                        (first.y, second.y),
                        (points[a].y, points[b].y),
                    ),
                ] {
                    layout.projection.validate_span(
                        axis,
                        source,
                        projected,
                        source.0.min(source.1) < source.0.max(source.1),
                        "line distinct source points",
                    )?;
                }
            }
        }
        let line = LineStyle::solid(
            color,
            if chart.kind() == PlotKind::Line {
                1.8
            } else {
                1.4
            },
        );
        let point_style =
            chart.point_style(color, if chart.kind() == PlotKind::Line { 3 } else { 4 });
        if chart.kind() == PlotKind::Line && chart.markers_visible() {
            data.reserve_source(&mut payload.marks, 2)?;
            let markers = data.collect_source(points.iter().copied().map(Ok))?;
            payload.marks.push(LineMark::Line(points, line.clone()));
            payload
                .marks
                .push(LineMark::Points(markers, point_style.clone()));
        } else {
            data.reserve_source(&mut payload.marks, 1)?;
            payload.marks.push(LineMark::Line(points, line.clone()));
        }
    }
    // Trajectories and paths paint every connector before any point.
    if chart.kind() != PlotKind::Line
        && (chart.kind() == PlotKind::Trajectory || chart.markers_visible())
    {
        for &index in &order {
            let group = &prepared.groups[index];
            for point in &group.points {
                data.claim_source_visits(1)?;
                let color = if hue_paths {
                    let index = group
                        .hue_index
                        .ok_or_else(|| Error::Render("line path: missing hue index".into()))?;
                    chart.categorical_color(
                        "line path",
                        &prepared.hues[index],
                        chart.hue_color_index(&prepared.hues[index], index),
                        theme,
                    )?
                } else {
                    let slot = point.hue_index.map_or(0, |index| {
                        chart.hue_color_index(&prepared.hues[index], index)
                    });
                    theme.colors[slot % theme.colors.len()]
                };
                data.reserve_source(&mut payload.marks, 1)?;
                payload.marks.push(LineMark::Points(
                    data.collect_source(std::iter::once(Ok(project(point.x, point.y))))?,
                    chart.point_style(color, 4),
                ));
            }
        }
    }
    data.reserve_source(&mut payload.layers, prepared.layers.len())?;
    for (index, layer) in prepared.layers.iter().enumerate() {
        let slots = if chart.hue_col().is_some() {
            chart
                .global_hue_levels()
                .map_or(prepared.primary_color_slots, <[String]>::len)
        } else {
            prepared.primary_color_slots
        };
        let style = chart.point_layers()[index]
            .point_style(theme.colors[(slots + index) % theme.colors.len()]);
        let points = data.collect_source(layer.points.iter().map(|&(x, y)| {
            data.claim_source_visits(1)?;
            Ok(project(x, y))
        }))?;
        let mut text = Vec::new();
        if let Some(labels) = &layer.labels {
            data.reserve_source(&mut text, labels.len())?;
            let x_range = layout.projection.x_range();
            let y_range = layout.projection.y_range();
            let offset = f64::from(style.radius) + theme.annotations.point_label_gap;
            for ((&(x, y), point), label) in layer.points.iter().zip(&points).zip(labels) {
                if x < x_range.0 || x > x_range.1 || y < y_range.0 || y > y_range.1 {
                    continue;
                }
                let mut buffer = [0_u8; 80];
                let mut cursor = std::io::Cursor::new(buffer.as_mut_slice());
                std::io::Write::write_fmt(
                    &mut cursor,
                    format_args!("line point layer {index} label"),
                )
                .expect("bounded layer index");
                let length = usize::try_from(cursor.position()).expect("bounded layer index");
                let owner = std::str::from_utf8(&buffer[..length]).expect("ASCII layer index");
                let mut appearance = theme.legacy_text_appearance(10, crate::TextWeight::Normal);
                appearance.line_height_multiplier = 1.2;
                appearance.paint = style.color.into();
                let plan = theme.measure_legacy_appearance_text(
                    label,
                    &appearance,
                    crate::layout::PhysicalTextAlignment::Start,
                    owner,
                    false,
                )?;
                text.push(PlacedText::middle(
                    plan,
                    Point2D::new(
                        (point.x + offset).max(0.0).round(),
                        (point.y + offset).max(0.0).round(),
                    ),
                    TextAnchor::Start,
                    0.0,
                ));
            }
        }
        payload.layers.push((points, style, text));
    }
    let clip = crate::render::backend::validate_clip_region(payload.clip)?.panel();
    let mut geometry = crate::render::geometry::GeometrySnapshot::new(theme)?;
    for mark in &payload.marks {
        match mark {
            LineMark::Ribbon(points, style) => geometry.polygon(points, style, Some(clip))?,
            LineMark::Line(points, style) => geometry.line(points, style, Some(clip))?,
            LineMark::Points(points, style) => geometry.scatter(points, style, clip)?,
        }
    }
    for (points, style, labels) in &payload.layers {
        geometry.scatter(points, style, clip)?;
        for text in labels {
            geometry.text(text.placement, &text.plan, Some(clip))?;
        }
    }
    let mut result = PlotRenderResult::new(layout, legends).with_axis_domains(
        if prepared.conditions.is_some() {
            AxisDomain::Categorical
        } else {
            AxisDomain::Continuous
        },
        AxisDomain::Continuous,
    );
    result.marks = Some(data.source_arc(crate::layout::interior::DeferredMarkPlan {
        payload: crate::layout::interior::MarkPayload::Line(payload),
        geometry,
    })?);
    Ok(result)
}

impl LinePayload {
    pub(crate) fn draw(&self, backend: &mut dyn Backend) -> Result<(), Error> {
        #[cfg(test)]
        stage(self.kind, 3);
        with_clip(backend, self.clip, |backend| {
            for mark in &self.marks {
                match mark {
                    LineMark::Ribbon(points, style) => backend.draw_polygon(points, style)?,
                    LineMark::Line(points, style) => backend.draw_line(points, style)?,
                    LineMark::Points(points, style) => backend.draw_scatter(points, style)?,
                }
            }
            Ok(())
        })?;
        if !self.layers.is_empty() {
            with_clip(backend, self.clip, |backend| {
                for (points, style, labels) in &self.layers {
                    backend.draw_scatter(points, style)?;
                    for text in labels {
                        text.draw(backend)?;
                    }
                }
                Ok(())
            })?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
