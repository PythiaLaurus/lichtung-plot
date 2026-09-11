//! Long-form categorical heatmap with a continuous colorbar.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::too_many_lines
)]

use std::collections::{HashMap, HashSet};

use crate::Error;
use unicode_segmentation::UnicodeSegmentation;

use crate::layout::{AxisSpec, AxisTick, Layout, PanelBox, PhysicalTextAlignment, TextBlock};
use crate::render::backend::{
    with_clip, Backend, FillStyle, GradientStop, LineStyle, PlacedText, Point2D, TextAnchor,
};
use crate::spec::{
    CategoryOrder, HeatmapAnnotations, HeatmapLabelFit, HeatmapLayout, HeatmapScale, LegendPolicy,
    PlotRenderContext, PlotRenderResult,
};
use crate::theme::ResolvedTheme;
use crate::Plot;

const MAX_CELLS: usize = 1_000_000;
const ANNOTATION_FONT_SIZE: u32 = 11;
const MISSING_FILL: (u8, u8, u8) = (224, 224, 224);
const MISSING_STROKE: (u8, u8, u8) = (120, 120, 120);

#[derive(Debug, PartialEq)]
struct HeatmapData {
    x_levels: Vec<String>,
    y_levels: Vec<String>,
    cells: Vec<Option<f64>>,
    highlighted: Vec<bool>,
    null_row_annotations: Vec<Option<String>>,
    has_missing: bool,
    domain: ColorDomain,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ColorDomain {
    minimum: f64,
    maximum: f64,
    center: Option<f64>,
}

pub(crate) fn text_census<'a>(
    chart: &Plot,
    data: impl Into<crate::data::PlotData<'a>>,
    mut visit: impl FnMut(crate::font::canonical::CensusSource<'_>) -> Result<(), Error>,
) -> Result<(), Error> {
    let prepared = prepare(chart, data)?;
    for value in [prepared.domain.minimum, prepared.domain.maximum] {
        visit(format_args!("{}", ColorbarValue(value)).into())?;
    }
    if let Some(center) = prepared
        .domain
        .center
        .filter(|center| *center > prepared.domain.minimum && *center < prepared.domain.maximum)
    {
        visit(format_args!("{}", ColorbarValue(center)).into())?;
    }
    if let HeatmapAnnotations::Values(format) = chart.heatmap_annotations_spec() {
        for value in prepared.cells.iter().flatten() {
            visit(format_args!("{}", format.display(*value)).into())?;
        }
    }
    if prepared.has_missing {
        visit("Missing".into())?;
    }
    Ok(())
}

pub(crate) fn render(
    chart: &Plot,
    context: PlotRenderContext<'_>,
    backend: &mut dyn Backend,
) -> Result<PlotRenderResult, crate::layout::fit::PlanError> {
    let theme = context.theme;
    let mut text = context.producer_text()?;
    let prepared = prepare(chart, context.data)?;
    let color_label = chart.resolved_color_label();
    let detail_font_size = theme.detail_font_size;
    let guide_scope = context
        .fit_scope()
        .child(crate::layout::AddressRole::OutsideGuideRight, 0)?;
    let title_width = if prepared.has_missing {
        (context.panel.height() - 34.0).clamp(40.0, 160.0)
    } else {
        context.panel.height().clamp(40.0, 180.0)
    };
    let color_label_block =
        TextBlock::from_resolved(text.plan(&color_label, "heatmap colorbar title", || {
            crate::layout::fit::measured(crate::layout::fit::text(
                theme,
                &color_label,
                &theme.legacy_text_appearance(detail_font_size, crate::TextWeight::Normal),
                title_width,
                PhysicalTextAlignment::Center,
                "heatmap colorbar title",
                &guide_scope.child(crate::layout::AddressRole::GuideTitle, 0)?,
            ))
        })?);
    let mut colorbar_values = vec![
        format_colorbar_value(prepared.domain.minimum),
        format_colorbar_value(prepared.domain.maximum),
    ];
    if let Some(center) = prepared
        .domain
        .center
        .filter(|center| *center > prepared.domain.minimum && *center < prepared.domain.maximum)
    {
        colorbar_values.push(format_colorbar_value(center));
    }
    if prepared.has_missing {
        colorbar_values.push("Missing".to_string());
    }
    let value_plans = colorbar_values
        .iter()
        .map(|label| {
            text.plan(label, "heatmap colorbar value", || {
                Ok(theme.measure_legacy_text(
                    label,
                    detail_font_size,
                    PhysicalTextAlignment::Start,
                    "heatmap colorbar value",
                )?)
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let value_width = value_plans
        .iter()
        .map(crate::layout::ResolvedTextPlan::width)
        .fold(0.0_f64, f64::max);
    let colorbar_reserve = (36.0 + value_width).max(
        73.0 + color_label_block.lines().len().saturating_sub(1) as f64
            * color_label_block.line_height()
            + f64::from(detail_font_size) / 2.0,
    ) + 4.0;
    if context.panel.width() - colorbar_reserve < 40.0 {
        return Err(guide_scope.failure(crate::layout::fit::FitKind::LegendOrGuide, "heatmap: colorbar title does not fit while preserving the minimum 40px plot panel; increase chart width").into());
    }
    let mut body = context.panel;
    body.x1 -= colorbar_reserve;
    let mut annotations = None;
    let layout = context.axis_layout(|| -> Result<_, crate::layout::fit::PlanError> {
    let x_labels = match context.prepared_category_labels(0) {
        Some(labels) => labels.to_vec(),
        None => chart.resolve_label_domain("heatmap x-axis categories", prepared.x_levels.iter().map(String::as_str))?,
    };
    let y_labels = match context.prepared_category_labels(1) {
        Some(labels) => labels.to_vec(),
        None => chart.resolve_label_domain("heatmap y-axis categories", prepared.y_levels.iter().map(String::as_str))?,
    };
    let wrap_labels = chart.heatmap_label_fit_spec() == HeatmapLabelFit::Wrap;
    // Heatmap owns its rotated/elided text adapter until family extraction.
    let chrome = crate::layout::y_axis_left_inset_for_tick_labels(
        0.0,
        theme.axes.ticks.as_ref().map_or(0, |ticks| ticks.length),
        theme.typography.axis_title.font_size,
        &theme.axes.spacing,
    );
    let base_left = chart.base_canvas_layout(theme).panel_box.x0.min(body.x0);
    let available = body.width() - 40.0 + base_left - chrome;
    if available <= 0.0 {
        return Err(context
            .fit_scope()
            .failure(
                crate::layout::fit::FitKind::TrackOrResidual,
                "heatmap: y-axis labels leave no width for the minimum 40px plot panel",
            )
            .into());
    }
    let mut natural = 1.0 / 1024.0;
    for label in &y_labels {
        let plan = theme.measure_text(
            label,
            &theme.typography.tick_label,
            PhysicalTextAlignment::End,
            "y-axis category label",
            false,
        )?;
        natural = f64::max(natural, plan.width().max(plan.fit_width()));
    }
    let tick_width = if natural <= available || !wrap_labels {
        natural.min(available)
    } else {
        theme
            .intrinsic_tick_label_width(y_labels.iter().map(String::as_str), body.height())?
            .min(available)
    };
    let mut panel = body;
    panel.x0 += (tick_width + chrome - base_left).max(0.0);
    let minimum_panel_height = if prepared.has_missing { 90.0 } else { 60.0 };
    if panel.width() < 40.0
        || panel.height() < minimum_panel_height
        || panel.x1 + colorbar_reserve > f64::from(chart.width())
    {
        return Err(context.fit_scope().failure(crate::layout::fit::FitKind::SpecializedGeometry,
            "heatmap: canvas is too small after reserving categorical labels and the colorbar; increase chart size"
        ).into());
    }

    let x_count = prepared.x_levels.len();
    let y_count = prepared.y_levels.len();
    let x_ticks = prepared
        .x_levels
        .iter()
        .enumerate()
        .map(|(index, _)| AxisTick::new(index as f64 + 0.5, ""))
        .collect();
    let y_labels = if wrap_labels {
        y_labels
    } else {
        fitted_unique_labels(
            theme,
            &y_labels,
            tick_width,
            "y",
            &context
                .fit_scope()
                .child(crate::layout::AddressRole::AxisLeft, 0)?,
        )?
    };
    let y_ticks = y_labels
        .iter()
        .enumerate()
        .map(|(index, label)| AxisTick::new(row_y(index, y_count), label.clone()))
        .collect();
    let x_spec = AxisSpec::default().limits(0.0, x_count as f64);
    let y_spec = AxisSpec::default().limits(0.0, y_count as f64);
        let mut layout = Layout::new_with_ticks_and_axis_specs_in_panel_box(
            (0.0, x_count as f64),
            (0.0, y_count as f64),
            panel,
            chart.x_label_text().unwrap_or("").to_string(),
            chart.y_label_text().unwrap_or("").to_string(),
            x_ticks,
            y_ticks,
            &x_spec,
            &y_spec,
            theme,
            context.fit_scope(),
        )?;
    // Elision already selected the complete displayed labels; do not wrap them again.
        layout.y_axis.explicit_ticks = !wrap_labels;
        layout.y_axis.assigned_tick_width = Some(tick_width);
        // Preserve annotation-error precedence before the final x-label fit.
        annotations = Some(plan_annotations(
            chart, &prepared, &layout, theme, context.fit_scope(), &text,
        )?);
    let x_labels = if wrap_labels {
        x_labels
    } else {
        fitted_unique_labels(
            theme,
            &x_labels,
            panel.width() / x_count as f64 * 0.9,
            "x",
            &context
                .fit_scope()
                .child(crate::layout::AddressRole::AxisBottom, 0)?,
        )?
    };
    for (tick, label) in layout.x_axis.ticks.iter_mut().zip(x_labels) {
        tick.label = label;
    }
    Ok(layout)
    })?;
    let panel = layout.projection.panel_box();
    let annotations = annotations.map_or_else(
        || plan_annotations(chart, &prepared, &layout, theme, context.fit_scope(), &text),
        Ok,
    )?;

    with_clip(backend, panel, |backend| {
        draw_cells(
            chart,
            &prepared,
            &layout,
            theme,
            &annotations,
            &mut text,
            backend,
        )
    })?;
    draw_colorbar(
        &prepared,
        &layout,
        &color_label_block,
        &value_plans,
        &mut text,
        backend,
    )?;

    let mut result = PlotRenderResult::new(layout, Vec::new())
        .with_resolved_axis_keys(prepared.x_levels.clone(), prepared.y_levels.clone());
    result.producer_text = text.finish()?;
    Ok(result)
}

pub(crate) fn axis_categories(
    chart: &Plot,
    data: crate::data::PlotData<'_>,
) -> Result<[Vec<String>; 2], Error> {
    let x = normalized_categories(
        data,
        chart
            .x_col()
            .ok_or_else(|| Error::Data("heatmap: x category column not set".into()))?,
        "x",
    )?;
    let y = normalized_categories(
        data,
        chart
            .y_col()
            .ok_or_else(|| Error::Data("heatmap: y category column not set".into()))?,
        "y",
    )?;
    category_levels(chart, data, &x, &y)
}

fn category_levels(
    chart: &Plot,
    data: crate::data::PlotData<'_>,
    x: &[String],
    y: &[String],
) -> Result<[Vec<String>; 2], Error> {
    match chart.heatmap_layout_spec() {
        HeatmapLayout::Cartesian => Ok([
            resolve_axis_order(
                &data.source_levels(x)?,
                chart.x_category_order_spec(),
                "x",
                !data.is_full_plot(),
                data,
            )?,
            resolve_axis_order(
                &data.source_levels(y)?,
                chart.y_category_order_spec(),
                "y",
                !data.is_full_plot(),
                data,
            )?,
        ]),
        layout => {
            let observed = triangular_family_order(x, y, layout, data)?;
            let family = resolve_triangular_order(chart, &observed, !data.is_full_plot(), data)?;
            let copy =
                data.collect_source(family.iter().map(|level| data.copy_source_string(level)))?;
            Ok([copy, family])
        }
    }
}

fn prepare<'a>(
    chart: &Plot,
    data: impl Into<crate::data::PlotData<'a>>,
) -> Result<HeatmapData, Error> {
    let data = data.into();
    validate_options(chart)?;
    let x_col = chart
        .x_col()
        .ok_or_else(|| Error::Data("heatmap: x category column not set".into()))?;
    let y_col = chart
        .y_col()
        .ok_or_else(|| Error::Data("heatmap: y category column not set".into()))?;
    let color_col = chart
        .color_col()
        .ok_or_else(|| Error::Data("heatmap: numeric color column not set".into()))?;

    let x_values = normalized_categories(data, x_col, "x")?;
    let y_values = normalized_categories(data, y_col, "y")?;
    let fills = crate::data::nullable_finite_f64(data, color_col, "heatmap")?;
    let highlights = chart
        .heatmap_highlight_col()
        .map(|column| crate::data::strict_bool(data, column, "heatmap"))
        .transpose()?;
    if x_values.is_empty() || x_values.len() != y_values.len() || x_values.len() != fills.len() {
        return Err(Error::Data(
            "heatmap: x/y/color columns must have equal non-zero lengths".into(),
        ));
    }
    if highlights
        .as_ref()
        .is_some_and(|values| values.len() != x_values.len())
    {
        return Err(Error::Data(
            "heatmap: highlight column must align with x/y/color columns".into(),
        ));
    }

    let heatmap_layout = chart.heatmap_layout_spec();
    let [x_levels, y_levels] = category_levels(chart, data, &x_values, &y_values)?;
    let cell_count = x_levels
        .len()
        .checked_mul(y_levels.len())
        .ok_or_else(|| Error::Data("heatmap: observed grid cell count overflowed".into()))?;
    if cell_count > MAX_CELLS {
        return Err(Error::Data(format!(
            "heatmap: observed grid has {cell_count} cells, exceeding the safety limit of {MAX_CELLS}"
        )));
    }

    let x_indices = x_levels
        .iter()
        .enumerate()
        .map(|(index, level)| (level.as_str(), index))
        .collect::<HashMap<_, _>>();
    let y_indices = y_levels
        .iter()
        .enumerate()
        .map(|(index, level)| (level.as_str(), index))
        .collect::<HashMap<_, _>>();
    let explicit_triangular_order = heatmap_layout != HeatmapLayout::Cartesian
        && (!matches!(chart.x_category_order_spec(), CategoryOrder::Observed)
            || !matches!(chart.y_category_order_spec(), CategoryOrder::Observed));
    let mut seen = HashSet::with_capacity(x_values.len());
    let mut cells = vec![None; cell_count];
    let mut highlighted = vec![false; cell_count];
    for (row, ((x, y), fill)) in x_values.iter().zip(&y_values).zip(&fills).enumerate() {
        let mut x_index = x_indices[x.as_str()];
        let mut y_index = y_indices[y.as_str()];
        if explicit_triangular_order && !active_cell(heatmap_layout, x_index, y_index) {
            std::mem::swap(&mut x_index, &mut y_index);
        }
        if !seen.insert((x_index, y_index)) {
            let source_row = data.source_row(row);
            return Err(Error::Data(format!(
                "heatmap: duplicate normalized (x, y) pair ({x:?}, {y:?}) at source row {source_row}"
            )));
        }
        if !active_cell(heatmap_layout, x_index, y_index) {
            let source_row = data.source_row(row);
            return Err(Error::Data(format!(
                "heatmap: normalized (x, y) pair ({x:?}, {y:?}) at source row {source_row} is outside the active {heatmap_layout:?} layout"
            )));
        }
        cells[y_index * x_levels.len() + x_index] = *fill;
        highlighted[y_index * x_levels.len() + x_index] =
            highlights.as_ref().is_some_and(|values| values[row]);
    }
    let null_row_annotations = resolve_null_row_annotations(
        chart,
        heatmap_layout,
        &y_levels,
        &cells,
        &highlighted,
        x_levels.len(),
    )?;

    let finite = fills.iter().flatten().copied().collect::<Vec<_>>();
    if finite.is_empty() && chart.heatmap_color_limits_spec().is_none() {
        return Err(Error::Data(
            "heatmap: color column must contain at least one finite value unless explicit color limits are set"
                .into(),
        ));
    }
    let domain = color_domain(
        &finite,
        chart.heatmap_scale_spec(),
        chart.heatmap_color_limits_spec(),
    )?;
    let has_missing = cells.iter().enumerate().any(|(index, value)| {
        let x_index = index % x_levels.len();
        let y_index = index / x_levels.len();
        active_cell(heatmap_layout, x_index, y_index) && value.is_none()
    });

    Ok(HeatmapData {
        x_levels,
        y_levels,
        cells,
        highlighted,
        null_row_annotations,
        has_missing,
        domain,
    })
}

fn validate_options(chart: &Plot) -> Result<(), Error> {
    if chart.x_axis_spec() != &AxisSpec::default() || chart.y_axis_spec() != &AxisSpec::default() {
        return Err(Error::Data(
            "heatmap: categorical axis limits are not supported".into(),
        ));
    }
    if let Some(hue_col) = chart.hue_col() {
        return Err(Error::Data(format!(
            "heatmap: hue column {hue_col:?} is not supported; use color for numeric fill"
        )));
    }
    if !matches!(chart.category_order_spec(), CategoryOrder::Observed) {
        return Err(Error::Data(
            "heatmap: primary category order is ambiguous; use x_category_order(...) and y_category_order(...)"
                .into(),
        ));
    }
    for (axis, order) in [
        ("x", chart.x_category_order_spec()),
        ("y", chart.y_category_order_spec()),
    ] {
        if let CategoryOrder::Value(sort) = order {
            return Err(Error::Data(format!(
                "heatmap: {axis} categories cannot use ValueSort::{sort:?}; each category maps to multiple cells, so supply an explicit order"
            )));
        }
    }
    if chart.legend_policy_spec() != LegendPolicy::Automatic {
        return Err(Error::Data(
            "heatmap: legend policy overrides are not supported; the right-side colorbar is mandatory"
                .into(),
        ));
    }
    if let HeatmapScale::Diverging { center } = chart.heatmap_scale_spec() {
        if !center.is_finite() {
            return Err(Error::Data(format!(
                "heatmap: diverging center must be finite, got {center}"
            )));
        }
    }
    Ok(())
}

fn resolve_null_row_annotations(
    chart: &Plot,
    layout: HeatmapLayout,
    y_levels: &[String],
    cells: &[Option<f64>],
    highlighted: &[bool],
    x_count: usize,
) -> Result<Vec<Option<String>>, Error> {
    let annotations = chart.heatmap_null_row_annotations_spec();
    let mut resolved = vec![None; y_levels.len()];
    if annotations.is_empty() {
        return Ok(resolved);
    }
    if layout != HeatmapLayout::Cartesian {
        return Err(Error::Data(
            "heatmap: fully-null row annotations require HeatmapLayout::Cartesian".into(),
        ));
    }

    let mut seen = HashSet::with_capacity(annotations.len());
    for annotation in annotations {
        let row_label = crate::data::collapse_whitespace(annotation.row_label());
        if !seen.insert(row_label.clone()) {
            return Err(Error::Data(format!(
                "heatmap: duplicate fully-null row annotation for normalized y category {row_label:?}"
            )));
        }
        let y_index = y_levels
            .iter()
            .position(|level| level == &row_label)
            .ok_or_else(|| {
                Error::Data(format!(
                    "heatmap: fully-null row annotation targets unknown normalized y category {row_label:?}"
                ))
            })?;
        let row = &cells[y_index * x_count..(y_index + 1) * x_count];
        if row.iter().any(Option::is_some) {
            return Err(Error::Data(format!(
                "heatmap: row {row_label:?} must be fully null before it can receive a merged row annotation"
            )));
        }
        if highlighted[y_index * x_count..(y_index + 1) * x_count]
            .iter()
            .any(|value| *value)
        {
            return Err(Error::Data(format!(
                "heatmap: fully-null row {row_label:?} cannot combine a merged annotation with true cell highlights"
            )));
        }
        resolved[y_index] = Some(annotation.text().to_owned());
    }
    Ok(resolved)
}

fn normalized_categories(
    data: crate::data::PlotData<'_>,
    column: &str,
    axis: &str,
) -> Result<Vec<String>, Error> {
    data.collect_source(
        crate::data::strict_strings(data, column, "heatmap")?
            .into_iter()
            .enumerate()
            .map(|(row, value)| {
                let normalized = data.normalize_source_string(&value)?;
                if normalized.is_empty() {
                    let source_row = data.source_row(row);
                    Err(Error::Data(format!(
                        "heatmap: {axis} category label is empty at source row {source_row}"
                    )))
                } else {
                    Ok(normalized)
                }
            }),
    )
}

fn resolve_axis_order(
    observed: &[String],
    order: &CategoryOrder,
    axis: &str,
    is_subset: bool,
    data: crate::data::PlotData<'_>,
) -> Result<Vec<String>, Error> {
    match order {
        CategoryOrder::Observed => data.collect_source(observed.iter().map(|level| data.copy_source_string(level))),
        CategoryOrder::Explicit(requested) => {
            let requested = data.collect_source(requested.iter().map(|level| data.normalize_source_string(level)))?;
            data.claim_source_visits(requested.len())?;
            if requested.iter().any(String::is_empty) {
                return Err(Error::Data(format!(
                    "heatmap: explicit {axis} category order contains an empty normalized level"
                )));
            }
            crate::data::explicit_category_order(
                observed,
                &requested,
                &format!("heatmap {axis} category order"),
                is_subset,
                Some(data),
            )
        }
        CategoryOrder::Value(sort) => Err(Error::Data(format!(
            "heatmap: {axis} categories cannot use ValueSort::{sort:?}; each category maps to multiple cells, so supply an explicit order"
        ))),
    }
}

fn resolve_triangular_order(
    chart: &Plot,
    observed: &[String],
    is_subset: bool,
    data: crate::data::PlotData<'_>,
) -> Result<Vec<String>, Error> {
    match (chart.x_category_order_spec(), chart.y_category_order_spec()) {
        (CategoryOrder::Observed, CategoryOrder::Observed) => {
            data.collect_source(observed.iter().map(|level| data.copy_source_string(level)))
        }
        (CategoryOrder::Observed, y) => resolve_axis_order(observed, y, "y", is_subset, data),
        (x, CategoryOrder::Observed) => resolve_axis_order(observed, x, "x", is_subset, data),
        (x, y) => {
            let x = resolve_axis_order(observed, x, "x", is_subset, data)?;
            let y = resolve_axis_order(observed, y, "y", is_subset, data)?;
            data.claim_source_visits(x.len())?;
            if x != y {
                return Err(Error::Data(
                    "heatmap: triangular x and y category orders must resolve to the same family order"
                        .into(),
                ));
            }
            Ok(x)
        }
    }
}

fn triangular_family_order(
    x_values: &[String],
    y_values: &[String],
    layout: HeatmapLayout,
    data: crate::data::PlotData<'_>,
) -> Result<Vec<String>, Error> {
    let mut levels = Vec::new();
    for (x, y) in x_values.iter().zip(y_values) {
        let pair = match layout {
            HeatmapLayout::UpperTriangle { .. } => [y, x],
            HeatmapLayout::LowerTriangle { .. } | HeatmapLayout::Cartesian => [x, y],
        };
        for value in pair {
            data.claim_source_visits(levels.len())?;
            if !levels.contains(value) {
                data.reserve_source(&mut levels, 1)?;
                levels.push(data.copy_source_string(value)?);
            }
        }
    }
    Ok(levels)
}

const fn active_cell(layout: HeatmapLayout, x_index: usize, y_index: usize) -> bool {
    match layout {
        HeatmapLayout::Cartesian => true,
        HeatmapLayout::LowerTriangle { diagonal } => {
            y_index > x_index || (diagonal && y_index == x_index)
        }
        HeatmapLayout::UpperTriangle { diagonal } => {
            x_index > y_index || (diagonal && x_index == y_index)
        }
    }
}

fn color_domain(
    values: &[f64],
    scale: HeatmapScale,
    explicit: Option<(f64, f64)>,
) -> Result<ColorDomain, Error> {
    let center = match scale {
        HeatmapScale::Sequential => None,
        HeatmapScale::Diverging { center } => Some(center),
    };
    if let Some((minimum, maximum)) = explicit {
        if !minimum.is_finite() || !maximum.is_finite() || minimum >= maximum {
            return Err(Error::Data(format!(
                "heatmap: color limits must be finite and strictly ascending; got {minimum}..{maximum}"
            )));
        }
        if let Some(center) = center.filter(|center| *center < minimum || *center > maximum) {
            return Err(Error::Data(format!(
                "heatmap: diverging color limits {minimum}..{maximum} must contain center {center}"
            )));
        }
        return Ok(ColorDomain {
            minimum,
            maximum,
            center,
        });
    }

    let (minimum, maximum) = crate::data::min_max(values.iter().copied(), "heatmap")?;
    let Some(center) = center else {
        return Ok(ColorDomain {
            minimum,
            maximum,
            center: None,
        });
    };
    let mut radius = 0.0_f64;
    for value in values {
        let distance = if *value >= center {
            *value - center
        } else {
            center - *value
        };
        if !distance.is_finite() {
            return Err(Error::Data(format!(
                "heatmap: cannot form a finite symmetric diverging domain around center {center}; set explicit color limits containing the center"
            )));
        }
        radius = radius.max(distance);
    }
    if radius == 0.0 {
        return Ok(ColorDomain {
            minimum: center,
            maximum: center,
            center: Some(center),
        });
    }
    let minimum = center - radius;
    let maximum = center + radius;
    if !minimum.is_finite() || !maximum.is_finite() || minimum >= center || maximum <= center {
        return Err(Error::Data(format!(
            "heatmap: symmetric diverging domain around center {center} exceeds finite f64 bounds; set explicit color limits containing the center"
        )));
    }
    Ok(ColorDomain {
        minimum,
        maximum,
        center: Some(center),
    })
}

fn draw_cells(
    chart: &Plot,
    prepared: &HeatmapData,
    layout: &Layout,
    theme: &ResolvedTheme,
    annotations: &AnnotationPlans,
    text: &mut crate::spec::ProducerText<'_>,
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    let x_count = prepared.x_levels.len();
    let y_count = prepared.y_levels.len();
    for y_index in 0..y_count {
        let y0 = y_count as f64 - y_index as f64 - 1.0;
        let y1 = y0 + 1.0;
        if let Some(annotation) = &annotations.rows[y_index] {
            let (left, bottom) = layout.projection.project(0.0, y0);
            let (right, top) = layout.projection.project(x_count as f64, y1);
            backend.draw_polygon(
                &[
                    Point2D::new(left, bottom),
                    Point2D::new(right, bottom),
                    Point2D::new(right, top),
                    Point2D::new(left, top),
                ],
                &FillStyle {
                    color: MISSING_FILL,
                    stroke: (255, 255, 255),
                    stroke_width: 0.5,
                    opacity: 1.0,
                },
            )?;
            text.draw(
                PlacedText {
                    plan: annotation.clone(),
                    placement: crate::render::backend::PlannedTextPlacement::middle(
                        Point2D::new(
                            f64::midpoint(left, right).round(),
                            f64::midpoint(bottom, top).round(),
                        ),
                        TextAnchor::Middle,
                        0.0,
                    ),
                },
                backend,
            )?;
            continue;
        }
        for x_index in 0..x_count {
            if !active_cell(chart.heatmap_layout_spec(), x_index, y_index) {
                continue;
            }
            let x0 = x_index as f64;
            let x1 = x0 + 1.0;
            let (left, bottom) = layout.projection.project(x0, y0);
            let (right, top) = layout.projection.project(x1, y1);
            let value = prepared.cells[y_index * x_count + x_index];
            let highlighted = prepared.highlighted[y_index * x_count + x_index];
            let color = value.map_or(MISSING_FILL, |value| heatmap_color(value, prepared.domain));
            backend.draw_polygon(
                &[
                    Point2D::new(left, bottom),
                    Point2D::new(right, bottom),
                    Point2D::new(right, top),
                    Point2D::new(left, top),
                ],
                &FillStyle {
                    color,
                    stroke: if highlighted {
                        theme.foreground
                    } else {
                        (255, 255, 255)
                    },
                    stroke_width: if highlighted { 2.5 } else { 0.5 },
                    opacity: 1.0,
                },
            )?;
            if value.is_some() {
                if let Some(plan) = &annotations.values[y_index * x_count + x_index] {
                    text.draw(
                        PlacedText::middle(
                            plan.clone(),
                            Point2D::new(
                                f64::midpoint(left, right).round(),
                                f64::midpoint(bottom, top).round(),
                            ),
                            TextAnchor::Middle,
                            0.0,
                        ),
                        backend,
                    )?;
                }
            } else {
                let inset = ((right - left).min(top - bottom) * 0.16).clamp(1.0, 4.0);
                let cross = LineStyle::solid(MISSING_STROKE, 1.0);
                backend.draw_line(
                    &[
                        Point2D::new(left + inset, bottom + inset),
                        Point2D::new(right - inset, top - inset),
                    ],
                    &cross,
                )?;
                backend.draw_line(
                    &[
                        Point2D::new(left + inset, top - inset),
                        Point2D::new(right - inset, bottom + inset),
                    ],
                    &cross,
                )?;
            }
        }
    }
    Ok(())
}

struct AnnotationPlans {
    values: Vec<Option<crate::layout::ResolvedTextPlan>>,
    rows: Vec<Option<crate::layout::ResolvedTextPlan>>,
}

fn plan_annotations(
    chart: &Plot,
    prepared: &HeatmapData,
    layout: &Layout,
    theme: &ResolvedTheme,
    scope: &crate::layout::fit::FitScope,
    text: &crate::spec::ProducerText<'_>,
) -> Result<AnnotationPlans, crate::layout::fit::PlanError> {
    use crate::layout::fit::{self, FitFailure, FitKind, FitOutcome};
    use crate::layout::AddressRole;
    let mut miss = None;
    let count = prepared.cells.len() + prepared.null_row_annotations.len();
    let environment = theme.prepared_font_environment()?;
    environment
        .claim_text_generation_slots(count as u128)
        .map_err(|error| Error::Render(format!("heatmap text slots: {error:?}")))?;
    environment
        .claim_unicode_bytes(
            (count * std::mem::size_of::<Option<crate::layout::ResolvedTextPlan>>()) as u64,
        )
        .map_err(|error| Error::Render(format!("heatmap text bytes: {error}")))?;
    let mut plans = AnnotationPlans {
        values: Vec::new(),
        rows: Vec::new(),
    };
    plans
        .values
        .try_reserve_exact(prepared.cells.len())
        .map_err(|_| Error::Render("heatmap value plans allocation failed".into()))?;
    plans
        .rows
        .try_reserve_exact(prepared.null_row_annotations.len())
        .map_err(|_| Error::Render("heatmap row plans allocation failed".into()))?;
    plans.values.resize(prepared.cells.len(), None);
    plans.rows.resize(prepared.null_row_annotations.len(), None);
    let cell_width = layout.projection.panel_box().width() / prepared.x_levels.len() as f64;
    let cell_height = layout.projection.panel_box().height() / prepared.y_levels.len() as f64;
    if let HeatmapAnnotations::Values(format) = chart.heatmap_annotations_spec() {
        for (index, value) in prepared.cells.iter().enumerate() {
            let Some(value) = value else { continue };
            let source = format.format(*value);
            let mut appearance =
                theme.legacy_text_appearance(ANNOTATION_FONT_SIZE, crate::TextWeight::Normal);
            appearance.paint = contrasting_text(heatmap_color(*value, prepared.domain)).into();
            let owner = format!("heatmap value annotation {index}");
            let plan = text.plan(&source, &owner, || {
                Ok(theme.measure_legacy_appearance_text(
                    &source,
                    &appearance,
                    PhysicalTextAlignment::Center,
                    &owner,
                    false,
                )?)
            })?;
            let width = plan.width();
            plans.values[index] = Some(plan);
            if cell_height < f64::from(ANNOTATION_FONT_SIZE) + 5.0 || cell_width < width + 6.0 {
                FitFailure::retain_least(&mut miss, scope.child(AddressRole::HeatmapValueLabel, index)?.failure(FitKind::Text, format!(
                "heatmap: cells are too small for value annotations at precision {} (cell {cell_width:.1}x{cell_height:.1}px, need at least {:.1}x{:.1}px); increase chart size, reduce precision, or hide annotations",
                format.configured_precision(),
                width + 6.0,
                f64::from(ANNOTATION_FONT_SIZE) + 5.0
                )));
            }
        }
    }
    let available_width = layout.projection.panel_box().width() - 12.0;
    let available_height = cell_height - 4.0;
    for (y_index, annotation) in prepared.null_row_annotations.iter().enumerate() {
        let Some(annotation) = annotation else {
            continue;
        };
        let row_scope = scope.child(AddressRole::HeatmapMergedRow, y_index)?;
        let block = match fit::outcome(text.plan(
            annotation,
            "heatmap fully-null row annotation",
            || {
                fit::measured(fit::text(
                    theme,
                    annotation,
                    &theme.legacy_text_appearance(ANNOTATION_FONT_SIZE, crate::TextWeight::Normal),
                    available_width,
                    PhysicalTextAlignment::Center,
                    "heatmap fully-null row annotation",
                    &row_scope,
                ))
            },
        ))? {
            FitOutcome::Measured(plan) => TextBlock::from_resolved(plan),
            FitOutcome::Miss(failure) => {
                FitFailure::retain_least(&mut miss, failure);
                continue;
            }
        };
        if block.width() > available_width || block.height() > available_height {
            FitFailure::retain_least(&mut miss, row_scope.failure(FitKind::Text, format!(
                "heatmap: wrapped fully-null row annotation for {:?} does not fit its merged row ({available_width:.1}x{available_height:.1}px available, {:.1}x{:.1}px needed); increase chart size or shorten the annotation",
                prepared.y_levels[y_index],
                block.width(),
                block.height()
            )));
        }
        plans.rows[y_index] = Some(block.plan().clone());
    }
    match miss {
        Some(failure) => Err(failure.into()),
        None => Ok(plans),
    }
}

fn draw_colorbar(
    prepared: &HeatmapData,
    layout: &Layout,
    color_label: &crate::layout::TextBlock,
    values: &[crate::layout::ResolvedTextPlan],
    text: &mut crate::spec::ProducerText<'_>,
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    let (_, panel_right, panel_bottom, panel_top) = layout.projection.pixel_box();
    let left = panel_right + 18.0;
    let available_height = panel_top - panel_bottom;
    let height = if prepared.has_missing {
        (available_height - 34.0).clamp(40.0, 160.0)
    } else {
        available_height.clamp(40.0, 180.0)
    };
    let bottom = panel_bottom + (available_height - height) / 2.0;
    let stops = colorbar_gradient_stops(prepared.domain);
    backend.draw_linear_gradient(
        PanelBox::new(left, left + 12.0, bottom, bottom + height),
        Point2D::new(left + 6.0, bottom),
        Point2D::new(left + 6.0, bottom + height),
        &stops,
    )?;
    text.draw(
        PlacedText::from_deferred_first_line(
            values[1].clone(),
            Point2D::new(
                (left + 18.0).round(),
                (bottom + height - 4.0).max(0.0).round(),
            ),
            TextAnchor::Start,
            0.0,
        ),
        backend,
    )?;
    text.draw(
        PlacedText::from_deferred_first_line(
            values[0].clone(),
            Point2D::new((left + 18.0).round(), (bottom - 12.0).max(0.0).round()),
            TextAnchor::Start,
            0.0,
        ),
        backend,
    )?;
    if let Some(center) = prepared.domain.center {
        if center > prepared.domain.minimum && center < prepared.domain.maximum {
            text.draw(
                PlacedText::from_deferred_first_line(
                    values[2].clone(),
                    Point2D::new(
                        (left + 18.0).round(),
                        (bottom
                            + height
                                * normalized_fraction(
                                    center,
                                    prepared.domain.minimum,
                                    prepared.domain.maximum,
                                ))
                        .max(0.0)
                        .round(),
                    ),
                    TextAnchor::Start,
                    0.0,
                ),
                backend,
            )?;
        }
    }
    text.draw(
        PlacedText::from_deferred_first_line(
            color_label.plan().clone(),
            Point2D::new((left + 55.0).round(), (bottom + height / 2.0).round()),
            TextAnchor::Middle,
            -90.0,
        ),
        backend,
    )?;

    if prepared.has_missing {
        let swatch_bottom = (bottom - 32.0).max(3.0);
        backend.draw_polygon(
            &[
                Point2D::new(left, swatch_bottom),
                Point2D::new(left + 12.0, swatch_bottom),
                Point2D::new(left + 12.0, swatch_bottom + 12.0),
                Point2D::new(left, swatch_bottom + 12.0),
            ],
            &FillStyle {
                color: MISSING_FILL,
                stroke: MISSING_STROKE,
                stroke_width: 0.8,
                opacity: 1.0,
            },
        )?;
        text.draw(
            PlacedText::from_deferred_first_line(
                values
                    .last()
                    .ok_or_else(|| Error::Render("missing colorbar text plan".into()))?
                    .clone(),
                Point2D::new((left + 18.0).round(), (swatch_bottom + 6.0).round()),
                TextAnchor::Start,
                0.0,
            ),
            backend,
        )?;
    }
    Ok(())
}

fn heatmap_color(value: f64, domain: ColorDomain) -> (u8, u8, u8) {
    if let Some(center) = domain.center {
        if same_value(domain.minimum, domain.maximum) {
            return (247, 247, 247);
        }
        if same_value(value, center) {
            return (247, 247, 247);
        }
        if value < center {
            if same_value(domain.minimum, center) {
                return (247, 247, 247);
            }
            let t = normalized_fraction(value, domain.minimum, center);
            interpolate((49, 130, 189), (247, 247, 247), t)
        } else {
            if same_value(domain.maximum, center) {
                return (247, 247, 247);
            }
            let t = normalized_fraction(value, center, domain.maximum);
            interpolate((247, 247, 247), (222, 45, 38), t)
        }
    } else {
        let t = if same_value(domain.minimum, domain.maximum) {
            0.5
        } else {
            normalized_fraction(value, domain.minimum, domain.maximum)
        };
        interpolate((247, 251, 255), (8, 48, 107), t)
    }
}

fn colorbar_gradient_stops(domain: ColorDomain) -> Vec<GradientStop> {
    let mut stops = vec![GradientStop::new(
        0.0,
        heatmap_color(domain.minimum, domain),
    )];
    if let Some(center) = domain
        .center
        .filter(|center| *center > domain.minimum && *center < domain.maximum)
    {
        let offset = normalized_fraction(center, domain.minimum, domain.maximum);
        if offset.is_finite() && offset > 0.0 && offset < 1.0 {
            stops.push(GradientStop::new(offset, heatmap_color(center, domain)));
        }
    }
    stops.push(GradientStop::new(
        1.0,
        heatmap_color(domain.maximum, domain),
    ));
    stops
}

fn normalized_fraction(value: f64, minimum: f64, maximum: f64) -> f64 {
    if value <= minimum {
        return 0.0;
    }
    if value >= maximum {
        return 1.0;
    }
    let scale = value.abs().max(minimum.abs()).max(maximum.abs());
    if scale.to_bits() == 0 || same_value(minimum, maximum) {
        return 0.5;
    }
    ((value / scale - minimum / scale) / (maximum / scale - minimum / scale)).clamp(0.0, 1.0)
}

fn interpolate(low: (u8, u8, u8), high: (u8, u8, u8), t: f64) -> (u8, u8, u8) {
    let channel =
        |low: u8, high: u8| (f64::from(low) + (f64::from(high) - f64::from(low)) * t).round() as u8;
    (
        channel(low.0, high.0),
        channel(low.1, high.1),
        channel(low.2, high.2),
    )
}

fn same_value(left: f64, right: f64) -> bool {
    left.total_cmp(&right).is_eq() || (left.abs().to_bits() == 0 && right.abs().to_bits() == 0)
}

fn contrasting_text(color: (u8, u8, u8)) -> (u8, u8, u8) {
    let luminance =
        0.2126 * f64::from(color.0) + 0.7152 * f64::from(color.1) + 0.0722 * f64::from(color.2);
    if luminance < 145.0 {
        (255, 255, 255)
    } else {
        (0, 0, 0)
    }
}

fn format_colorbar_value(value: f64) -> String {
    ColorbarValue(value).to_string()
}

struct ColorbarValue(f64);

impl std::fmt::Display for ColorbarValue {
    fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write as _;
        let value = if self.0.abs().to_bits() == 0 {
            0.0
        } else {
            self.0
        };
        if value.abs() >= 1_000_000.0 || (value.abs().to_bits() != 0 && value.abs() < 0.0001) {
            write!(output, "{value:.3e}")
        } else {
            // Fixed notation is below one million: sign, seven rounded integer
            // digits, decimal point, and six decimals fit without heap storage.
            struct Decimal {
                bytes: [u8; 32],
                length: usize,
            }
            impl std::fmt::Write for Decimal {
                fn write_str(&mut self, text: &str) -> std::fmt::Result {
                    let end = self.length.checked_add(text.len()).ok_or(std::fmt::Error)?;
                    self.bytes
                        .get_mut(self.length..end)
                        .ok_or(std::fmt::Error)?
                        .copy_from_slice(text.as_bytes());
                    self.length = end;
                    Ok(())
                }
            }
            let mut decimal = Decimal {
                bytes: [0; 32],
                length: 0,
            };
            write!(&mut decimal, "{value:.6}")?;
            let text = std::str::from_utf8(&decimal.bytes[..decimal.length])
                .map_err(|_| std::fmt::Error)?;
            output.write_str(text.trim_end_matches('0').trim_end_matches('.'))
        }
    }
}

fn fitted_label(theme: &ResolvedTheme, label: &str, maximum_width: f64) -> Result<String, Error> {
    const ELLIPSIS: &str = "...";

    let text_width = |text: &str| {
        theme
            .measure_text(
                text,
                &theme.typography.tick_label,
                PhysicalTextAlignment::End,
                "heatmap category label",
                false,
            )
            .map(|plan| plan.fit_width())
    };
    if text_width(label)? <= maximum_width {
        return Ok(label.to_string());
    }
    if text_width(ELLIPSIS)? > maximum_width {
        return Ok(String::new());
    }
    let mut fitted = String::new();
    for grapheme in label.graphemes(true) {
        let mut candidate = fitted.clone();
        candidate.push_str(grapheme);
        candidate.push_str(ELLIPSIS);
        #[cfg(test)]
        crate::orchestrator::measurements::count(
            crate::orchestrator::measurements::Counter::HeatmapPrefixTrial,
        );
        if text_width(&candidate)? > maximum_width {
            break;
        }
        fitted.push_str(grapheme);
    }
    fitted.push_str(ELLIPSIS);
    Ok(fitted)
}

fn fitted_unique_labels(
    theme: &ResolvedTheme,
    labels: &[String],
    maximum_width: f64,
    axis: &str,
    scope: &crate::layout::fit::FitScope,
) -> Result<Vec<String>, crate::layout::fit::PlanError> {
    let fitted = labels
        .iter()
        .map(|label| fitted_label(theme, label, maximum_width))
        .collect::<Result<Vec<_>, _>>()?;
    if fitted.iter().any(String::is_empty) {
        return Err(scope.failure(crate::layout::fit::FitKind::Tick, format!(
            "heatmap: {axis}-axis category labels have no room for an ellipsis; increase chart size or shorten labels"
        )).into());
    }
    let unique = fitted.iter().collect::<HashSet<_>>();
    if unique.len() != fitted.len() {
        return Err(scope.failure(crate::layout::fit::FitKind::Tick, format!(
            "heatmap: {axis}-axis category labels become ambiguous after elision; increase chart size or shorten labels"
        )).into());
    }
    Ok(fitted)
}

fn row_y(index: usize, count: usize) -> f64 {
    count as f64 - index as f64 - 0.5
}

#[cfg(test)]
mod tests;
