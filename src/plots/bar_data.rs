//! Shared data preparation for vertical and horizontal bar plots.

#![allow(clippy::cast_precision_loss)]

use crate::Error;

use crate::decorations::legend::LegendEntry;
use crate::render::backend::{FillStyle, PointStyle};
use crate::spec::{
    BarInput, BarSource, ConfidenceIntervalAlgorithm, ErrorBarKind, PercentileInterval,
    StandardDeviation, StandardError,
};
use crate::{CategoryOrder, Plot, ValueSort};

/// Bar arrangement mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BarMode {
    /// One aggregate bar per x category.
    Single,
    /// One bar per x category and graphical group, dodged side-by-side.
    Grouped,
    /// One additive segment per x category and graphical group.
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
    /// Graphical-group labels in observed order. Empty for `Single` mode.
    pub(crate) series: Vec<String>,
    /// Hue labels in observed order. These alone own color and legend identity.
    pub(crate) hues: Vec<String>,
    /// Prepared bars/segments.
    pub(crate) bars: Vec<PreparedBar>,
    /// Numeric value range including zero and CI/stack extents.
    pub(crate) value_range: (f64, f64),
    /// Supplemental points at resolved category/group centers.
    pub(crate) points: Vec<PreparedBarPoint>,
    /// Significance labels attached to exact bar/category anchors.
    pub(crate) significance: Vec<PreparedBarSignificance>,
}

#[derive(Clone, Copy)]
struct BarSummaryInput<'a> {
    units: Option<&'a [String]>,
    input: &'a BarInput,
    facet_identity: &'a FacetIdentity,
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

/// Build prepared bar data from x/y and optional graphical group and hue channels.
pub(crate) fn prepare<'a>(
    chart: &Plot,
    data: impl Into<crate::data::PlotData<'a>>,
    mode: BarMode,
) -> Result<PreparedBars, Error> {
    let data = data.into();
    let x_col = chart
        .x_col()
        .ok_or_else(|| Error::Data("bar: x column not set".into()))?;
    let y_col = chart
        .y_col()
        .ok_or_else(|| Error::Data("bar: y column not set".into()))?;

    let categories = crate::data::strict_strings(data, x_col, "bar")?;
    let values = crate::data::strict_finite_f64(data, y_col, "bar")?;
    if categories.len() != values.len() {
        return Err(Error::Data("bar: x/y length mismatch".into()));
    }

    let channels = crate::data::collective_channels(
        data,
        chart.group_col(),
        chart.hue_col(),
        Some(&categories),
        "bar",
    )?;
    let group_order = chart.resolved_group_levels(channels.group_levels.clone());
    let hue_order = chart.resolved_hue_levels(channels.hue_levels.clone());
    let units = chart
        .bar_units_col()
        .map(|column| crate::data::strict_strings(data, column, "bar sampling units"))
        .transpose()?;
    let facet_identity = facet_identity(chart, data)?;
    let mut prepared = match mode {
        BarMode::Single => prepare_single(
            categories,
            channels.hues.as_deref(),
            values,
            hue_order,
            BarSummaryInput {
                units: units.as_deref(),
                input: chart
                    .bar_input()
                    .ok_or_else(|| Error::Data("ordinary bar input is not configured".into()))?,
                facet_identity: &facet_identity,
            },
        ),
        BarMode::Grouped => {
            let groups = channels.groups.ok_or_else(|| {
                Error::Data("clustered bar requires a graphical group column".into())
            })?;
            prepare_grouped(
                categories,
                groups,
                channels.hues.as_deref(),
                values,
                group_order,
                hue_order,
                BarSummaryInput {
                    units: units.as_deref(),
                    input: chart.bar_input().ok_or_else(|| {
                        Error::Data("ordinary bar input is not configured".into())
                    })?,
                    facet_identity: &facet_identity,
                },
            )
        }
        BarMode::Stacked => {
            let groups = channels.groups.ok_or_else(|| {
                Error::Data("stacked bar requires a graphical group column".into())
            })?;
            prepare_stacked(
                categories,
                groups,
                channels.hues.as_deref(),
                values,
                group_order,
                hue_order,
            )
        }
    }?;
    apply_category_order(
        &mut prepared,
        chart.category_order_spec(),
        mode,
        !data.is_full_plot(),
    )?;
    if let Some(bounds) = chart.bar_bound_cols() {
        apply_precomputed_bounds(chart, data, &mut prepared, bounds)?;
    }
    apply_attached_significance(chart, &mut prepared, mode)?;
    prepared.points = prepare_point_layers(chart, &prepared, mode)?;
    for point in &prepared.points {
        prepared.value_range.0 = prepared.value_range.0.min(point.value);
        prepared.value_range.1 = prepared.value_range.1.max(point.value);
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
        if values.iter().any(|value| value != level) {
            return Err(Error::Data(
                "bar facet preparation mixed multiple levels in one panel".into(),
            ));
        }
        let name = data.column(facet_column)?.name();
        identity.push((name.to_owned(), level.clone()));
    }
    Ok(FacetIdentity(identity))
}

fn apply_attached_significance(
    chart: &Plot,
    prepared: &mut PreparedBars,
    mode: BarMode,
) -> Result<(), Error> {
    for annotation in chart.significance_annotations() {
        let Some(target) = annotation.at_target() else {
            continue;
        };
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
        prepared.significance.push(PreparedBarSignificance {
            category_index,
            series_index,
            edge,
            positive,
            label: annotation.mark().glyph()?.to_owned(),
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
                            "bar estimate bounds graphical group disappeared during preparation"
                                .into(),
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
    prepared.value_range = value_range(&prepared.bars)?;
    Ok(())
}

fn prepare_point_layers(
    chart: &Plot,
    prepared: &PreparedBars,
    mode: BarMode,
) -> Result<Vec<PreparedBarPoint>, Error> {
    if chart.bar_point_layers().is_empty() {
        return Ok(Vec::new());
    }
    if mode == BarMode::Stacked {
        return Err(Error::Data(
            "bar point layers are not supported by stacked bars".into(),
        ));
    }
    let grouped = chart.group_col().is_some();
    let mut points = Vec::new();
    for (layer_index, layer) in chart.bar_point_layers().iter().enumerate() {
        let context = format!("bar point layer {layer_index}");
        if layer.group_values().is_some() != grouped {
            return Err(Error::Data(format!(
                "{context}: layer graphical-group configuration must match the bar chart"
            )));
        }
        let categories = crate::data::strict_category_values(layer.category_values(), &context)?;
        let groups = layer
            .group_values()
            .map(|values| crate::data::strict_category_values(values, &context))
            .transpose()?;
        let values = crate::data::strict_numeric_values(layer.values(), &context)?;
        if categories.is_empty() || categories.len() != values.len() {
            return Err(Error::Data(format!(
                "{context}: category and value columns must have equal non-zero lengths"
            )));
        }
        for row in 0..values.len() {
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
            let series_index = groups
                .as_ref()
                .map(|group_values| {
                    prepared
                        .series
                        .iter()
                        .position(|value| value == &group_values[row])
                        .ok_or_else(|| {
                            Error::Data(format!(
                                "{context}: unknown graphical group {:?} at row {row}",
                                group_values[row]
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
                    "{context}: category/group at row {row} does not resolve to a prepared bar cell"
                )));
            }
            points.push(PreparedBarPoint {
                category_index,
                series_index,
                value: values[row],
                style: layer.point_style(),
            });
        }
    }
    Ok(points)
}

/// Validate cardinality and strict values for typed ordinary-bar inputs.
#[expect(
    clippy::too_many_lines,
    reason = "one pass preserves source-row, facet, category, and graphical-group validation context"
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
            "bar: category/value/hue/facet length mismatch".into(),
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
        if let Some(group_name) = chart.group_name() {
            labels.push(format!(
                "graphical group {group_name}={}",
                key[category_index + 1]
            ));
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
) -> Result<(), Error> {
    match order {
        CategoryOrder::Observed => Ok(()),
        CategoryOrder::Explicit(requested) => {
            let ordered = crate::data::explicit_category_order(
                &prepared.categories,
                requested,
                "bar category order",
                is_subset,
            )?;
            let old_categories = std::mem::replace(&mut prepared.categories, ordered);
            for bar in &mut prepared.bars {
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
            sort_single_values(prepared, *sort);
            Ok(())
        }
        CategoryOrder::Value(_) => Err(Error::Data(
            "bar: value-derived category order requires one ungrouped bar per category; use an explicit category order for grouped or stacked bars"
                .into(),
        )),
    }
}

fn sort_single_values(prepared: &mut PreparedBars, sort: ValueSort) {
    let mut order: Vec<usize> = (0..prepared.categories.len()).collect();
    order.sort_by(|left, right| {
        let left_value = prepared.bars[*left].end;
        let right_value = prepared.bars[*right].end;
        match sort {
            ValueSort::Ascending => left_value.total_cmp(&right_value),
            ValueSort::Descending => right_value.total_cmp(&left_value),
        }
    });
    let categories = order
        .iter()
        .map(|index| prepared.categories[*index].clone())
        .collect();
    let bars = order
        .iter()
        .enumerate()
        .map(|(category_index, old_index)| {
            let mut bar = prepared.bars[*old_index].clone();
            bar.category_index = category_index;
            bar
        })
        .collect();
    prepared.categories = categories;
    prepared.bars = bars;
}

/// Legend entries for hue series.
pub(crate) fn legend_entries(prepared: &PreparedBars, chart: &Plot) -> Vec<LegendEntry> {
    let palette = chart.palette();
    prepared
        .hues
        .iter()
        .enumerate()
        .map(|(index, label)| {
            LegendEntry::fill(
                chart.legend_identity(label.clone()),
                FillStyle {
                    color: palette.colors[index % palette.colors.len()],
                    stroke: palette.foreground,
                    stroke_width: 1.0,
                    opacity: 1.0,
                },
            )
        })
        .collect()
}

fn prepare_single(
    categories: Vec<String>,
    hues: Option<&[String]>,
    values: Vec<f64>,
    mut hue_levels: Vec<String>,
    summary: BarSummaryInput<'_>,
) -> Result<PreparedBars, Error> {
    let mut category_levels = Vec::new();
    let mut groups: Vec<SingleBarCell> = Vec::new();
    for (row, (category, value)) in categories.into_iter().zip(values).enumerate() {
        let category_index = level_index(&mut category_levels, category);
        let hue_index = hues.map(|hues| level_index(&mut hue_levels, hues[row].clone()));
        if let Some(cell) = groups
            .iter_mut()
            .find(|cell| cell.category_index == category_index)
        {
            cell.values.push(value);
            if let Some(units) = summary.units {
                cell.units.push(units[row].clone());
            }
        } else {
            groups.push(SingleBarCell {
                category_index,
                hue_index,
                values: vec![value],
                units: summary
                    .units
                    .map_or_else(Vec::new, |units| vec![units[row].clone()]),
            });
        }
    }
    if groups.is_empty() {
        return Err(Error::Data("bar: no groups".into()));
    }

    let mut bars = Vec::with_capacity(groups.len());
    for cell in &groups {
        let category = &category_levels[cell.category_index];
        let (value, interval) = summarize(
            &cell.values,
            summary.units.map(|_| cell.units.as_slice()),
            &format!("category {category:?}"),
            summary.facet_identity,
            category,
            None,
            summary.input,
        )?;
        bars.push(PreparedBar {
            category_index: cell.category_index,
            series_index: None,
            hue_index: cell.hue_index,
            start: 0.0,
            end: value,
            interval,
        });
    }
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
    if categories.len() != series.len() || categories.len() != values.len() {
        return Err(Error::Data("bar: x/y/hue length mismatch".into()));
    }
    let mut category_levels = Vec::new();
    let mut series_levels = series_order;
    let mut hue_levels = hue_order;
    let mut groups: Vec<GroupedBarCell> = Vec::new();

    for (row, ((category, series_label), value)) in
        categories.into_iter().zip(series).zip(values).enumerate()
    {
        let category_index = level_index(&mut category_levels, category);
        let series_index = level_index(&mut series_levels, series_label);
        let hue_index = hues
            .as_ref()
            .map(|hues| level_index(&mut hue_levels, hues[row].clone()));
        if let Some(cell) = groups
            .iter_mut()
            .find(|cell| cell.category_index == category_index && cell.series_index == series_index)
        {
            cell.values.push(value);
            if let Some(units) = summary.units {
                cell.units.push(units[row].clone());
            }
        } else {
            groups.push(GroupedBarCell {
                category_index,
                series_index,
                hue_index,
                values: vec![value],
                units: summary
                    .units
                    .map_or_else(Vec::new, |units| vec![units[row].clone()]),
            });
        }
    }
    if groups.is_empty() {
        return Err(Error::Data("bar: no groups".into()));
    }

    let mut bars = Vec::with_capacity(groups.len());
    for cell in groups {
        let context = format!(
            "category {:?}, graphical group {:?}",
            category_levels[cell.category_index], series_levels[cell.series_index]
        );
        let (value, interval) = summarize(
            &cell.values,
            summary.units.map(|_| cell.units.as_slice()),
            &context,
            summary.facet_identity,
            &category_levels[cell.category_index],
            Some(&series_levels[cell.series_index]),
            summary.input,
        )?;
        bars.push(PreparedBar {
            category_index: cell.category_index,
            series_index: Some(cell.series_index),
            hue_index: cell.hue_index,
            start: 0.0,
            end: value,
            interval,
        });
    }
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

fn prepare_stacked(
    categories: Vec<String>,
    series: Vec<String>,
    hues: Option<&[String]>,
    values: Vec<f64>,
    series_order: Vec<String>,
    hue_order: Vec<String>,
) -> Result<PreparedBars, Error> {
    if categories.len() != series.len() || categories.len() != values.len() {
        return Err(Error::Data("bar: x/y/hue length mismatch".into()));
    }
    let mut category_levels = Vec::new();
    let mut series_levels = series_order;
    let mut hue_levels = hue_order;
    let mut groups: Vec<(usize, usize, Option<usize>, Vec<f64>)> = Vec::new();

    for (row, ((category, series_label), value)) in
        categories.into_iter().zip(series).zip(values).enumerate()
    {
        let category_index = level_index(&mut category_levels, category);
        let series_index = level_index(&mut series_levels, series_label);
        let hue_index = hues
            .as_ref()
            .map(|hues| level_index(&mut hue_levels, hues[row].clone()));
        if let Some((_, _, _, values)) = groups
            .iter_mut()
            .find(|(c, s, _, _)| *c == category_index && *s == series_index)
        {
            values.push(value);
        } else {
            groups.push((category_index, series_index, hue_index, vec![value]));
        }
    }
    if groups.is_empty() {
        return Err(Error::Data("bar: no groups".into()));
    }
    let sums = groups
        .into_iter()
        .map(|(category_index, series_index, hue_index, values)| {
            let context = format!(
                "stacked category {:?}, graphical group {:?}",
                category_levels[category_index], series_levels[series_index]
            );
            stable_sum(&values).map_or_else(
                |reason| Err(Error::Data(format!("bar: {context} {reason}"))),
                |sum| Ok((category_index, series_index, hue_index, sum)),
            )
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut positive_offsets = vec![0.0; category_levels.len()];
    let mut negative_offsets = vec![0.0; category_levels.len()];
    let mut bars = Vec::with_capacity(sums.len());
    for series_index in 0..series_levels.len() {
        for category_index in 0..category_levels.len() {
            let Some((_, _, hue_index, value)) = sums
                .iter()
                .find(|(c, s, _, _)| *c == category_index && *s == series_index)
            else {
                continue;
            };
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
                interval: None,
            });
        }
    }

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

fn level_index(levels: &mut Vec<String>, label: String) -> usize {
    if let Some(index) = levels.iter().position(|existing| existing == &label) {
        index
    } else {
        let index = levels.len();
        levels.push(label);
        index
    }
}

fn summarize(
    values: &[f64],
    units: Option<&[String]>,
    context: &str,
    facet_identity: &FacetIdentity,
    category: &str,
    series: Option<&str>,
    input: &BarInput,
) -> Result<(f64, Option<(f64, f64)>), Error> {
    match &input.source {
        BarSource::Samples { error_bar, .. } => {
            let mean = stable_mean(values)
                .map_err(|reason| Error::Data(format!("bar: {context} {reason}")))?;
            let interval = generated_interval(
                values,
                units,
                mean,
                error_bar.kind(),
                facet_identity,
                category,
                series,
            )
            .map_err(|reason| Error::Data(format!("bar: {context} {reason}")))?;
            Ok((mean, interval))
        }
        BarSource::Estimates { .. } => match values {
            [value] if value.is_finite() => Ok((*value, None)),
            _ => Err(Error::Data(format!(
                "bar: {context} precomputed estimates require exactly one finite value, got {}",
                values.len()
            ))),
        },
    }
}

fn generated_interval(
    values: &[f64],
    units: Option<&[String]>,
    mean: f64,
    error_bar: &ErrorBarKind,
    facet_identity: &FacetIdentity,
    category: &str,
    series: Option<&str>,
) -> Result<Option<(f64, f64)>, &'static str> {
    if values.len() == 1 || matches!(error_bar, ErrorBarKind::Hidden) {
        return Ok(None);
    }
    match error_bar {
        ErrorBarKind::Hidden => Ok(None),
        ErrorBarKind::StandardDeviation(StandardDeviation { scale }) => {
            symmetric_interval(mean, sample_standard_deviation(values)? * scale)
        }
        ErrorBarKind::PercentileInterval(PercentileInterval { level }) => {
            central_quantiles(values, *level).map(Some)
        }
        ErrorBarKind::StandardError(StandardError { scale }) => symmetric_interval(
            mean,
            sample_standard_deviation(values)? / (values.len() as f64).sqrt() * scale,
        ),
        ErrorBarKind::ConfidenceInterval(interval) => match &interval.algorithm {
            ConfidenceIntervalAlgorithm::Bootstrap(bootstrap) => {
                if units.is_some_and(|units| effective_unit_count(units) == 1) {
                    return Ok(None);
                }
                let seed = stable_cell_seed(bootstrap.seed, facet_identity, category, series);
                percentile_bootstrap(values, units, interval.level, bootstrap.resamples, seed)
                    .map(Some)
            }
            ConfidenceIntervalAlgorithm::StudentT => {
                let critical = crate::stats::student_t_critical(values.len() - 1, interval.level)?;
                symmetric_interval(
                    mean,
                    critical * sample_standard_deviation(values)? / (values.len() as f64).sqrt(),
                )
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

fn sample_standard_deviation(values: &[f64]) -> Result<f64, &'static str> {
    if values.len() < 2 {
        return Err("sample standard deviation requires at least two observations");
    }
    let scale = values.iter().map(|value| value.abs()).fold(0.0, f64::max);
    if !scale.is_finite() {
        return Err("contains non-finite observations");
    }
    if scale == 0.0 {
        return Ok(0.0);
    }
    let mut mean = 0.0;
    let mut squared_deviations = 0.0;
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
        Err("sample standard deviation is not numerically representable")
    }
}

fn central_quantiles(values: &[f64], level: f64) -> Result<(f64, f64), &'static str> {
    let tail = (1.0 - level) / 2.0;
    let lower = crate::stats::distribution::quantile(values, tail)
        .map_err(|_| "could not compute the lower error-bar percentile")?;
    let upper = crate::stats::distribution::quantile(values, 1.0 - tail)
        .map_err(|_| "could not compute the upper error-bar percentile")?;
    if lower.is_finite() && upper.is_finite() && lower <= upper {
        Ok((lower, upper))
    } else {
        Err("percentile error-bar geometry is not numerically representable")
    }
}

fn percentile_bootstrap(
    values: &[f64],
    units: Option<&[String]>,
    level: f64,
    resamples: usize,
    seed: u64,
) -> Result<(f64, f64), &'static str> {
    if units.is_some_and(|units| units.len() != values.len()) {
        return Err("sampling units must align with observations");
    }
    let mut rng = SplitMix64::new(seed);
    let mut means = Vec::with_capacity(resamples);
    if let Some(units) = units {
        let grouped = grouped_unit_values(values, units);
        for _ in 0..resamples {
            let mut sample = Vec::new();
            for _ in 0..grouped.len() {
                let unit = &grouped[rng.index(grouped.len())].1;
                for _ in 0..unit.len() {
                    sample.push(unit[rng.index(unit.len())]);
                }
            }
            means.push(stable_mean(&sample)?);
        }
    } else {
        let mut sorted = values.to_vec();
        sorted.sort_by(f64::total_cmp);
        let mut sample = vec![0.0; sorted.len()];
        for _ in 0..resamples {
            for value in &mut sample {
                *value = sorted[rng.index(sorted.len())];
            }
            means.push(stable_mean(&sample)?);
        }
    }
    central_quantiles(&means, level)
}

fn grouped_unit_values(values: &[f64], units: &[String]) -> Vec<(String, Vec<f64>)> {
    let mut grouped: Vec<(String, Vec<f64>)> = Vec::new();
    for (value, unit) in values.iter().copied().zip(units) {
        if let Some((_, observations)) = grouped.iter_mut().find(|(label, _)| label == unit) {
            observations.push(value);
        } else {
            grouped.push((unit.clone(), vec![value]));
        }
    }
    grouped.sort_by(|left, right| left.0.cmp(&right.0));
    for (_, observations) in &mut grouped {
        observations.sort_by(f64::total_cmp);
    }
    grouped
}

fn effective_unit_count(units: &[String]) -> usize {
    let mut unique: Vec<&str> = Vec::new();
    for unit in units {
        if !unique.contains(&unit.as_str()) {
            unique.push(unit);
        }
    }
    unique.len()
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
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut value = self.state;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^ (value >> 31)
    }

    fn index(&mut self, upper: usize) -> usize {
        let upper = u64::try_from(upper).unwrap_or(u64::MAX);
        let limit = u64::MAX - u64::MAX % upper;
        loop {
            let value = self.next();
            if value < limit {
                return usize::try_from(value % upper).unwrap_or(0);
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

fn stable_mean(values: &[f64]) -> Result<f64, &'static str> {
    if values.is_empty() {
        return Err("cannot compute a mean from no observations");
    }
    let scale = values.iter().map(|value| value.abs()).fold(0.0, f64::max);
    if !scale.is_finite() {
        return Err("contains non-finite observations");
    }
    if scale == 0.0 {
        return Ok(0.0);
    }
    let mut sum = 0.0;
    let mut correction = 0.0;
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
        Err("mean is not numerically representable")
    }
}

fn stable_sum(values: &[f64]) -> Result<f64, &'static str> {
    if values.is_empty() {
        return Err("cannot compute a sum from no observations");
    }
    if values.iter().any(|value| !value.is_finite()) {
        return Err("contains non-finite observations");
    }
    if let Some(sum) = compensated_sum(values) {
        return Ok(sum);
    }

    let scale = values.iter().map(|value| value.abs()).fold(0.0, f64::max);
    if scale == 0.0 {
        return Ok(0.0);
    }
    let normalized: Vec<f64> = values.iter().map(|value| value / scale).collect();
    let sum = compensated_sum(&normalized).ok_or("sum is not numerically representable")? * scale;
    if sum.is_finite() {
        Ok(sum)
    } else {
        Err("sum is not numerically representable")
    }
}

fn compensated_sum(values: &[f64]) -> Option<f64> {
    let mut sum: f64 = 0.0;
    let mut correction: f64 = 0.0;
    for value in values {
        let next = sum + value;
        if !next.is_finite() {
            return None;
        }
        correction += if sum.abs() >= value.abs() {
            (sum - next) + value
        } else {
            (value - next) + sum
        };
        if !correction.is_finite() {
            return None;
        }
        sum = next;
    }
    let result = sum + correction;
    result.is_finite().then_some(result)
}

#[cfg(test)]
mod tests {
    use super::{
        generated_interval, prepare, stable_cell_seed, stable_mean, stable_sum, BarMode,
        FacetIdentity,
    };
    use crate::spec::{
        Bootstrap, ConfidenceInterval, ConfidenceIntervalAlgorithm, ErrorBarKind,
        PercentileInterval, StandardDeviation, StandardError,
    };
    use crate::{BarInput, Bounds, CategoryValues, ErrorBar, NumericValues, Plot};

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
    fn grouped_sample_cells_compute_independent_means() {
        let chart = Plot::bar(
            CategoryValues::new("category", ["a", "a", "a", "a"]).unwrap(),
            BarInput::samples(NumericValues::new("value", [1.0, 3.0, 10.0, 30.0]))
                .error_bar(ErrorBar::hidden()),
        )
        .group(CategoryValues::new("group", ["left", "left", "right", "right"]).unwrap())
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
    fn configured_sampling_units_are_scoped_to_each_group() {
        let units = CategoryValues::new("participant", ["p1", "p1", "p2", "p2"]).unwrap();
        let bootstrap = Bootstrap::new().resamples(50).unwrap().seed(7).units(units);
        let chart = Plot::bar(
            CategoryValues::new("category", ["a", "a", "a", "a"]).unwrap(),
            BarInput::samples(NumericValues::new("value", [1.0, 3.0, 10.0, 30.0]))
                .error_bar(ErrorBar::confidence_interval().bootstrap(bootstrap)),
        )
        .group(CategoryValues::new("group", ["left", "left", "right", "right"]).unwrap())
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
