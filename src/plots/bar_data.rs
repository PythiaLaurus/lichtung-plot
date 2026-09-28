//! Shared data preparation for vertical and horizontal bar plots.

#![allow(clippy::cast_precision_loss)]

#[cfg(test)]
mod extraction;

use crate::data::{
    source::{sort_numeric_source, sort_source_by, SourceAdmission},
    PlotData,
};
use crate::Error;

use crate::decorations::legend::LegendEntry;
use crate::render::backend::{FillStyle, PointStyle};
use crate::spec::{
    BarInput, BarSource, ConfidenceIntervalAlgorithm, ErrorBarKind, PercentileInterval,
    StandardDeviation, StandardError,
};
use crate::theme::ResolvedTheme;
use crate::{CategoryOrder, Plot, ValueSort};

#[cfg(test)]
thread_local! {
    pub(crate) static ATTACHED_SIGNIFICANCE_TEXT_PLANNING_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static BOOTSTRAP_DRAWS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Bar arrangement mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BarMode {
    /// One aggregate bar per x category.
    Single,
    /// One bar per x category and series, dodged side-by-side.
    Grouped,
    /// One additive segment per x category and series.
    Stacked,
}

/// A prepared bar or stacked-bar segment in data coordinates.
#[derive(Debug, Clone)]
pub(crate) struct PreparedBar {
    /// Index into [`PreparedBars::categories`].
    pub(crate) category_index: usize,
    /// Optional index into [`PreparedBars::series`].
    pub(crate) series_index: Option<usize>,
    /// Optional index into [`PreparedBars::hues`] for color only.
    pub(crate) hue_index: Option<usize>,
    /// Start value on the numeric axis.
    pub(crate) start: f64,
    /// End value on the numeric axis.
    pub(crate) end: f64,
    /// Whether the aggregate is nonzero before cumulative stack addition.
    pub(crate) nonzero: bool,
    /// Optional explicit lower and upper interval endpoints around `end`.
    pub(crate) interval: Option<(f64, f64)>,
}

/// One attached significance label resolved to a rendered bar or category cluster.
#[derive(Debug, Clone)]
pub(crate) struct PreparedBarSignificance {
    pub(crate) category_index: usize,
    pub(crate) series_index: Option<usize>,
    pub(crate) edge: f64,
    pub(crate) positive: bool,
    pub(crate) label: String,
}

/// One strict supplemental point resolved to an ordinary bar anchor.
#[derive(Debug, Clone)]
pub(crate) struct PreparedBarPoint {
    pub(crate) category_index: usize,
    pub(crate) series_index: Option<usize>,
    pub(crate) value: f64,
    pub(crate) style: PointStyle,
}

/// Prepared bar data and metadata shared by both orientations.
#[derive(Debug, Clone)]
pub(crate) struct PreparedBars {
    /// Primary category labels in observed order.
    pub(crate) categories: Vec<String>,
    /// Series labels in observed order. Empty for `Single` mode.
    pub(crate) series: Vec<String>,
    /// Color labels derived from positioned series in observed order.
    pub(crate) hues: Vec<String>,
    /// Prepared bars/segments.
    pub(crate) bars: Vec<PreparedBar>,
    /// Numeric value range including zero and CI/stack extents.
    pub(crate) value_range: (f64, f64),
    /// Supplemental points at resolved category/series centers.
    pub(crate) points: Vec<PreparedBarPoint>,
    /// Significance labels attached to exact bar/category anchors.
    pub(crate) significance: Vec<PreparedBarSignificance>,
}

#[derive(Debug)]
pub(crate) struct PreparedBarInputs {
    pub(crate) panels: Vec<(Option<String>, PreparedBars)>,
}

#[derive(Debug)]
pub(crate) struct BarShape {
    pub(crate) polygon: [crate::render::backend::Point2D; 4],
    pub(crate) fill: FillStyle,
    pub(crate) interval: Option<[[crate::render::backend::Point2D; 2]; 3]>,
}

#[derive(Debug)]
pub(crate) struct BarPayload {
    pub(crate) clip: crate::layout::PanelBox,
    pub(crate) domain_expansion: Option<crate::layout::interior::DomainExpansion>,
    pub(crate) zero: Option<[crate::render::backend::Point2D; 2]>,
    pub(crate) zero_style: crate::render::backend::LineStyle,
    pub(crate) interval_style: crate::render::backend::LineStyle,
    pub(crate) bars: Vec<BarShape>,
    pub(crate) text: Vec<crate::render::backend::PlacedText>,
    pub(crate) points: Vec<(crate::render::backend::Point2D, PointStyle)>,
    #[cfg(test)]
    pub(crate) kind: crate::PlotKind,
}

impl BarPayload {
    pub(crate) fn draw(
        &self,
        backend: &mut dyn crate::render::backend::Backend,
    ) -> Result<(), Error> {
        #[cfg(test)]
        count(self.kind, 3);
        crate::render::backend::with_clip(backend, self.clip, |backend| {
            if let Some(line) = &self.zero {
                backend.draw_line(line, &self.zero_style)?;
            }
            for bar in &self.bars {
                backend.draw_polygon(&bar.polygon, &bar.fill)?;
                for line in bar.interval.iter().flatten() {
                    backend.draw_line(line, &self.interval_style)?;
                }
            }
            for text in &self.text {
                text.draw(backend)?;
            }
            for (point, style) in &self.points {
                backend.draw_scatter(std::slice::from_ref(point), style)?;
            }
            Ok(())
        })
    }

    pub(crate) fn retain(
        self,
        mut result: crate::spec::PlotRenderResult,
        context: crate::spec::PlotRenderContext<'_>,
        chart: &Plot,
    ) -> Result<crate::spec::PlotRenderResult, Error> {
        let resources = context
            .theme
            .prepared_font_environment()?
            .generation_resources();
        let data = context.data.with_axis_resources(&resources);
        let mut geometry = crate::render::geometry::GeometrySnapshot::new(context.theme)?;
        let clip = crate::render::backend::validate_clip_region(self.clip)?.panel();
        if let Some(line) = &self.zero {
            geometry.line(line, &self.zero_style, Some(clip))?;
        }
        for bar in &self.bars {
            geometry.polygon(&bar.polygon, &bar.fill, Some(clip))?;
            for line in bar.interval.iter().flatten() {
                geometry.line(line, &self.interval_style, Some(clip))?;
            }
        }
        for text in &self.text {
            geometry.text(text.placement, &text.plan, Some(clip))?;
        }
        for (point, style) in &self.points {
            geometry.scatter(std::slice::from_ref(point), style, clip)?;
        }
        if let Some(significance) = &result.significance {
            for bracket in &significance.brackets {
                let label = &bracket.label.plan;
                let style = crate::render::backend::BracketStyle {
                    color: context.theme.annotations.bracket_stroke.paint,
                    width: context.theme.annotations.bracket_stroke.width,
                    label_color: label.style.paint,
                    font_family: data.copy_source_string(&label.style.stack_source)?,
                    font_size: label.style.font_size,
                    line_height: label.line_height(),
                    weight: label.style.weight,
                    label_gap: chart.significance_mark_style_spec().gap_spec(),
                };
                geometry.bracket(crate::render::backend::PlannedSignificanceCall {
                    bracket: &bracket.geometry,
                    style: &style,
                    label: &bracket.label,
                })?;
            }
        }
        #[cfg(test)]
        count(self.kind, 2);
        result.marks = Some(data.source_arc(crate::layout::interior::DeferredMarkPlan {
            payload: crate::layout::interior::MarkPayload::Bars(self),
            geometry,
        })?);
        Ok(result)
    }
}

#[cfg(test)]
thread_local! {
    pub(crate) static STAGES: std::cell::Cell<[[usize; 4]; 4]> = const { std::cell::Cell::new([[0; 4]; 4]) };
}

#[cfg(test)]
pub(crate) fn count(kind: crate::PlotKind, stage: usize) {
    let family = match kind {
        crate::PlotKind::Bar => 0,
        crate::PlotKind::HorizontalBar => 1,
        crate::PlotKind::StackedBar => 2,
        crate::PlotKind::StackedHorizontalBar => 3,
        _ => unreachable!(),
    };
    STAGES.with(|counts| {
        let mut next = counts.get();
        next[family][stage] += 1;
        counts.set(next);
    });
}

pub(crate) fn mode(chart: &Plot) -> BarMode {
    if matches!(
        chart.kind(),
        crate::PlotKind::StackedBar | crate::PlotKind::StackedHorizontalBar
    ) {
        BarMode::Stacked
    } else if chart.group_col().is_some() {
        BarMode::Grouped
    } else {
        BarMode::Single
    }
}

pub(crate) fn retained(chart: &Plot) -> Result<&PreparedBars, Error> {
    chart
        .prepared_bars
        .as_ref()
        .and_then(|inputs| {
            inputs
                .panels
                .iter()
                .find(|(scope, _)| scope.as_deref() == chart.axis_scope())
        })
        .map(|(_, marks)| marks)
        .ok_or_else(|| Error::Render("bar measurement requires root-prepared semantics".into()))
}

pub(crate) fn domain_expansion(
    range: (f64, f64),
    marks: (f64, f64),
    brackets: (f64, f64),
    axis: crate::layout::interior::ClearanceAxis,
    resources: &crate::font::canonical::OperationGenerationResources,
) -> Result<Option<crate::layout::interior::DomainExpansion>, Error> {
    let limits = (marks.0.min(brackets.0), marks.1.max(brackets.1));
    if limits == range {
        return Ok(None);
    }
    // Bracket bisection gives the minimum for this projection's pixel clearances.
    // Reuse the bar's anchored automatic padding so reprojected attached marks
    // do not chase that boundary asymptotically across parent feedback rounds.
    let expanded = crate::layout::anchored_auto_range(
        limits,
        &crate::layout::AxisSpec::default(),
        0.0,
        resources,
    )?;
    Ok(Some(crate::layout::interior::DomainExpansion {
        axis,
        limits: (range.0.min(expanded.0), range.1.max(expanded.1)),
    }))
}

pub(crate) fn attached_significance_text_plans(
    context: crate::spec::PlotRenderContext<'_>,
    marks: &[PreparedBarSignificance],
    appearance: &crate::theme::TextAppearance,
    alignment: impl Fn(&PreparedBarSignificance) -> crate::layout::PhysicalTextAlignment,
    owner: &str,
    exact_override: bool,
) -> Result<Vec<crate::layout::ResolvedTextPlan>, Error> {
    #[cfg(test)]
    ATTACHED_SIGNIFICANCE_TEXT_PLANNING_CALLS.with(|calls| calls.set(calls.get() + 1));

    let resources = context
        .theme
        .prepared_font_environment()?
        .generation_resources();
    context
        .data
        .with_axis_resources(&resources)
        .collect_source(marks.iter().map(|mark| {
            context.theme.measure_text(
                &mark.label,
                appearance,
                alignment(mark),
                owner,
                exact_override,
            )
        }))
}

#[derive(Clone, Copy)]
struct BarSummaryInput<'a> {
    units: Option<&'a [String]>,
    input: &'a BarInput,
    facet_identity: &'a FacetIdentity,
    data: PlotData<'a>,
}

struct FacetIdentity(Vec<(String, String)>);

struct SingleBarCell {
    category_index: usize,
    hue_index: Option<usize>,
    values: Vec<f64>,
    units: Vec<String>,
}

struct GroupedBarCell {
    category_index: usize,
    series_index: usize,
    hue_index: Option<usize>,
    values: Vec<f64>,
    units: Vec<String>,
}

/// Build prepared bar data from category, value, and optional series channels.
pub(crate) fn prepare<'a>(
    chart: &Plot,
    data: impl Into<crate::data::PlotData<'a>>,
    mode: BarMode,
) -> Result<PreparedBars, Error> {
    let data = data.into();
    prepare_scoped(chart, data, mode, None, chart.significance_annotations())
}

pub(crate) fn prepare_scoped<'a>(
    chart: &Plot,
    data: PlotData<'_>,
    mode: BarMode,
    facet: Option<crate::data::FacetAxisSources<'_>>,
    annotations: impl IntoIterator<Item = &'a crate::significance::SignificanceAnnotation>,
) -> Result<PreparedBars, Error> {
    #[cfg(test)]
    count(chart.kind(), 0);
    let mut prepared = prepare_core(chart, data, mode, facet)?;
    apply_attached_significance(annotations, &mut prepared, mode, data)?;
    prepared.points = prepare_point_layers(chart, &prepared, mode, data)?;
    for point in &prepared.points {
        prepared.value_range.0 = prepared.value_range.0.min(point.value);
        prepared.value_range.1 = prepared.value_range.1.max(point.value);
    }
    Ok(prepared)
}

fn prepare_core(
    chart: &Plot,
    data: PlotData<'_>,
    mode: BarMode,
    facet: Option<crate::data::FacetAxisSources<'_>>,
) -> Result<PreparedBars, Error> {
    data.claim_mark_rows()?;
    let x_col = chart
        .x_col()
        .ok_or_else(|| Error::Data("bar: x column not set".into()))?;
    let y_col = chart
        .y_col()
        .ok_or_else(|| Error::Data("bar: y column not set".into()))?;

    let categories = crate::data::strict_strings(data, x_col, "bar")?;
    let values = crate::data::strict_finite_f64(data, y_col, "bar")?;

    let channels = crate::data::collective_channels(
        data,
        chart.group_col(),
        chart.hue_col(),
        Some(&categories),
        "bar",
    )?;
    let group_order = data.collect_source(
        facet
            .map_or_else(|| chart.global_group_levels(), |sources| sources.groups)
            .unwrap_or(&channels.group_levels)
            .iter()
            .map(|s| data.copy_source_string(s)),
    )?;
    let hue_order = data.collect_source(
        facet
            .map_or_else(|| chart.global_hue_levels(), |sources| sources.hues)
            .unwrap_or(&channels.hue_levels)
            .iter()
            .map(|s| data.copy_source_string(s)),
    )?;
    let units = chart
        .bar_units_col()
        .map(|column| crate::data::strict_strings(data, column, "bar sampling units"))
        .transpose()?;
    let facet_identity = facet_identity(chart, data)?;
    let summary = || -> Result<_, Error> {
        Ok(BarSummaryInput {
            units: units.as_deref(),
            input: chart
                .bar_input()
                .ok_or_else(|| Error::Data("ordinary bar input is not configured".into()))?,
            facet_identity: &facet_identity,
            data,
        })
    };
    let mut prepared = match mode {
        BarMode::Single => prepare_single(
            categories,
            channels.hues.as_deref(),
            values,
            hue_order,
            summary()?,
        ),
        BarMode::Grouped => {
            let groups = channels
                .groups
                .ok_or_else(|| Error::Data("clustered bar requires a series column".into()))?;
            prepare_grouped(
                categories,
                groups,
                channels.hues.as_deref(),
                values,
                group_order,
                hue_order,
                summary()?,
            )
        }
        BarMode::Stacked => {
            let groups = channels
                .groups
                .ok_or_else(|| Error::Data("stacked bar requires a series column".into()))?;
            prepare_stacked(
                categories,
                groups,
                channels.hues.as_deref(),
                values,
                group_order,
                hue_order,
                data,
            )
        }
    }?;
    apply_category_order(
        &mut prepared,
        chart.category_order_spec(),
        mode,
        !data.is_full_plot(),
        data,
    )?;
    if let Some(bounds) = chart.bar_bound_cols() {
        apply_precomputed_bounds(chart, data, &mut prepared, bounds)?;
    }
    Ok(prepared)
}

fn facet_identity(chart: &Plot, data: crate::data::PlotData<'_>) -> Result<FacetIdentity, Error> {
    let (row, column, wrap, _) = chart.facet_columns();
    let mut identity = Vec::new();
    for facet_column in [row, column, wrap].into_iter().flatten() {
        let values = crate::data::strict_strings(data, facet_column, "bar facet identity")?;
        let level = values
            .first()
            .ok_or_else(|| Error::Data("bar facet cell contains no observations".into()))?;
        data.claim_source_visits(values.len())?;
        if values.iter().any(|value| value != level) {
            return Err(Error::Data(
                "bar facet preparation mixed multiple levels in one panel".into(),
            ));
        }
        let name = data.column(facet_column)?.name();
        data.reserve_source(&mut identity, 1)?;
        identity.push((
            data.copy_source_string(name)?,
            data.copy_source_string(level)?,
        ));
    }
    Ok(FacetIdentity(identity))
}

fn apply_attached_significance<'a>(
    annotations: impl IntoIterator<Item = &'a crate::significance::SignificanceAnnotation>,
    prepared: &mut PreparedBars,
    mode: BarMode,
    data: PlotData<'_>,
) -> Result<(), Error> {
    for annotation in annotations {
        let Some(target) = annotation.at_target() else {
            continue;
        };
        data.claim_source_visits(prepared.categories.len())?;
        data.claim_source_visits(prepared.series.len())?;
        data.claim_source_visits(prepared.significance.len())?;
        data.claim_source_visits(prepared.bars.len())?;
        let category_index = prepared
            .categories
            .iter()
            .position(|category| category == target.category_name())
            .ok_or_else(|| {
                Error::Data(format!(
                    "bar significance references unknown category {:?}; available categories: {}",
                    target.category_name(),
                    prepared.categories.join(", ")
                ))
            })?;
        let series_index = target
            .series_name()
            .map(|series| {
                prepared
                    .series
                    .iter()
                    .position(|candidate| candidate == series)
                    .ok_or_else(|| {
                        Error::Data(format!(
                            "bar significance references unknown series {series:?}; available series: {}",
                            prepared.series.join(", ")
                        ))
                    })
            })
            .transpose()?;
        if target.series_name().is_some() && mode == BarMode::Single {
            return Err(Error::Data(
                "series-qualified significance target requires a configured bar series".into(),
            ));
        }
        if prepared
            .significance
            .iter()
            .any(|mark| mark.category_index == category_index && mark.series_index == series_index)
        {
            return Err(Error::Data(format!(
                "bar significance target {:?} is annotated more than once",
                target.category_name()
            )));
        }
        let bars = prepared.bars.iter().filter(|bar| {
            bar.category_index == category_index
                && series_index.is_none_or(|index| bar.series_index == Some(index))
        });
        let mut positive_edge = None::<f64>;
        let mut negative_edge = None::<f64>;
        for bar in bars {
            if bar.end >= 0.0 {
                let edge = bar
                    .interval
                    .map_or(bar.end, |(_, upper)| upper.max(bar.end));
                positive_edge = Some(positive_edge.map_or(edge, |current| current.max(edge)));
            } else {
                let edge = bar
                    .interval
                    .map_or(bar.end, |(lower, _)| lower.min(bar.end));
                negative_edge = Some(negative_edge.map_or(edge, |current| current.min(edge)));
            }
        }
        let (edge, positive) = positive_edge
            .map(|edge| (edge, true))
            .or_else(|| negative_edge.map(|edge| (edge, false)))
            .ok_or_else(|| Error::Data("bar significance target has no rendered bar".into()))?;
        data.reserve_source(&mut prepared.significance, 1)?;
        prepared.significance.push(PreparedBarSignificance {
            category_index,
            series_index,
            edge,
            positive,
            label: data.copy_source_string(annotation.mark().glyph()?)?,
        });
    }
    Ok(())
}

fn apply_precomputed_bounds(
    chart: &Plot,
    data: crate::data::PlotData<'_>,
    prepared: &mut PreparedBars,
    (lower_column, upper_column): (&str, &str),
) -> Result<(), Error> {
    let category_column = chart
        .x_col()
        .ok_or_else(|| Error::Data("bar: x column not set".into()))?;
    let categories = crate::data::strict_strings(data, category_column, "bar estimate bounds")?;
    let groups = chart
        .group_col()
        .map(|column| crate::data::strict_strings(data, column, "bar estimate bounds"))
        .transpose()?;
    let lower = crate::data::strict_finite_f64(data, lower_column, "bar estimate bounds")?;
    let upper = crate::data::strict_finite_f64(data, upper_column, "bar estimate bounds")?;
    if lower.len() != categories.len() || upper.len() != categories.len() {
        return Err(Error::Data(
            "bar estimate bounds: category/lower/upper length mismatch".into(),
        ));
    }
    for row in 0..categories.len() {
        data.claim_source_visits(prepared.categories.len())?;
        if groups.is_some() {
            data.claim_source_visits(prepared.series.len())?;
        }
        data.claim_source_visits(prepared.bars.len())?;
        let category_index = prepared
            .categories
            .iter()
            .position(|category| category == &categories[row])
            .ok_or_else(|| {
                Error::Data("bar estimate bounds category disappeared during preparation".into())
            })?;
        let series_index = groups
            .as_ref()
            .map(|values| {
                prepared
                    .series
                    .iter()
                    .position(|group| group == &values[row])
                    .ok_or_else(|| {
                        Error::Data(
                            "bar estimate bounds series disappeared during preparation".into(),
                        )
                    })
            })
            .transpose()?;
        let bar = prepared
            .bars
            .iter_mut()
            .find(|bar| bar.category_index == category_index && bar.series_index == series_index)
            .ok_or_else(|| {
                Error::Data("bar estimate bounds cell disappeared during preparation".into())
            })?;
        if lower[row] > bar.end || bar.end > upper[row] {
            return Err(Error::Data(format!(
                "bar estimate bounds row {row} requires finite lower <= value <= upper; got {} <= {} <= {}",
                lower[row], bar.end, upper[row]
            )));
        }
        bar.interval = Some((lower[row], upper[row]));
    }
    data.claim_source_visits(prepared.bars.len())?;
    prepared.value_range = value_range(&prepared.bars)?;
    Ok(())
}

fn prepare_point_layers(
    chart: &Plot,
    prepared: &PreparedBars,
    mode: BarMode,
    data: PlotData<'_>,
) -> Result<Vec<PreparedBarPoint>, Error> {
    if chart.bar_point_layers().is_empty() {
        return Ok(Vec::new());
    }
    if mode == BarMode::Stacked {
        return Err(Error::Data(
            "bar point layers are not supported by stacked bars".into(),
        ));
    }
    let has_series = chart.group_col().is_some();
    let mut points = Vec::new();
    for (layer_index, layer) in chart.bar_point_layers().iter().enumerate() {
        let context = format!("bar point layer {layer_index}");
        if layer.series_values().is_some() != has_series {
            return Err(Error::Data(format!(
                "{context}: layer series configuration must match the bar chart"
            )));
        }
        let category_values = |values: &crate::CategoryValues| {
            data.collect_source((0..values.len()).map(|row| {
                data.claim_source_visits(1)?;
                match values.get(row) {
                    Some(Some(value)) => data.copy_source_string(value),
                    _ => Err(Error::Data(format!(
                        "{context}: value {:?} contains null at source row {row}",
                        values.name()
                    ))),
                }
            }))
        };
        let categories = category_values(layer.category_values())?;
        let series = layer.series_values().map(category_values).transpose()?;
        let values = data.collect_source((0..layer.values().len()).map(|row| {
            data.claim_source_visits(1)?;
            crate::data::strict_numeric_value(layer.values(), row, &context)
        }))?;
        if categories.is_empty() || categories.len() != values.len() {
            return Err(Error::Data(format!(
                "{context}: category and value columns must have equal non-zero lengths"
            )));
        }
        for row in 0..values.len() {
            data.claim_source_visits(prepared.categories.len())?;
            data.claim_source_visits(prepared.series.len())?;
            data.claim_source_visits(prepared.bars.len())?;
            let category_index = prepared
                .categories
                .iter()
                .position(|value| value == &categories[row])
                .ok_or_else(|| {
                    Error::Data(format!(
                        "{context}: unknown category {:?} at row {row}",
                        categories[row]
                    ))
                })?;
            let series_index = series
                .as_ref()
                .map(|series_values| {
                    prepared
                        .series
                        .iter()
                        .position(|value| value == &series_values[row])
                        .ok_or_else(|| {
                            Error::Data(format!(
                                "{context}: unknown series {:?} at row {row}",
                                series_values[row]
                            ))
                        })
                })
                .transpose()?;
            if !prepared
                .bars
                .iter()
                .any(|bar| bar.category_index == category_index && bar.series_index == series_index)
            {
                return Err(Error::Data(format!(
                    "{context}: category/series at row {row} does not resolve to a prepared bar cell"
                )));
            }
            data.reserve_source(&mut points, 1)?;
            points.push(PreparedBarPoint {
                category_index,
                series_index,
                value: values[row],
                style: layer.point_style()?,
            });
        }
    }
    Ok(points)
}

/// Validate cardinality and strict values for typed ordinary-bar inputs.
#[expect(
    clippy::too_many_lines,
    reason = "one pass preserves source-row, facet, category, and series validation context"
)]
pub(crate) fn validate_cells(chart: &Plot, data: crate::data::PlotData<'_>) -> Result<(), Error> {
    let x_col = chart
        .x_col()
        .ok_or_else(|| Error::Data("bar: x column not set".into()))?;
    let y_col = chart
        .y_col()
        .ok_or_else(|| Error::Data("bar: y column not set".into()))?;
    let categories = crate::data::strict_strings(data, x_col, "bar")?;
    let values = crate::data::strict_finite_f64(data, y_col, "bar")?;
    let category_name = data.column(x_col)?.name();
    let facet_columns = {
        let (row, column, wrap, _) = chart.facet_columns();
        [row, column, wrap]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
    };
    let facet_values = facet_columns
        .iter()
        .map(|column| crate::data::strict_strings(data, column, "bar"))
        .collect::<Result<Vec<_>, _>>()?;
    let facet_names = facet_columns
        .iter()
        .map(|column| data.column(column).map(crate::data::DataColumn::name))
        .collect::<Result<Vec<_>, _>>()?;
    let mark_cells = (0..values.len())
        .map(|index| {
            let mut identity = String::new();
            for field in facet_values
                .iter()
                .map(|facet| facet[index].as_str())
                .chain(std::iter::once(categories[index].as_str()))
            {
                identity.push_str(&field.len().to_string());
                identity.push(':');
                identity.push_str(field);
            }
            identity
        })
        .collect::<Vec<_>>();
    let channels = crate::data::collective_channels(
        data,
        chart.group_col(),
        chart.hue_col(),
        Some(&mark_cells),
        "bar",
    )?;
    if categories.len() != values.len()
        || channels
            .groups
            .as_ref()
            .is_some_and(|groups| groups.len() != values.len())
        || facet_values.iter().any(|facet| facet.len() != values.len())
    {
        return Err(Error::Data(
            "bar: category/value/series/facet length mismatch".into(),
        ));
    }

    let mut cells: Vec<(Vec<String>, usize)> = Vec::new();
    for index in 0..values.len() {
        let mut key = facet_values
            .iter()
            .map(|facet| facet[index].clone())
            .collect::<Vec<_>>();
        key.push(categories[index].clone());
        if let Some(groups) = &channels.groups {
            key.push(groups[index].clone());
        }
        if let Some((_, rows)) = cells.iter_mut().find(|(existing, _)| existing == &key) {
            *rows += 1;
        } else {
            cells.push((key, 1));
        }
    }
    if cells.is_empty() {
        return Err(Error::Data(
            "bar input requires at least one rendered category cell".into(),
        ));
    }
    let input = chart
        .bar_input()
        .ok_or_else(|| Error::Data("ordinary bar input is not configured".into()))?;
    if let Some(units_column) = chart.bar_units_col() {
        crate::data::strict_strings(data, units_column, "bar sampling units")?;
    }
    if let Some((lower_column, upper_column)) = chart.bar_bound_cols() {
        let lower = crate::data::strict_finite_f64(data, lower_column, "bar estimate bounds")?;
        let upper = crate::data::strict_finite_f64(data, upper_column, "bar estimate bounds")?;
        for row in 0..values.len() {
            if lower[row] > values[row] || values[row] > upper[row] {
                return Err(Error::Data(format!(
                    "bar estimate bounds row {row} requires finite lower <= value <= upper; got {} <= {} <= {}",
                    lower[row], values[row], upper[row]
                )));
            }
        }
    }
    for (key, rows) in cells {
        let mut labels = facet_names
            .iter()
            .zip(&key)
            .map(|(column, level)| format!("facet {column}={level}"))
            .collect::<Vec<_>>();
        let category_index = facet_columns.len();
        labels.push(format!("category {category_name}={}", key[category_index]));
        if let Some(series_name) = chart.group_name() {
            labels.push(format!("series {series_name}={}", key[category_index + 1]));
        }
        let cell = labels.join(", ");
        if !input.is_samples() && rows != 1 {
            return Err(Error::Data(format!(
                "bar estimate cell `{cell}` has {rows} finite observations; exactly one is required"
            )));
        }
    }
    Ok(())
}

fn apply_category_order(
    prepared: &mut PreparedBars,
    order: &CategoryOrder,
    mode: BarMode,
    is_subset: bool,
    data: PlotData<'_>,
) -> Result<(), Error> {
    match order {
        CategoryOrder::Observed => Ok(()),
        CategoryOrder::Explicit(requested) => {
            let ordered = crate::data::explicit_category_order(
                &prepared.categories,
                requested,
                "bar category order",
                is_subset,
                Some(data),
            )?;
            let old_categories = std::mem::replace(&mut prepared.categories, ordered);
            for bar in &mut prepared.bars {
                data.claim_source_visits(prepared.categories.len())?;
                let label = &old_categories[bar.category_index];
                bar.category_index = prepared
                    .categories
                    .iter()
                    .position(|candidate| candidate == label)
                    .ok_or_else(|| {
                        Error::Render(format!(
                            "bar category {label:?} was lost while applying explicit order"
                        ))
                    })?;
            }
            Ok(())
        }
        CategoryOrder::Value(sort) if mode == BarMode::Single => {
            sort_single_values(prepared, *sort, data)
        }
        CategoryOrder::Value(_) => Err(Error::Data(
            "bar: value-derived category order requires one ungrouped bar per category; use an explicit category order for grouped or stacked bars"
                .into(),
        )),
    }
}

fn sort_single_values(
    prepared: &mut PreparedBars,
    sort: ValueSort,
    data: PlotData<'_>,
) -> Result<(), Error> {
    let mut order = data.collect_source((0..prepared.categories.len()).map(Ok))?;
    sort_source_by(&mut order, Some(data), |left, right| {
        let left_value = prepared.bars[*left].end;
        let right_value = prepared.bars[*right].end;
        (match sort {
            ValueSort::Ascending => left_value.total_cmp(&right_value),
            ValueSort::Descending => right_value.total_cmp(&left_value),
        })
        .then_with(|| left.cmp(right))
    })?;
    let categories = data.collect_source(
        order
            .iter()
            .map(|index| data.copy_source_string(&prepared.categories[*index])),
    )?;
    let bars =
        data.collect_source(order.iter().enumerate().map(|(category_index, old_index)| {
            let mut bar = prepared.bars[*old_index].clone();
            bar.category_index = category_index;
            Ok(bar)
        }))?;
    prepared.categories = categories;
    prepared.bars = bars;
    Ok(())
}

/// Legend entries for positioned series.
pub(crate) fn legend_entries(
    prepared: &PreparedBars,
    chart: &Plot,
    theme: &ResolvedTheme,
    data: PlotData<'_>,
) -> Result<Vec<LegendEntry>, Error> {
    data.collect_source(prepared.hues.iter().enumerate().map(|(index, label)| {
        Ok(LegendEntry::fill(
            chart.point_legend_identity(label, data)?,
            theme
                .marks
                .bar_body
                .backend_style(theme.colors[index % theme.colors.len()]),
        ))
    }))
}

fn prepare_single(
    categories: Vec<String>,
    hues: Option<&[String]>,
    values: Vec<f64>,
    mut hue_levels: Vec<String>,
    summary: BarSummaryInput<'_>,
) -> Result<PreparedBars, Error> {
    let data = summary.data;
    let mut category_levels = Vec::new();
    let mut groups: Vec<SingleBarCell> = Vec::new();
    for (row, (category, value)) in categories.into_iter().zip(values).enumerate() {
        let category_index = level_index(&mut category_levels, category, data)?;
        let hue_index = hues
            .map(|hues| level_index(&mut hue_levels, data.copy_source_string(&hues[row])?, data))
            .transpose()?;
        if let Some(index) =
            data.source_position(&groups, |cell| cell.category_index == category_index)?
        {
            let cell = &mut groups[index];
            data.reserve_source(&mut cell.values, 1)?;
            cell.values.push(value);
            if let Some(units) = summary.units {
                data.reserve_source(&mut cell.units, 1)?;
                cell.units.push(data.copy_source_string(&units[row])?);
            }
        } else {
            data.reserve_source(&mut groups, 1)?;
            groups.push(SingleBarCell {
                category_index,
                hue_index,
                values: data.collect_source(std::iter::once(Ok(value)))?,
                units: summary
                    .units
                    .map(|units| {
                        data.collect_source(std::iter::once_with(|| {
                            data.copy_source_string(&units[row])
                        }))
                    })
                    .transpose()?
                    .unwrap_or_default(),
            });
        }
    }
    if groups.is_empty() {
        return Err(Error::Data("bar: no groups".into()));
    }

    let mut bars = Vec::new();
    data.reserve_source(&mut bars, groups.len())?;
    for cell in &groups {
        let category = &category_levels[cell.category_index];
        let (value, interval) = summarize(
            &cell.values,
            summary.units.map(|_| cell.units.as_slice()),
            || format!("category {category:?}"),
            category,
            None,
            summary,
        )?;
        bars.push(PreparedBar {
            category_index: cell.category_index,
            series_index: None,
            hue_index: cell.hue_index,
            start: 0.0,
            end: value,
            nonzero: value != 0.0,
            interval,
        });
    }
    data.claim_source_visits(bars.len())?;
    let value_range = value_range(&bars)?;
    Ok(PreparedBars {
        categories: category_levels,
        series: Vec::new(),
        hues: hue_levels,
        value_range,
        bars,
        points: Vec::new(),
        significance: Vec::new(),
    })
}

fn prepare_grouped(
    categories: Vec<String>,
    series: Vec<String>,
    hues: Option<&[String]>,
    values: Vec<f64>,
    series_order: Vec<String>,
    hue_order: Vec<String>,
    summary: BarSummaryInput<'_>,
) -> Result<PreparedBars, Error> {
    let data = summary.data;
    if categories.len() != series.len() || categories.len() != values.len() {
        return Err(Error::Data(
            "bar: category/value/series length mismatch".into(),
        ));
    }
    let mut category_levels = Vec::new();
    let mut series_levels = series_order;
    let mut hue_levels = hue_order;
    let mut groups: Vec<GroupedBarCell> = Vec::new();

    for (row, ((category, series_label), value)) in
        categories.into_iter().zip(series).zip(values).enumerate()
    {
        let category_index = level_index(&mut category_levels, category, data)?;
        let series_index = level_index(&mut series_levels, series_label, data)?;
        let hue_index = hues
            .as_ref()
            .map(|hues| level_index(&mut hue_levels, data.copy_source_string(&hues[row])?, data))
            .transpose()?;
        if let Some(index) = data.source_position(&groups, |cell| {
            cell.category_index == category_index && cell.series_index == series_index
        })? {
            let cell = &mut groups[index];
            data.reserve_source(&mut cell.values, 1)?;
            cell.values.push(value);
            if let Some(units) = summary.units {
                data.reserve_source(&mut cell.units, 1)?;
                cell.units.push(data.copy_source_string(&units[row])?);
            }
        } else {
            data.reserve_source(&mut groups, 1)?;
            groups.push(GroupedBarCell {
                category_index,
                series_index,
                hue_index,
                values: data.collect_source(std::iter::once(Ok(value)))?,
                units: summary
                    .units
                    .map(|units| {
                        data.collect_source(std::iter::once_with(|| {
                            data.copy_source_string(&units[row])
                        }))
                    })
                    .transpose()?
                    .unwrap_or_default(),
            });
        }
    }
    if groups.is_empty() {
        return Err(Error::Data("bar: no groups".into()));
    }

    let mut bars = Vec::new();
    data.reserve_source(&mut bars, groups.len())?;
    for cell in groups {
        let context = || {
            format!(
                "category {:?}, series {:?}",
                category_levels[cell.category_index], series_levels[cell.series_index]
            )
        };
        let (value, interval) = summarize(
            &cell.values,
            summary.units.map(|_| cell.units.as_slice()),
            context,
            &category_levels[cell.category_index],
            Some(&series_levels[cell.series_index]),
            summary,
        )?;
        bars.push(PreparedBar {
            category_index: cell.category_index,
            series_index: Some(cell.series_index),
            hue_index: cell.hue_index,
            start: 0.0,
            end: value,
            nonzero: value != 0.0,
            interval,
        });
    }
    data.claim_source_visits(bars.len())?;
    let value_range = value_range(&bars)?;

    Ok(PreparedBars {
        categories: category_levels,
        series: series_levels,
        hues: hue_levels,
        value_range,
        bars,
        points: Vec::new(),
        significance: Vec::new(),
    })
}

#[expect(
    clippy::too_many_lines,
    reason = "admitted grouping and ordered signed stack accumulation share one scientific owner"
)]
fn prepare_stacked(
    categories: Vec<String>,
    series: Vec<String>,
    hues: Option<&[String]>,
    values: Vec<f64>,
    series_order: Vec<String>,
    hue_order: Vec<String>,
    data: PlotData<'_>,
) -> Result<PreparedBars, Error> {
    if categories.len() != series.len() || categories.len() != values.len() {
        return Err(Error::Data(
            "bar: category/value/series length mismatch".into(),
        ));
    }
    let mut category_levels = Vec::new();
    let mut series_levels = series_order;
    let mut hue_levels = hue_order;
    let mut groups: Vec<(usize, usize, Option<usize>, Vec<f64>)> = Vec::new();

    for (row, ((category, series_label), value)) in
        categories.into_iter().zip(series).zip(values).enumerate()
    {
        let category_index = level_index(&mut category_levels, category, data)?;
        let series_index = level_index(&mut series_levels, series_label, data)?;
        let hue_index = hues
            .as_ref()
            .map(|hues| level_index(&mut hue_levels, data.copy_source_string(&hues[row])?, data))
            .transpose()?;
        if let Some(index) = data.source_position(&groups, |(c, s, _, _)| {
            *c == category_index && *s == series_index
        })? {
            let (_, _, _, values) = &mut groups[index];
            data.reserve_source(values, 1)?;
            values.push(value);
        } else {
            data.reserve_source(&mut groups, 1)?;
            groups.push((
                category_index,
                series_index,
                hue_index,
                data.collect_source(std::iter::once(Ok(value)))?,
            ));
        }
    }
    if groups.is_empty() {
        return Err(Error::Data("bar: no groups".into()));
    }
    let sums = data.collect_source(groups.into_iter().map(
        |(category_index, series_index, hue_index, values)| {
            stable_sum(&values, Some(data)).map_or_else(
                |error| {
                    Err(match error {
                        Error::Data(reason) => Error::Data(format!(
                            "bar: stacked category {:?}, series {:?} {reason}",
                            category_levels[category_index], series_levels[series_index]
                        )),
                        error => error,
                    })
                },
                |sum| Ok((category_index, series_index, hue_index, sum)),
            )
        },
    ))?;

    let mut positive_offsets = data.collect_source((0..category_levels.len()).map(|_| Ok(0.0)))?;
    let mut negative_offsets = data.collect_source((0..category_levels.len()).map(|_| Ok(0.0)))?;
    let mut bars = Vec::new();
    data.reserve_source(&mut bars, sums.len())?;
    for series_index in 0..series_levels.len() {
        for category_index in 0..category_levels.len() {
            let Some(index) = data.source_position(&sums, |(c, s, _, _)| {
                *c == category_index && *s == series_index
            })?
            else {
                continue;
            };
            let (_, _, hue_index, value) = &sums[index];
            let (start, end) = if *value >= 0.0 {
                let start = positive_offsets[category_index];
                positive_offsets[category_index] += value;
                if !positive_offsets[category_index].is_finite() {
                    return Err(Error::Data(format!(
                        "bar: positive stack geometry for category {:?} is not representable",
                        category_levels[category_index]
                    )));
                }
                (start, positive_offsets[category_index])
            } else {
                let start = negative_offsets[category_index];
                negative_offsets[category_index] += value;
                if !negative_offsets[category_index].is_finite() {
                    return Err(Error::Data(format!(
                        "bar: negative stack geometry for category {:?} is not representable",
                        category_levels[category_index]
                    )));
                }
                (start, negative_offsets[category_index])
            };
            bars.push(PreparedBar {
                category_index,
                series_index: Some(series_index),
                hue_index: *hue_index,
                start,
                end,
                nonzero: *value != 0.0,
                interval: None,
            });
        }
    }

    data.claim_source_visits(bars.len())?;
    let value_range = value_range(&bars)?;
    Ok(PreparedBars {
        categories: category_levels,
        series: series_levels,
        hues: hue_levels,
        value_range,
        bars,
        points: Vec::new(),
        significance: Vec::new(),
    })
}

fn level_index(
    levels: &mut Vec<String>,
    label: String,
    data: PlotData<'_>,
) -> Result<usize, Error> {
    if let Some(index) = data.source_position(levels, |existing| existing == &label)? {
        Ok(index)
    } else {
        let index = levels.len();
        data.reserve_source(levels, 1)?;
        levels.push(label);
        Ok(index)
    }
}

fn summarize(
    values: &[f64],
    units: Option<&[String]>,
    context: impl Fn() -> String,
    category: &str,
    series: Option<&str>,
    summary: BarSummaryInput<'_>,
) -> Result<(f64, Option<(f64, f64)>), Error> {
    let data = Some(summary.data);
    let contextualize = |error| match error {
        Error::Data(reason) => Error::Data(format!("bar: {} {reason}", context())),
        error => error,
    };
    match &summary.input.source {
        BarSource::Samples { error_bar, .. } => {
            let mean = stable_mean(values, data).map_err(&contextualize)?;
            let interval = generated_interval(
                values,
                units,
                mean,
                error_bar.kind(),
                summary.facet_identity,
                category,
                series,
                data,
            )
            .map_err(contextualize)?;
            Ok((mean, interval))
        }
        BarSource::Estimates { .. } => match values {
            [value] if value.is_finite() => Ok((*value, None)),
            _ => Err(Error::Data(format!(
                "bar: {} precomputed estimates require exactly one finite value, got {}",
                context(),
                values.len()
            ))),
        },
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "bootstrap identity and admission accompany every interval algorithm"
)]
fn generated_interval(
    values: &[f64],
    units: Option<&[String]>,
    mean: f64,
    error_bar: &ErrorBarKind,
    facet_identity: &FacetIdentity,
    category: &str,
    series: Option<&str>,
    data: Option<PlotData<'_>>,
) -> Result<Option<(f64, f64)>, Error> {
    if values.len() == 1 || matches!(error_bar, ErrorBarKind::Hidden) {
        return Ok(None);
    }
    match error_bar {
        ErrorBarKind::Hidden => Ok(None),
        ErrorBarKind::StandardDeviation(StandardDeviation { scale }) => {
            symmetric_interval(mean, sample_standard_deviation(values, data)? * scale)
                .map_err(stat_error)
        }
        ErrorBarKind::PercentileInterval(PercentileInterval { level }) => {
            central_quantiles(values, *level, data).map(Some)
        }
        ErrorBarKind::StandardError(StandardError { scale }) => symmetric_interval(
            mean,
            sample_standard_deviation(values, data)? / (values.len() as f64).sqrt() * scale,
        )
        .map_err(stat_error),
        ErrorBarKind::ConfidenceInterval(interval) => match &interval.algorithm {
            ConfidenceIntervalAlgorithm::Bootstrap(bootstrap) => {
                if let Some(units) = units {
                    data.source_visits(units.len())?;
                    if units
                        .first()
                        .is_some_and(|first| units.iter().all(|unit| unit == first))
                    {
                        return Ok(None);
                    }
                }
                // Charge the actual fields hashed into the deterministic cell stream.
                data.source_visits(16)?;
                for (name, level) in &facet_identity.0 {
                    data.source_visits(16)?;
                    data.source_visits(name.len())?;
                    data.source_visits(level.len())?;
                }
                data.source_visits(9)?;
                data.source_visits(category.len())?;
                if let Some(series) = series {
                    data.source_visits(8)?;
                    data.source_visits(series.len())?;
                }
                let seed = stable_cell_seed(bootstrap.seed, facet_identity, category, series);
                percentile_bootstrap(
                    values,
                    units,
                    interval.level,
                    bootstrap.resamples,
                    seed,
                    data,
                )
                .map(Some)
            }
            ConfidenceIntervalAlgorithm::StudentT => {
                data.source_visits(1)?;
                let critical = crate::stats::student_t_critical(values.len() - 1, interval.level)
                    .map_err(stat_error)?;
                symmetric_interval(
                    mean,
                    critical * sample_standard_deviation(values, data)?
                        / (values.len() as f64).sqrt(),
                )
                .map_err(stat_error)
            }
        },
    }
}

fn symmetric_interval(mean: f64, half_width: f64) -> Result<Option<(f64, f64)>, &'static str> {
    let lower = mean - half_width;
    let upper = mean + half_width;
    if !half_width.is_finite() || half_width < 0.0 || !lower.is_finite() || !upper.is_finite() {
        return Err("error-bar geometry is not numerically representable");
    }
    Ok(Some((lower, upper)))
}

fn stat_error(reason: &str) -> Error {
    Error::Data(reason.to_owned())
}

fn sample_standard_deviation(values: &[f64], data: Option<PlotData<'_>>) -> Result<f64, Error> {
    if values.len() < 2 {
        return Err(stat_error(
            "sample standard deviation requires at least two observations",
        ));
    }
    data.source_visits(values.len())?;
    let scale = values.iter().map(|value| value.abs()).fold(0.0, f64::max);
    if !scale.is_finite() {
        return Err(stat_error("contains non-finite observations"));
    }
    if scale == 0.0 {
        return Ok(0.0);
    }
    let mut mean = 0.0;
    let mut squared_deviations = 0.0;
    data.source_visits(values.len())?;
    for (index, value) in values.iter().enumerate() {
        let normalized = value / scale;
        let count = (index + 1) as f64;
        let delta = normalized - mean;
        mean += delta / count;
        squared_deviations += delta * (normalized - mean);
    }
    let deviation = (squared_deviations / (values.len() - 1) as f64).sqrt() * scale;
    if deviation.is_finite() {
        Ok(deviation)
    } else {
        Err(stat_error(
            "sample standard deviation is not numerically representable",
        ))
    }
}

fn central_quantiles(
    values: &[f64],
    level: f64,
    data: Option<PlotData<'_>>,
) -> Result<(f64, f64), Error> {
    let tail = (1.0 - level) / 2.0;
    data.source_visits(values.len())?;
    if values.is_empty()
        || !tail.is_finite()
        || !(0.0..=1.0).contains(&tail)
        || values.iter().any(|value| !value.is_finite())
    {
        return Err(stat_error(
            "could not compute the lower error-bar percentile",
        ));
    }
    let mut sorted = Vec::new();
    data.source_reserve(&mut sorted, values.len())?;
    sorted.extend_from_slice(values);
    sort_numeric_source(&mut sorted, data)?;
    let lower = crate::stats::distribution::sorted_quantile(&sorted, tail);
    let upper = crate::stats::distribution::sorted_quantile(&sorted, 1.0 - tail);
    if lower.is_finite() && upper.is_finite() && lower <= upper {
        Ok((lower, upper))
    } else {
        Err(stat_error(
            "percentile error-bar geometry is not numerically representable",
        ))
    }
}

fn percentile_bootstrap(
    values: &[f64],
    units: Option<&[String]>,
    level: f64,
    resamples: usize,
    seed: u64,
    data: Option<PlotData<'_>>,
) -> Result<(f64, f64), Error> {
    if units.is_some_and(|units| units.len() != values.len()) {
        return Err(stat_error("sampling units must align with observations"));
    }
    let mut rng = SplitMix64::new(seed);
    let mut means = Vec::new();
    data.source_reserve(&mut means, resamples)?;
    if let Some(units) = units {
        let grouped = grouped_unit_values(values, units, data)?;
        let mut sample = Vec::new();
        for _ in 0..resamples {
            sample.clear();
            for _ in 0..grouped.len() {
                let unit = &grouped[rng.index(grouped.len(), data)?].1;
                data.source_reserve(&mut sample, unit.len())?;
                for _ in 0..unit.len() {
                    sample.push(unit[rng.index(unit.len(), data)?]);
                }
            }
            means.push(stable_mean(&sample, data)?);
        }
    } else {
        let mut sorted = Vec::new();
        data.source_reserve(&mut sorted, values.len())?;
        sorted.extend_from_slice(values);
        sort_numeric_source(&mut sorted, data)?;
        let mut sample = Vec::new();
        data.source_reserve(&mut sample, sorted.len())?;
        sample.resize(sorted.len(), 0.0);
        for _ in 0..resamples {
            for value in &mut sample {
                *value = sorted[rng.index(sorted.len(), data)?];
            }
            means.push(stable_mean(&sample, data)?);
        }
    }
    central_quantiles(&means, level, data)
}

fn grouped_unit_values<'a>(
    values: &[f64],
    units: &'a [String],
    data: Option<PlotData<'_>>,
) -> Result<Vec<(&'a str, Vec<f64>)>, Error> {
    let mut grouped: Vec<(&str, Vec<f64>)> = Vec::new();
    for (value, unit) in values.iter().copied().zip(units) {
        data.source_visits(grouped.len())?;
        if let Some((_, observations)) = grouped.iter_mut().find(|(label, _)| *label == unit) {
            data.source_reserve(observations, 1)?;
            observations.push(value);
        } else {
            data.source_reserve(&mut grouped, 1)?;
            let mut observations = Vec::new();
            data.source_reserve(&mut observations, 1)?;
            observations.push(value);
            grouped.push((unit, observations));
        }
    }
    sort_source_by(&mut grouped, data, |left, right| left.0.cmp(right.0))?;
    for (_, observations) in &mut grouped {
        sort_numeric_source(observations, data)?;
    }
    Ok(grouped)
}

fn stable_cell_seed(
    seed: u64,
    facets: &FacetIdentity,
    category: &str,
    series: Option<&str>,
) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    hash_bytes(&mut hash, &seed.to_le_bytes());
    hash_bytes(
        &mut hash,
        &u64::try_from(facets.0.len())
            .unwrap_or(u64::MAX)
            .to_le_bytes(),
    );
    for (name, level) in &facets.0 {
        hash_field(&mut hash, name);
        hash_field(&mut hash, level);
    }
    hash_field(&mut hash, category);
    if let Some(series) = series {
        hash_bytes(&mut hash, &[1]);
        hash_field(&mut hash, series);
    } else {
        hash_bytes(&mut hash, &[0]);
    }
    splitmix64(hash)
}

fn hash_field(hash: &mut u64, field: &str) {
    let length = u64::try_from(field.len()).unwrap_or(u64::MAX);
    hash_bytes(hash, &length.to_le_bytes());
    hash_bytes(hash, field.as_bytes());
}

fn hash_bytes(hash: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *hash ^= u64::from(*byte);
        *hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
}

fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next(&mut self) -> u64 {
        #[cfg(test)]
        BOOTSTRAP_DRAWS.with(|draws| draws.set(draws.get() + 1));
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut value = self.state;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^ (value >> 31)
    }

    fn index(&mut self, upper: usize, data: Option<PlotData<'_>>) -> Result<usize, Error> {
        let upper = u64::try_from(upper).unwrap_or(u64::MAX);
        let limit = u64::MAX - u64::MAX % upper;
        loop {
            data.source_visits(1)?;
            let value = self.next();
            if value < limit {
                return Ok(usize::try_from(value % upper).unwrap_or(0));
            }
        }
    }
}

fn value_range(bars: &[PreparedBar]) -> Result<(f64, f64), Error> {
    let mut range = (0.0_f64, 0.0_f64);
    for bar in bars {
        let (low, high) = bar.interval.unwrap_or((bar.end, bar.end));
        if [bar.start, bar.end, low, high]
            .iter()
            .any(|value| !value.is_finite())
        {
            return Err(Error::Data(
                "bar: derived statistics or geometry are not finite".into(),
            ));
        }
        range = (
            range.0.min(bar.start).min(low),
            range.1.max(bar.start).max(high),
        );
    }
    Ok(range)
}

fn stable_mean(values: &[f64], data: Option<PlotData<'_>>) -> Result<f64, Error> {
    if values.is_empty() {
        return Err(stat_error("cannot compute a mean from no observations"));
    }
    data.source_visits(values.len())?;
    let scale = values.iter().map(|value| value.abs()).fold(0.0, f64::max);
    if !scale.is_finite() {
        return Err(stat_error("contains non-finite observations"));
    }
    if scale == 0.0 {
        return Ok(0.0);
    }
    let mut sum = 0.0;
    let mut correction = 0.0;
    data.source_visits(values.len())?;
    for value in values {
        let normalized = value / scale;
        let next = sum + normalized;
        correction += if sum.abs() >= normalized.abs() {
            (sum - next) + normalized
        } else {
            (normalized - next) + sum
        };
        sum = next;
    }
    let mean = ((sum + correction) / values.len() as f64) * scale;
    if mean.is_finite() {
        Ok(mean)
    } else {
        Err(stat_error("mean is not numerically representable"))
    }
}

fn stable_sum(values: &[f64], data: Option<PlotData<'_>>) -> Result<f64, Error> {
    if values.is_empty() {
        return Err(stat_error("cannot compute a sum from no observations"));
    }
    data.source_visits(values.len())?;
    if values.iter().any(|value| !value.is_finite()) {
        return Err(stat_error("contains non-finite observations"));
    }
    if let Some(sum) = compensated_sum(values.iter().copied(), data)? {
        return Ok(sum);
    }

    data.source_visits(values.len())?;
    let scale = values.iter().map(|value| value.abs()).fold(0.0, f64::max);
    if scale == 0.0 {
        return Ok(0.0);
    }
    let sum = compensated_sum(values.iter().map(|value| value / scale), data)?
        .ok_or_else(|| stat_error("sum is not numerically representable"))?
        * scale;
    if sum.is_finite() {
        Ok(sum)
    } else {
        Err(stat_error("sum is not numerically representable"))
    }
}

fn compensated_sum(
    mut values: impl ExactSizeIterator<Item = f64>,
    data: Option<PlotData<'_>>,
) -> Result<Option<f64>, Error> {
    let mut sum: f64 = 0.0;
    let mut correction: f64 = 0.0;
    for _ in 0..values.len() {
        data.source_visits(1)?;
        let value = values.next().expect("exact sample length");
        let next = sum + value;
        if !next.is_finite() {
            return Ok(None);
        }
        correction += if sum.abs() >= value.abs() {
            (sum - next) + value
        } else {
            (value - next) + sum
        };
        if !correction.is_finite() {
            return Ok(None);
        }
        sum = next;
    }
    let result = sum + correction;
    Ok(result.is_finite().then_some(result))
}

#[cfg(test)]
mod tests {
    use super::{generated_interval, prepare, stable_cell_seed, BarMode, FacetIdentity};
    use crate::spec::{
        Bootstrap, ConfidenceInterval, ConfidenceIntervalAlgorithm, ErrorBarKind,
        PercentileInterval, StandardDeviation, StandardError,
    };
    use crate::{BarInput, Bounds, CategoryValues, ErrorBar, NumericValues, Plot};

    #[test]
    fn custom_bar_fill_and_outline_match_selected_marks_and_originating_guide() {
        let source = Plot::horizontal_bar(
            CategoryValues::new("category", ["a", "a", "b", "b"]).unwrap(),
            BarInput::samples(NumericValues::new("value", [-1.0, 2.0, 3.0, 4.0])),
        )
        .series(CategoryValues::new("series", ["first", "second", "first", "second"]).unwrap())
        .zero_line()
        .erase();
        let resources = crate::font::canonical::OperationGenerationResources::new();
        let inputs = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
        let chart = inputs.attach(source).unwrap();
        let panel = crate::layout::PanelBox::new(10.0, 900.0, 20.0, 600.0);
        let mut theme = crate::Theme::Publication.resolve();
        let original = chart
            .measure_selected_interior(crate::spec::PlotRenderContext::new(
                chart.data(),
                panel,
                &theme,
            ))
            .unwrap();
        theme.marks.bar_body.fill_alpha = 128;
        theme.marks.bar_body.outline_paint =
            crate::theme::FillOutlinePaint::Fixed(crate::render::backend::Rgba::new(1, 3, 5, 64));
        theme.marks.bar_body.outline_width = 2.0;
        theme.marks.bar_zero.paint = crate::render::backend::Rgba::new(2, 4, 6, 188);
        theme.marks.bar_zero.width = 1.25;
        let selected = chart
            .measure_selected_interior(crate::spec::PlotRenderContext::new(
                chart.data(),
                panel,
                &theme,
            ))
            .unwrap();
        let extract = |result: &crate::spec::PlotRenderResult| {
            let crate::layout::interior::MarkPayload::Bars(payload) =
                &result.marks.as_ref().unwrap().payload
            else {
                panic!("selected ordinary bar payload")
            };
            (payload.bars[0].polygon, payload.bars[0].fill.clone())
        };
        let (old_polygon, old_style) = extract(&original);
        let (new_polygon, new_style) = extract(&selected);
        assert_eq!(old_polygon, new_polygon);
        assert_eq!(old_style.color.alpha(), 255);
        assert_eq!(new_style.color.alpha(), 128);
        assert_eq!(new_style.stroke.alpha(), 64);
        assert_eq!(new_style.stroke_width.to_bits(), 2.0_f32.to_bits());
        let zero = |result: &crate::spec::PlotRenderResult| {
            let crate::layout::interior::MarkPayload::Bars(payload) =
                &result.marks.as_ref().unwrap().payload
            else {
                panic!("selected ordinary bar payload")
            };
            (payload.zero, payload.zero_style.clone())
        };
        let (old_zero, old_zero_style) = zero(&original);
        let (new_zero, new_zero_style) = zero(&selected);
        assert_eq!(old_zero, new_zero);
        assert!(new_zero.is_some());
        assert_ne!(old_zero_style, new_zero_style);
        assert_eq!(new_zero_style.color.alpha(), 188);
        assert_eq!(new_zero_style.width.to_bits(), 1.25_f32.to_bits());
        assert!(crate::decorations::legend::merge_exact_entries(
            [
                original.legend_entries[0].clone(),
                selected.legend_entries[0].clone()
            ],
            "custom bar fill",
            &crate::font::canonical::OperationGenerationResources::new(),
        )
        .unwrap_err()
        .to_string()
        .contains("glyphs must match exactly"));
    }

    fn stable_mean(values: &[f64]) -> Result<f64, crate::Error> {
        super::stable_mean(values, None)
    }
    fn stable_sum(values: &[f64]) -> Result<f64, crate::Error> {
        super::stable_sum(values, None)
    }

    #[test]
    fn admitted_bootstrap_matches_pre_admission_numeric_baselines() {
        let values = [-10.0, -0.0, 0.0, 1.0, 7.0, 13.0];
        let units = ["z", "a", "z", "b", "a", "a"].map(str::to_owned);
        let chart = Plot::bar(
            CategoryValues::new("category", ["a"]).unwrap(),
            BarInput::samples(NumericValues::new("value", [1.0])),
        )
        .erase();
        // Measured from the pre-admission stable-sort/per-resample allocation
        // algorithm, including unequal unit sizes and signed zero observations.
        for (units, expected) in [
            (
                None,
                [13_836_709_375_145_532_896, 4_618_141_177_893_285_616],
            ),
            (
                Some(units.as_slice()),
                [13_840_124_604_862_955_520, 4_619_138_403_525_060_512],
            ),
        ] {
            let resources = crate::font::canonical::OperationGenerationResources::new();
            let (lo, hi) = super::percentile_bootstrap(
                &values,
                units,
                0.8,
                137,
                42,
                Some(chart.data().with_axis_resources(&resources)),
            )
            .unwrap();
            assert_eq!([lo.to_bits(), hi.to_bits()], expected);
        }
    }

    #[test]
    fn bar_helpers_refuse_before_cell_storage_quantiles_resampling_and_order_mutation() {
        use crate::font::canonical::OperationGenerationResources;
        let chart = Plot::bar(
            CategoryValues::new("category", ["a", "b"]).unwrap(),
            BarInput::samples(NumericValues::new("value", [1.0, 2.0])),
        )
        .erase();
        let before_values =
            std::mem::size_of::<String>() + std::mem::size_of::<super::SingleBarCell>();
        let resources = OperationGenerationResources::with_test_limits(
            1000,
            u64::try_from(before_values).unwrap(),
        );
        let data = chart.data().with_axis_resources(&resources);
        let error = super::prepare_single(
            vec!["a".to_owned()],
            None,
            vec![1.0],
            Vec::new(),
            super::BarSummaryInput {
                units: None,
                input: chart.bar_input().unwrap(),
                facet_identity: &FacetIdentity(Vec::new()),
                data,
            },
        )
        .unwrap_err();
        assert!(error.to_string().contains("storage admission"), "{error}");
        let resources = OperationGenerationResources::with_test_limits(1000, 0);
        let data = Some(chart.data().with_axis_resources(&resources));
        assert!(super::central_quantiles(&[1.0, 2.0], 0.95, data)
            .unwrap_err()
            .to_string()
            .contains("storage admission"));
        for units in [None, Some(["a".to_owned(), "b".to_owned()])] {
            super::BOOTSTRAP_DRAWS.with(|draws| draws.set(0));
            let resources = OperationGenerationResources::with_test_limits(1000, 8 * 10);
            let data = Some(chart.data().with_axis_resources(&resources));
            let error = super::percentile_bootstrap(
                &[1.0, 2.0],
                units.as_ref().map(<[_; 2]>::as_slice),
                0.95,
                10,
                0,
                data,
            )
            .unwrap_err();
            assert!(error.to_string().contains("storage admission"), "{error}");
            assert_eq!(super::BOOTSTRAP_DRAWS.with(std::cell::Cell::get), 0);
        }
        let resources = OperationGenerationResources::with_test_limits(0, 1000);
        let data = chart.data().with_axis_resources(&resources);
        let mut rng = super::SplitMix64::new(42);
        assert!(rng.index(2, Some(data)).is_err());
        assert_eq!(rng.state, 42);
        let mut prepared = prepare(&chart, chart.data(), BarMode::Single).unwrap();
        let original = prepared.categories.clone();
        for order in [
            crate::CategoryOrder::Explicit(vec!["b".into(), "a".into()]),
            crate::CategoryOrder::Value(crate::ValueSort::Descending),
        ] {
            assert!(super::apply_category_order(
                &mut prepared,
                &order,
                BarMode::Single,
                false,
                data
            )
            .is_err());
            assert_eq!(prepared.categories, original);
        }
    }

    #[test]
    fn omitted_intervals_do_not_resample() {
        let chart = Plot::bar(
            CategoryValues::new("category", ["a", "a", "b", "b"]).unwrap(),
            BarInput::samples(NumericValues::new("value", [1.0, 3.0, 5.0, 7.0]))
                .error_bar(ErrorBar(ErrorBarKind::Hidden)),
        )
        .category_order(crate::ValueSort::Descending)
        .y_limits(-10.0, 10.0)
        .point_layer(
            crate::BarPointLayer::new(
                CategoryValues::new("category", ["a"]).unwrap(),
                None,
                NumericValues::new("value", [100.0]),
                (1, 2, 3),
                3,
                0.5,
            )
            .unwrap(),
        )
        .erase();
        super::BOOTSTRAP_DRAWS.with(|draws| draws.set(0));
        let resources =
            crate::font::canonical::OperationGenerationResources::with_test_limits(1000, 4096);
        assert_eq!(
            super::prepare(
                &chart,
                chart.data().with_axis_resources(&resources),
                BarMode::Single,
            )
            .unwrap()
            .categories,
            vec!["b".to_owned(), "a".to_owned()]
        );
        let resources =
            crate::font::canonical::OperationGenerationResources::with_test_limits(0, 0);
        let data = Some(chart.data().with_axis_resources(&resources));
        for (values, kind) in [
            (&[1.0, 2.0][..], ErrorBarKind::Hidden),
            (&[1.0][..], bootstrap(1000, 0)),
        ] {
            assert!(generated_interval(
                values,
                None,
                1.0,
                &kind,
                &FacetIdentity(Vec::new()),
                "a",
                None,
                data
            )
            .unwrap()
            .is_none());
        }
        let resources =
            crate::font::canonical::OperationGenerationResources::with_test_limits(2, 0);
        let units = ["same".to_owned(), "same".to_owned()];
        assert!(generated_interval(
            &[1.0, 2.0],
            Some(&units),
            1.5,
            &bootstrap(1000, 0),
            &FacetIdentity(Vec::new()),
            "a",
            None,
            Some(chart.data().with_axis_resources(&resources))
        )
        .unwrap()
        .is_none());
        assert_eq!(super::BOOTSTRAP_DRAWS.with(std::cell::Cell::get), 0);
    }

    #[test]
    fn bar_axis_sources_preserve_intervals_stacks_facets_bounds_and_point_extents() {
        let compare = |chart: &Plot, data: crate::data::PlotData<'_>, mode| {
            let expected = prepare(chart, data, mode).unwrap();
            let resources = crate::font::canonical::OperationGenerationResources::new();
            let actual = super::prepare(chart, data.with_axis_resources(&resources), mode).unwrap();
            assert_eq!(
                (actual.categories, actual.value_range),
                (expected.categories, expected.value_range)
            );
        };
        let categories =
            CategoryValues::new("category", ["b", "a", "b", "a", "b", "a", "b", "a"]).unwrap();
        let values = NumericValues::new("value", [-4.0, 1.0, 6.0, 3.0, -2.0, 7.0, 8.0, 9.0]);
        let groups = CategoryValues::new(
            "series",
            [
                "left", "left", "left", "left", "right", "right", "right", "right",
            ],
        )
        .unwrap();
        let variants = [
            ErrorBarKind::Hidden,
            ErrorBarKind::StandardDeviation(StandardDeviation { scale: 2.0 }),
            ErrorBarKind::StandardError(StandardError { scale: 1.5 }),
            ErrorBarKind::PercentileInterval(PercentileInterval { level: 0.8 }),
            bootstrap(137, 42),
            ErrorBarKind::ConfidenceInterval(ConfidenceInterval {
                level: 0.95,
                algorithm: ConfidenceIntervalAlgorithm::StudentT,
            }),
        ];
        for kind in variants {
            let input = BarInput::samples(values.clone()).error_bar(ErrorBar(kind));
            let chart = Plot::bar(categories.clone(), input.clone())
                .category_order(crate::ValueSort::Descending)
                .erase();
            compare(&chart, chart.data(), BarMode::Single);
            let chart = Plot::horizontal_bar(categories.clone(), input.clone())
                .series(groups.clone())
                .category_order(crate::CategoryOrder::explicit(["a", "b"]))
                .erase();
            compare(&chart, chart.data(), BarMode::Grouped);
            let chart = Plot::bar(categories.clone(), input)
                .facet(crate::FacetSpec::column(groups.clone()))
                .erase();
            for rows in [[0, 1, 2, 3], [4, 5, 6, 7]] {
                compare(
                    &chart,
                    crate::data::PlotData::with_rows(&chart, &rows, &rows),
                    BarMode::Single,
                );
            }
        }
        let chart = Plot::stacked_bar(categories, values, groups).erase();
        compare(&chart, chart.data(), BarMode::Stacked);
        let chart = Plot::bar(
            CategoryValues::new("category", ["a", "b"]).unwrap(),
            BarInput::estimates_with_error_bars(
                NumericValues::new("value", [2.0, 5.0]),
                Bounds::new(
                    NumericValues::new("lower", [-2.0, 1.0]),
                    NumericValues::new("upper", [4.0, 9.0]),
                ),
            ),
        )
        .point_layer(
            crate::BarPointLayer::new(
                CategoryValues::new("category", ["a", "b"]).unwrap(),
                None,
                NumericValues::new("value", [-50.0, 100.0]),
                (1, 2, 3),
                3,
                0.5,
            )
            .unwrap(),
        )
        .erase();
        compare(&chart, chart.data(), BarMode::Single);
        let resources = crate::font::canonical::OperationGenerationResources::new();
        super::prepare_core(
            &chart,
            chart.data().with_axis_resources(&resources),
            BarMode::Single,
            None,
        )
        .unwrap();
        let before_points = resources.usage().unwrap().1;
        let resources = crate::font::canonical::OperationGenerationResources::with_test_limits(
            before_points,
            u64::MAX,
        );
        assert!(super::prepare(
            &chart,
            chart.data().with_axis_resources(&resources),
            BarMode::Single,
        )
        .unwrap_err()
        .to_string()
        .contains("work admission"));
    }

    fn bootstrap(resamples: usize, seed: u64) -> ErrorBarKind {
        ErrorBarKind::ConfidenceInterval(ConfidenceInterval {
            level: 0.95,
            algorithm: ConfidenceIntervalAlgorithm::Bootstrap(Bootstrap {
                resamples,
                seed,
                units: None,
            }),
        })
    }

    fn interval(
        values: &[f64],
        units: Option<&[String]>,
        error_bar: &ErrorBarKind,
    ) -> Option<(f64, f64)> {
        generated_interval(
            values,
            units,
            stable_mean(values).unwrap(),
            error_bar,
            &FacetIdentity(vec![("facet".to_owned(), "one".to_owned())]),
            "category",
            Some("group"),
            None,
        )
        .unwrap()
    }

    #[test]
    fn percentile_bootstrap_is_deterministic_for_a_stable_cell() {
        let values = [1.0, 2.0, 4.0, 8.0, 16.0];
        let spec = bootstrap(1_000, 0);

        assert_eq!(
            interval(&values, None, &spec),
            interval(&values, None, &spec)
        );
    }

    #[test]
    fn percentile_bootstrap_custom_resamples_and_seed_change_the_interval() {
        let values = [1.0, 2.0, 4.0, 8.0, 16.0];

        assert_ne!(
            interval(&values, None, &bootstrap(37, 9)),
            interval(&values, None, &bootstrap(73, 12))
        );
    }

    #[test]
    fn bootstrap_stream_identity_distinguishes_facets_categories_and_optional_series() {
        let left = FacetIdentity(vec![("panel".to_owned(), "left".to_owned())]);
        let right = FacetIdentity(vec![("panel".to_owned(), "right".to_owned())]);
        let seeds = [
            stable_cell_seed(0, &left, "a", None),
            stable_cell_seed(0, &right, "a", None),
            stable_cell_seed(0, &left, "b", None),
            stable_cell_seed(0, &left, "a", Some("")),
            stable_cell_seed(0, &left, "a", Some("group")),
        ];

        for (index, seed) in seeds.iter().enumerate() {
            assert!(!seeds[..index].contains(seed));
        }
    }

    #[test]
    fn student_t_confidence_interval_matches_small_sample_oracle() {
        let values = [1.0, 3.0];
        let spec = ErrorBarKind::ConfidenceInterval(ConfidenceInterval {
            level: 0.95,
            algorithm: ConfidenceIntervalAlgorithm::StudentT,
        });
        let (lower, upper) = interval(&values, None, &spec).unwrap();

        assert!((lower - (2.0 - 12.706_204_736_432_095)).abs() < 1e-9);
        assert!((upper - (2.0 + 12.706_204_736_432_095)).abs() < 1e-9);
    }

    #[test]
    fn singleton_sample_omits_every_generated_error_bar() {
        let variants = [
            ErrorBarKind::Hidden,
            ErrorBarKind::StandardDeviation(StandardDeviation { scale: 1.0 }),
            ErrorBarKind::PercentileInterval(PercentileInterval { level: 0.95 }),
            ErrorBarKind::StandardError(StandardError { scale: 1.0 }),
            bootstrap(100, 0),
        ];

        assert!(variants
            .iter()
            .all(|spec| interval(&[3.5], None, spec).is_none()));
    }

    #[test]
    fn generated_error_bar_taxonomy_has_distinct_numeric_meanings() {
        let values = [1.0, 2.0, 7.0, 10.0];
        let intervals = [
            ErrorBarKind::StandardDeviation(StandardDeviation { scale: 1.0 }),
            ErrorBarKind::PercentileInterval(PercentileInterval { level: 0.8 }),
            ErrorBarKind::StandardError(StandardError { scale: 1.0 }),
            bootstrap(200, 0),
        ]
        .map(|spec| interval(&values, None, &spec).unwrap());

        assert!(intervals.windows(2).all(|pair| pair[0] != pair[1]));
    }

    #[test]
    fn multilevel_bootstrap_resamples_units_and_within_unit_observations() {
        let values = [0.0, 2.0, 10.0, 14.0];
        let units = ["a", "a", "b", "b"].map(str::to_owned);
        let spec = bootstrap(500, 4);

        assert_ne!(
            interval(&values, Some(&units), &spec),
            interval(&values, None, &spec)
        );
    }

    #[test]
    fn one_effective_sampling_unit_omits_bootstrap_interval() {
        let units = ["same", "same", "same"].map(str::to_owned);

        assert!(interval(&[1.0, 2.0, 3.0], Some(&units), &bootstrap(100, 0)).is_none());
    }

    #[test]
    fn positioned_series_sample_cells_compute_independent_means() {
        let chart = Plot::bar(
            CategoryValues::new("category", ["a", "a", "a", "a"]).unwrap(),
            BarInput::samples(NumericValues::new("value", [1.0, 3.0, 10.0, 30.0]))
                .error_bar(ErrorBar::hidden()),
        )
        .series(CategoryValues::new("series", ["left", "left", "right", "right"]).unwrap())
        .erase();

        let prepared = prepare(&chart, chart.data(), BarMode::Grouped).unwrap();
        let mut means = prepared.bars.iter().map(|bar| bar.end).collect::<Vec<_>>();
        means.sort_by(f64::total_cmp);

        assert_eq!(means, [2.0, 20.0]);
    }

    #[test]
    fn faceted_sample_cells_compute_independent_means() {
        let chart = Plot::bar(
            CategoryValues::new("category", ["a", "a", "a", "a"]).unwrap(),
            BarInput::samples(NumericValues::new("value", [1.0, 3.0, 10.0, 30.0]))
                .error_bar(ErrorBar::hidden()),
        )
        .facet(crate::FacetSpec::column(
            CategoryValues::new("facet", ["left", "left", "right", "right"]).unwrap(),
        ))
        .erase();
        let left_rows = [0, 1];
        let right_rows = [2, 3];
        let left = prepare(
            &chart,
            crate::data::PlotData::with_rows(&chart, &left_rows, &left_rows),
            BarMode::Single,
        )
        .unwrap();
        let right = prepare(
            &chart,
            crate::data::PlotData::with_rows(&chart, &right_rows, &right_rows),
            BarMode::Single,
        )
        .unwrap();

        assert!((left.bars[0].end - 2.0).abs() < f64::EPSILON);
        assert!((right.bars[0].end - 20.0).abs() < f64::EPSILON);
    }

    #[test]
    fn precomputed_bounds_are_retained_exactly() {
        let chart = Plot::bar(
            CategoryValues::new("category", ["a", "b"]).unwrap(),
            BarInput::estimates_with_error_bars(
                NumericValues::new("estimate", [2.0, 5.0]),
                Bounds::new(
                    NumericValues::new("lower", [1.0, 3.5]),
                    NumericValues::new("upper", [4.0, 8.0]),
                ),
            ),
        )
        .erase();

        let prepared = prepare(&chart, chart.data(), BarMode::Single).unwrap();
        let intervals = prepared
            .bars
            .iter()
            .map(|bar| bar.interval.unwrap())
            .collect::<Vec<_>>();

        assert_eq!(intervals, [(1.0, 4.0), (3.5, 8.0)]);
    }

    #[test]
    fn configured_sampling_units_are_scoped_to_each_positioned_series() {
        let units = CategoryValues::new("participant", ["p1", "p1", "p2", "p2"]).unwrap();
        let bootstrap = Bootstrap::new().resamples(50).unwrap().seed(7).units(units);
        let chart = Plot::bar(
            CategoryValues::new("category", ["a", "a", "a", "a"]).unwrap(),
            BarInput::samples(NumericValues::new("value", [1.0, 3.0, 10.0, 30.0]))
                .error_bar(ErrorBar::confidence_interval().bootstrap(bootstrap)),
        )
        .series(CategoryValues::new("series", ["left", "left", "right", "right"]).unwrap())
        .erase();

        let prepared = prepare(&chart, chart.data(), BarMode::Grouped).unwrap();

        assert!(prepared.bars.iter().all(|bar| bar.interval.is_none()));
    }

    #[test]
    fn stable_mean_avoids_overflow_for_repeated_maximum_values() {
        assert_eq!(
            stable_mean(&[f64::MAX, f64::MAX]).unwrap().to_bits(),
            f64::MAX.to_bits()
        );
    }

    #[test]
    fn stable_sum_preserves_small_remainder_after_extreme_cancellation() {
        let sum = stable_sum(&[f64::MAX, -f64::MAX, 5e-16]).unwrap();

        assert_eq!(sum.to_bits(), 5e-16_f64.to_bits());
    }

    #[test]
    fn stable_sum_scales_when_an_intermediate_sum_overflows() {
        let sum = stable_sum(&[f64::MAX, f64::MAX, -f64::MAX]).unwrap();

        assert_eq!(sum.to_bits(), f64::MAX.to_bits());
    }
}
