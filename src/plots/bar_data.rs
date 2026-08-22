//! Shared data preparation for vertical and horizontal bar plots.

#![allow(clippy::cast_precision_loss)]

use crate::Error;

use crate::decorations::legend::LegendEntry;
use crate::render::backend::PointStyle;
use crate::theme::ThemePalette;
use crate::{BarAggregation, CategoryOrder, Plot, ValueSort};

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
    /// Exact caller-supplied significance mark.
    pub(crate) significance: String,
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
    let mut prepared = match mode {
        BarMode::Single => prepare_single(
            categories,
            channels.hues.as_deref(),
            values,
            hue_order,
            chart.bar_aggregation_spec(),
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
                chart.bar_aggregation_spec(),
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
    if chart.bar_aggregation_spec() == BarAggregation::Identity {
        if let Some(bounds) = chart.interval_bound_cols() {
            apply_identity_bounds(chart, data, &mut prepared, bounds)?;
        }
        if let Some(column) = chart.bar_significance_col() {
            apply_identity_significance(chart, data, &mut prepared, column)?;
        }
    }
    prepared.points = prepare_point_layers(chart, &prepared, mode)?;
    for point in &prepared.points {
        prepared.value_range.0 = prepared.value_range.0.min(point.value);
        prepared.value_range.1 = prepared.value_range.1.max(point.value);
    }
    Ok(prepared)
}

fn apply_identity_significance(
    chart: &Plot,
    data: crate::data::PlotData<'_>,
    prepared: &mut PreparedBars,
    column: &str,
) -> Result<(), Error> {
    let category_column = chart
        .x_col()
        .ok_or_else(|| Error::Data("bar: x column not set".into()))?;
    let categories = crate::data::strict_strings(data, category_column, "bar significance")?;
    let groups = chart
        .group_col()
        .map(|name| crate::data::strict_strings(data, name, "bar significance"))
        .transpose()?;
    let marks = crate::data::strict_strings(data, column, "bar significance")?;
    for row in 0..categories.len() {
        if crate::CoefficientSignificanceMark::parse_exact(&marks[row]).is_none() {
            return Err(Error::Data(format!(
                "bar significance column {column:?} has invalid value {:?} at source row {}; expected exactly \"\", \"*\", \"**\", or \"***\"",
                marks[row],
                data.source_row(row)
            )));
        }
        let category_index = prepared
            .categories
            .iter()
            .position(|category| category == &categories[row])
            .ok_or_else(|| Error::Data("bar significance category disappeared".into()))?;
        let series_index = groups
            .as_ref()
            .map(|values| {
                prepared
                    .series
                    .iter()
                    .position(|group| group == &values[row])
                    .ok_or_else(|| Error::Data("bar significance group disappeared".into()))
            })
            .transpose()?;
        let bar = prepared
            .bars
            .iter_mut()
            .find(|bar| bar.category_index == category_index && bar.series_index == series_index)
            .ok_or_else(|| Error::Data("bar significance cell disappeared".into()))?;
        bar.significance.clone_from(&marks[row]);
    }
    Ok(())
}

fn apply_identity_bounds(
    chart: &Plot,
    data: crate::data::PlotData<'_>,
    prepared: &mut PreparedBars,
    (lower_column, upper_column): (&str, &str),
) -> Result<(), Error> {
    let category_column = chart
        .x_col()
        .ok_or_else(|| Error::Data("bar: x column not set".into()))?;
    let categories = crate::data::strict_strings(data, category_column, "bar identity interval")?;
    let groups = chart
        .group_col()
        .map(|column| crate::data::strict_strings(data, column, "bar identity interval"))
        .transpose()?;
    let lower = crate::data::strict_finite_f64(data, lower_column, "bar identity interval")?;
    let upper = crate::data::strict_finite_f64(data, upper_column, "bar identity interval")?;
    if lower.len() != categories.len() || upper.len() != categories.len() {
        return Err(Error::Data(
            "bar identity interval: category/lower/upper length mismatch".into(),
        ));
    }
    for row in 0..categories.len() {
        let category_index = prepared
            .categories
            .iter()
            .position(|category| category == &categories[row])
            .ok_or_else(|| {
                Error::Data("bar identity interval category disappeared during preparation".into())
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
                            "bar identity interval graphical group disappeared during preparation"
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
                Error::Data("bar identity interval cell disappeared during preparation".into())
            })?;
        if lower[row] > bar.end || bar.end > upper[row] {
            return Err(Error::Data(format!(
                "bar identity interval row {row} requires finite lower <= value <= upper; got {} <= {} <= {}",
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

/// Validate cell cardinality for the selected ordinary-bar value construction.
#[expect(
    clippy::too_many_lines,
    reason = "one pass preserves source-row, facet, category, and graphical-group validation context"
)]
pub(crate) fn validate_aggregation_cells(
    chart: &Plot,
    data: crate::data::PlotData<'_>,
    aggregation: BarAggregation,
) -> Result<(), Error> {
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
            facet_values
                .iter()
                .map(|facet| facet[index].as_str())
                .chain(std::iter::once(categories[index].as_str()))
                .collect::<Vec<_>>()
                .join("\u{1f}")
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

    let mut cells: Vec<(Vec<String>, usize, usize)> = Vec::new();
    for index in 0..values.len() {
        let mut key = facet_values
            .iter()
            .map(|facet| facet[index].clone())
            .collect::<Vec<_>>();
        key.push(categories[index].clone());
        if let Some(groups) = &channels.groups {
            key.push(groups[index].clone());
        }
        if let Some((_, rows, finite)) = cells.iter_mut().find(|(existing, _, _)| existing == &key)
        {
            *rows += 1;
            *finite += usize::from(values[index].is_finite());
        } else {
            cells.push((key, 1, usize::from(values[index].is_finite())));
        }
    }
    if cells.is_empty() {
        return Err(Error::Data(format!(
            "bar aggregation {aggregation:?} requires at least one rendered category cell"
        )));
    }
    for (key, rows, finite) in cells {
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
        if finite != rows {
            return Err(Error::Data(format!(
                "generated bar interval cell `{cell}` contains {} non-finite observation(s); all rendered observations must be finite",
                rows - finite
            )));
        }
        match aggregation {
            BarAggregation::Mean if finite < 2 => {
                return Err(Error::Data(format!(
                    "bar mean cell `{cell}` has {finite} finite observation; at least two are required for a Student-t confidence interval"
                )));
            }
            BarAggregation::Identity if finite != 1 => {
                return Err(Error::Data(format!(
                    "bar identity cell `{cell}` has {finite} finite observations; exactly one is required"
                )));
            }
            BarAggregation::Mean | BarAggregation::Identity => {}
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
pub(crate) fn legend_entries(prepared: &PreparedBars, palette: &ThemePalette) -> Vec<LegendEntry> {
    prepared
        .hues
        .iter()
        .enumerate()
        .map(|(index, label)| {
            LegendEntry::point(
                label.clone(),
                PointStyle::new(palette.colors[index % palette.colors.len()], 3),
            )
        })
        .collect()
}

fn prepare_single(
    categories: Vec<String>,
    hues: Option<&[String]>,
    values: Vec<f64>,
    mut hue_levels: Vec<String>,
    aggregation: BarAggregation,
) -> Result<PreparedBars, Error> {
    let mut groups: Vec<(String, Option<usize>, Vec<f64>)> = Vec::new();
    for (row, (category, value)) in categories.into_iter().zip(values).enumerate() {
        let hue_index = hues.map(|hues| level_index(&mut hue_levels, hues[row].clone()));
        if let Some((_, _, category_values)) = groups
            .iter_mut()
            .find(|(seen_category, _, _)| seen_category == &category)
        {
            category_values.push(value);
        } else {
            groups.push((category, hue_index, vec![value]));
        }
    }
    if groups.is_empty() {
        return Err(Error::Data("bar: no groups".into()));
    }

    let categories: Vec<String> = groups
        .iter()
        .map(|(category, _, _)| category.clone())
        .collect();
    let mut bars = Vec::with_capacity(groups.len());
    for (category_index, (category, hue_index, ys)) in groups.iter().enumerate() {
        let (value, interval) = summarize(ys, &format!("category {category:?}"), aggregation)?;
        bars.push(PreparedBar {
            category_index,
            series_index: None,
            hue_index: *hue_index,
            start: 0.0,
            end: value,
            interval,
            significance: String::new(),
        });
    }
    let value_range = value_range(&bars)?;
    Ok(PreparedBars {
        categories,
        series: Vec::new(),
        hues: hue_levels,
        value_range,
        bars,
        points: Vec::new(),
    })
}

fn prepare_grouped(
    categories: Vec<String>,
    series: Vec<String>,
    hues: Option<&[String]>,
    values: Vec<f64>,
    series_order: Vec<String>,
    hue_order: Vec<String>,
    aggregation: BarAggregation,
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
        if let Some((_, _, _, ys)) = groups
            .iter_mut()
            .find(|(c, s, _, _)| *c == category_index && *s == series_index)
        {
            ys.push(value);
        } else {
            groups.push((category_index, series_index, hue_index, vec![value]));
        }
    }
    if groups.is_empty() {
        return Err(Error::Data("bar: no groups".into()));
    }

    let mut bars = Vec::with_capacity(groups.len());
    for (category_index, series_index, hue_index, ys) in groups {
        let context = format!(
            "category {:?}, graphical group {:?}",
            category_levels[category_index], series_levels[series_index]
        );
        let (value, interval) = summarize(&ys, &context, aggregation)?;
        bars.push(PreparedBar {
            category_index,
            series_index: Some(series_index),
            hue_index,
            start: 0.0,
            end: value,
            interval,
            significance: String::new(),
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
                significance: String::new(),
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

fn mean_summary(values: &[f64], context: &str) -> Result<(f64, f64), Error> {
    let mean =
        stable_mean(values).map_err(|reason| Error::Data(format!("bar: {context} {reason}")))?;
    let ci = mean_ci95(values, mean)
        .map_err(|reason| Error::Data(format!("bar: {context} {reason}")))?;
    if !mean.is_finite() || !ci.is_finite() || ci < 0.0 {
        return Err(Error::Data(format!(
            "bar: {context} produced non-finite mean or confidence interval"
        )));
    }
    if !(mean - ci).is_finite() || !(mean + ci).is_finite() {
        return Err(Error::Data(format!(
            "bar: {context} confidence interval geometry is not representable"
        )));
    }
    Ok((mean, ci))
}

fn summarize(
    values: &[f64],
    context: &str,
    aggregation: BarAggregation,
) -> Result<(f64, Option<(f64, f64)>), Error> {
    match aggregation {
        BarAggregation::Mean => {
            mean_summary(values, context).map(|(mean, ci)| (mean, Some((mean - ci, mean + ci))))
        }
        BarAggregation::Identity => match values {
            [value] if value.is_finite() => Ok((*value, None)),
            _ => Err(Error::Data(format!(
                "bar: {context} identity aggregation requires exactly one finite value, got {}",
                values.len()
            ))),
        },
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

pub(crate) fn mean_ci95(values: &[f64], mean: f64) -> Result<f64, &'static str> {
    if values.len() < 2 {
        return Err("sample mean confidence interval requires at least two observations");
    }
    let scale = values
        .iter()
        .map(|value| value.abs())
        .fold(mean.abs(), f64::max);
    if !scale.is_finite() {
        return Err("contains non-finite observations or mean");
    }
    if scale == 0.0 {
        return Ok(0.0);
    }
    let n = values.len() as f64;
    let mut normalized_mean = 0.0;
    let mut squared_deviations = 0.0;
    for (index, value) in values.iter().enumerate() {
        let normalized = value / scale;
        let count = (index + 1) as f64;
        let delta = normalized - normalized_mean;
        normalized_mean += delta / count;
        squared_deviations += delta * (normalized - normalized_mean);
    }
    let sem_normalized = (squared_deviations / (n - 1.0) / n).sqrt();
    let ci = (t_critical_975(values.len() - 1)? * sem_normalized) * scale;
    if ci.is_finite() {
        Ok(ci)
    } else {
        Err("confidence interval is not numerically representable")
    }
}

pub(crate) fn t_critical_975(df: usize) -> Result<f64, &'static str> {
    crate::stats::student_t_critical_975(df)
}

#[cfg(test)]
mod tests {
    use super::{mean_ci95, stable_mean, stable_sum, summarize, t_critical_975};
    use crate::BarAggregation;

    #[test]
    fn t_critical_uses_small_sample_values() {
        assert!((t_critical_975(1).unwrap() - 12.706_204_736_432_095).abs() < 1e-9);
        assert!((t_critical_975(10).unwrap() - 2.228_138_851_964_938_5).abs() < 1e-9);
    }

    #[test]
    fn t_critical_retains_finite_df_tail_for_large_samples() {
        assert!((t_critical_975(120).unwrap() - 1.979_930_405_052_777).abs() < 1e-9);
        assert!((t_critical_975(500).unwrap() - 1.964_719_837_467_343_8).abs() < 1e-9);
    }

    #[test]
    fn mean_ci95_matches_student_t_oracle_and_scales_affinely() {
        let values = [1.0, 3.0];
        let mean = 2.0;
        let ci = mean_ci95(&values, mean).unwrap();
        let transformed = [14.0, 22.0];
        let transformed_ci = mean_ci95(&transformed, 18.0).unwrap();

        assert!((ci - 12.706_204_736_432_095).abs() < 1e-9);
        assert!((transformed_ci - 4.0 * ci).abs() < 1e-9);
    }

    #[test]
    fn identity_summary_emits_no_confidence_interval_geometry() {
        let (value, interval) = summarize(&[3.5], "fixture", BarAggregation::Identity).unwrap();

        assert_eq!(value.to_bits(), 3.5_f64.to_bits());
        assert_eq!(interval, None);
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

    #[test]
    fn mean_ci95_avoids_intermediate_variance_overflow() {
        let mut values = [0.0; 16];
        values[8..].fill(f64::MAX / 2.0);
        let mean = stable_mean(&values).unwrap();

        assert!(mean_ci95(&values, mean).unwrap().is_finite());
    }

    #[test]
    fn mean_ci95_rejects_unrepresentable_interval() {
        let values = [-f64::MAX, f64::MAX];
        let mean = stable_mean(&values).unwrap();

        assert_eq!(
            mean_ci95(&values, mean).unwrap_err(),
            "confidence interval is not numerically representable"
        );
    }
}
