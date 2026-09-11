//! Horizontal categorical dumbbells with explicit paired endpoints.

#![expect(
    clippy::cast_precision_loss,
    reason = "validated row counts are converted to bounded plotting coordinates"
)]

use std::collections::HashSet;

use crate::Error;

use crate::decorations::legend::LegendEntry;
use crate::layout::{AxisSpec, AxisTick};
use crate::render::backend::{with_clip, Backend, LineStyle, Point2D};
use crate::spec::{CategoryOrder, PlotRenderContext, PlotRenderResult};
use crate::Plot;

struct DumbbellRow {
    category_index: usize,
    group_index: usize,
    left: f64,
    right: f64,
}

struct DumbbellData {
    categories: Vec<String>,
    group_count: usize,
    rows: Vec<DumbbellRow>,
    range: (f64, f64),
}

pub(crate) fn render(
    chart: &Plot,
    context: PlotRenderContext<'_>,
    backend: &mut dyn Backend,
) -> Result<PlotRenderResult, crate::layout::fit::PlanError> {
    let prepared = prepare(chart, context.data)?;
    let endpoints = chart
        .dumbbell_endpoints_spec()
        .expect("dumbbell input owns endpoint metadata");
    let category_count = prepared.categories.len();
    let layout = context.axis_layout(|| {
        let x_range = crate::layout::continuous_auto_range(prepared.range, chart.x_axis_spec());
        let x_axis = crate::layout::display_axis_spec(chart.x_axis_spec(), x_range);
        let y_ticks = prepared
            .categories
            .iter()
            .enumerate()
            .map(|(index, category)| AxisTick::new(row_y(index, category_count), category.clone()))
            .collect();
        let y_axis = AxisSpec::default().limits(0.0, category_count as f64);
        let panel = context.panel;
        context.layout_with_ticks(
            x_range,
            (0.0, category_count as f64),
            panel,
            chart.x_label_text().unwrap_or("").to_owned(),
            chart.y_label_text().unwrap_or("").to_owned(),
            Vec::new(),
            y_ticks,
            &x_axis,
            &y_axis,
            context.theme,
        )
    })?;
    let connector = LineStyle::solid(context.theme.foreground, 1.2);
    with_clip(backend, layout.projection.panel_box(), |backend| {
        for row in &prepared.rows {
            let y = replicate_y(
                row.category_index,
                category_count,
                row.group_index,
                prepared.group_count,
            );
            let (left_x, left_y) = layout.projection.project(row.left, y);
            let (right_x, right_y) = layout.projection.project(row.right, y);
            backend.draw_line(
                &[Point2D::new(left_x, left_y), Point2D::new(right_x, right_y)],
                &connector,
            )?;
            backend.draw_scatter(
                &[Point2D::new(left_x, left_y)],
                &chart.point_style(endpoints.left_color(), 4),
            )?;
            backend.draw_scatter(
                &[Point2D::new(right_x, right_y)],
                &chart.point_style(endpoints.right_color(), 4),
            )?;
        }
        Ok(())
    })?;

    let legend = vec![
        LegendEntry::point(
            chart.legend_identity(endpoints.left_label()),
            chart.point_style(endpoints.left_color(), 4),
        ),
        LegendEntry::point(
            chart.legend_identity(endpoints.right_label()),
            chart.point_style(endpoints.right_color(), 4),
        ),
    ];
    Ok(PlotRenderResult::new(layout, legend))
}

pub(crate) fn shared_x_domain(
    chart: &Plot,
    data: crate::data::PlotData<'_>,
) -> Result<(f64, f64), Error> {
    let left = crate::data::strict_finite_f64(
        data,
        chart
            .y_col()
            .ok_or_else(|| Error::Data("dumbbell: left value column not set".into()))?,
        "dumbbell",
    )?;
    let right = crate::data::strict_finite_f64(
        data,
        chart
            .dumbbell_right_col()
            .ok_or_else(|| Error::Data("dumbbell: right value column not set".into()))?,
        "dumbbell",
    )?;
    data.claim_source_visits(left.len())?;
    data.claim_source_visits(right.len())?;
    Ok(crate::layout::continuous_auto_range(
        crate::data::min_max(left.into_iter().chain(right), "dumbbell")?,
        chart.x_axis_spec(),
    ))
}

pub(crate) fn axis_categories(
    chart: &Plot,
    data: crate::data::PlotData<'_>,
) -> Result<Vec<String>, Error> {
    let categories = crate::data::strict_normalized_strings(
        data,
        chart
            .x_col()
            .ok_or_else(|| Error::Data("dumbbell: category column not set".into()))?,
        "dumbbell",
        "category",
    )?;
    data.source_levels(&categories)
}

fn prepare<'a>(
    chart: &Plot,
    data: impl Into<crate::data::PlotData<'a>>,
) -> Result<DumbbellData, Error> {
    let data = data.into();
    if chart.y_axis_spec() != &AxisSpec::default() {
        return Err(Error::Data(
            "dumbbell: categorical y-axis limits are not supported".into(),
        ));
    }
    if chart.hue_col().is_some() || chart.color_col().is_some() {
        return Err(Error::Data(
            "dumbbell: hue and continuous color channels are not supported; endpoint colors are explicit"
                .into(),
        ));
    }
    if !matches!(chart.category_order_spec(), CategoryOrder::Observed) {
        return Err(Error::Data(
            "dumbbell: category ordering is not supported; category order is first-observed".into(),
        ));
    }
    let category_col = chart
        .x_col()
        .ok_or_else(|| Error::Data("dumbbell: category column not set".into()))?;
    let left_col = chart
        .y_col()
        .ok_or_else(|| Error::Data("dumbbell: left value column not set".into()))?;
    let right_col = chart
        .dumbbell_right_col()
        .ok_or_else(|| Error::Data("dumbbell: right value column not set".into()))?;
    let categories =
        crate::data::strict_normalized_strings(data, category_col, "dumbbell", "category")?;
    let left = crate::data::strict_finite_f64(data, left_col, "dumbbell")?;
    let right = crate::data::strict_finite_f64(data, right_col, "dumbbell")?;
    let groups = chart
        .group_col()
        .map(|column| {
            crate::data::strict_normalized_strings(data, column, "dumbbell", "replicate/group")
        })
        .transpose()?;
    if categories.is_empty()
        || categories.len() != left.len()
        || categories.len() != right.len()
        || groups
            .as_ref()
            .is_some_and(|values| values.len() != categories.len())
    {
        return Err(Error::Data(
            "dumbbell: category, left, right, and optional group columns must have equal non-zero lengths"
                .into(),
        ));
    }

    let category_levels = crate::data::stable_levels(&categories);
    let group_levels = groups
        .as_ref()
        .map_or_else(Vec::new, |values| crate::data::stable_levels(values));
    let mut seen = HashSet::with_capacity(categories.len());
    let mut rows = Vec::with_capacity(categories.len());
    for index in 0..categories.len() {
        let source_row = data.source_row(index);
        let group = groups.as_ref().map(|values| values[index].as_str());
        if !seen.insert((categories[index].as_str(), group)) {
            return Err(Error::Data(format!(
                "dumbbell: duplicate normalized (category, group) key ({:?}, {:?}) at source row {}",
                categories[index],
                group,
                source_row
            )));
        }
        let category_index =
            resolved_level_index(&category_levels, &categories[index], "category", source_row)?;
        let group_index = group
            .map(|group| resolved_level_index(&group_levels, group, "group", source_row))
            .transpose()?
            .unwrap_or(0);
        rows.push(DumbbellRow {
            category_index,
            group_index,
            left: left[index],
            right: right[index],
        });
    }
    let range = crate::data::min_max(left.into_iter().chain(right), "dumbbell")?;
    Ok(DumbbellData {
        categories: category_levels,
        group_count: group_levels.len().max(1),
        rows,
        range,
    })
}

fn resolved_level_index(
    levels: &[String],
    value: &str,
    dimension: &str,
    source_row: usize,
) -> Result<usize, Error> {
    levels
        .iter()
        .position(|level| level == value)
        .ok_or_else(|| {
            Error::Data(format!(
                "dumbbell: normalized {dimension} {value:?} at source row {source_row} is absent from the resolved {dimension} order"
            ))
        })
}

fn row_y(category_index: usize, category_count: usize) -> f64 {
    category_count as f64 - category_index as f64 - 0.5
}

fn replicate_y(
    category_index: usize,
    category_count: usize,
    group_index: usize,
    group_count: usize,
) -> f64 {
    let center = row_y(category_index, category_count);
    let slot = 0.72 / group_count as f64;
    center + 0.36 - slot * (group_index as f64 + 0.5)
}
