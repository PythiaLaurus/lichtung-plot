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

use crate::layout::{AxisSpec, AxisTick, Layout, PanelBox};
use crate::render::backend::{
    with_clip, Backend, FillStyle, GradientStop, LineStyle, Point2D, TextAnchor, TextStyle,
};
use crate::spec::{
    CategoryOrder, HeatmapAnnotations, HeatmapLabelFit, HeatmapLayout, HeatmapScale, LegendPolicy,
    PlotRenderContext, PlotRenderResult,
};
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

pub(crate) fn render(
    chart: &Plot,
    context: PlotRenderContext<'_>,
    backend: &mut dyn Backend,
) -> Result<PlotRenderResult, Error> {
    let prepared = prepare(chart, context.data)?;
    let x_labels = chart.resolve_label_domain(
        "heatmap x-axis categories",
        prepared.x_levels.iter().map(String::as_str),
    )?;
    let y_labels = chart.resolve_label_domain(
        "heatmap y-axis categories",
        prepared.y_levels.iter().map(String::as_str),
    )?;
    let color_label = chart.resolved_color_label();
    let detail_font_size = chart.palette().detail_font_size;
    let color_label_block = crate::layout::TextBlock::layout(
        &color_label,
        detail_font_size,
        (context.panel.height().min(170.0) * 0.9).max(1.0),
        "heatmap colorbar title",
    )?;
    let mut colorbar_values = vec![
        format_colorbar_value(prepared.domain.minimum),
        format_colorbar_value(prepared.domain.maximum),
    ];
    if let Some(center) = prepared.domain.center {
        colorbar_values.push(format_colorbar_value(center));
    }
    if prepared.has_missing {
        colorbar_values.push("Missing".to_string());
    }
    let value_width = colorbar_values
        .iter()
        .map(|label| crate::layout::estimated_text_width(label, detail_font_size))
        .fold(0.0_f64, f64::max);
    let colorbar_reserve = (36.0 + value_width).max(
        73.0 + color_label_block.lines().len().saturating_sub(1) as f64
            * color_label_block.line_height()
            + f64::from(detail_font_size) / 2.0,
    ) + 4.0;
    let wrap_labels = chart.heatmap_label_fit_spec() == HeatmapLabelFit::Wrap;
    let mut panel = if wrap_labels {
        crate::plots::horizontal_bar::panel_box_for_y_tick_labels(chart, context.panel, &y_labels)?
    } else {
        crate::plots::horizontal_bar::panel_box_for_elided_y_tick_labels(
            chart,
            context.panel,
            &y_labels,
        )?
    };
    if panel.width() - colorbar_reserve < 40.0 {
        return Err(Error::Render(
            "heatmap: colorbar title does not fit while preserving the minimum 40px plot panel; increase chart width"
                .into(),
        ));
    }
    panel.x1 -= colorbar_reserve;
    let minimum_panel_height = if prepared.has_missing { 90.0 } else { 60.0 };
    if panel.width() < 40.0
        || panel.height() < minimum_panel_height
        || panel.x1 + colorbar_reserve > f64::from(chart.width())
    {
        return Err(Error::Render(
            "heatmap: canvas is too small after reserving categorical labels and the colorbar; increase chart size"
                .into(),
        ));
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
            &y_labels,
            (panel.x0 - 38.0).max(20.0),
            chart.palette().tick_font_size,
            "y",
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
    );
    validate_annotation_fit(chart, &prepared, &layout)?;
    let x_labels = if wrap_labels {
        x_labels
    } else {
        fitted_unique_labels(
            &x_labels,
            panel.width() / x_count as f64 * 0.9,
            chart.palette().tick_font_size,
            "x",
        )?
    };
    for (tick, label) in layout.x_axis.ticks.iter_mut().zip(x_labels) {
        tick.label = label;
    }

    with_clip(backend, panel, |backend| {
        draw_cells(chart, &prepared, &layout, backend)
    })?;
    draw_colorbar(chart, &prepared, &layout, &color_label_block, backend)?;

    Ok(PlotRenderResult::new(layout, Vec::new())
        .with_resolved_axis_keys(prepared.x_levels.clone(), prepared.y_levels.clone()))
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
    let (x_levels, y_levels) = match heatmap_layout {
        HeatmapLayout::Cartesian => (
            resolve_axis_order(
                &crate::data::stable_levels(&x_values),
                chart.x_category_order_spec(),
                "x",
                !data.is_full_plot(),
            )?,
            resolve_axis_order(
                &crate::data::stable_levels(&y_values),
                chart.y_category_order_spec(),
                "y",
                !data.is_full_plot(),
            )?,
        ),
        HeatmapLayout::LowerTriangle { .. } | HeatmapLayout::UpperTriangle { .. } => {
            let observed = triangular_family_order(&x_values, &y_values, heatmap_layout);
            let family = resolve_triangular_order(chart, &observed, !data.is_full_plot())?;
            (family.clone(), family)
        }
    };
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
            "heatmap: categorical axis limits and tick-count overrides are not supported".into(),
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
    crate::data::strict_strings(data, column, "heatmap")?
        .into_iter()
        .enumerate()
        .map(|(row, value)| {
            let normalized = crate::data::collapse_whitespace(&value);
            if normalized.is_empty() {
                let source_row = data.source_row(row);
                Err(Error::Data(format!(
                    "heatmap: {axis} category label is empty at source row {source_row}"
                )))
            } else {
                Ok(normalized)
            }
        })
        .collect()
}

fn resolve_axis_order(
    observed: &[String],
    order: &CategoryOrder,
    axis: &str,
    is_subset: bool,
) -> Result<Vec<String>, Error> {
    match order {
        CategoryOrder::Observed => Ok(observed.to_vec()),
        CategoryOrder::Explicit(requested) => {
            let requested = requested
                .iter()
                .map(|level| crate::data::collapse_whitespace(level))
                .collect::<Vec<_>>();
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
) -> Result<Vec<String>, Error> {
    match (chart.x_category_order_spec(), chart.y_category_order_spec()) {
        (CategoryOrder::Observed, CategoryOrder::Observed) => Ok(observed.to_vec()),
        (CategoryOrder::Observed, y) => resolve_axis_order(observed, y, "y", is_subset),
        (x, CategoryOrder::Observed) => resolve_axis_order(observed, x, "x", is_subset),
        (x, y) => {
            let x = resolve_axis_order(observed, x, "x", is_subset)?;
            let y = resolve_axis_order(observed, y, "y", is_subset)?;
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
) -> Vec<String> {
    let mut levels = Vec::new();
    for (x, y) in x_values.iter().zip(y_values) {
        let pair = match layout {
            HeatmapLayout::UpperTriangle { .. } => [y, x],
            HeatmapLayout::LowerTriangle { .. } | HeatmapLayout::Cartesian => [x, y],
        };
        for value in pair {
            if !levels.contains(value) {
                levels.push(value.clone());
            }
        }
    }
    levels
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
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    let x_count = prepared.x_levels.len();
    let y_count = prepared.y_levels.len();
    let annotation_format = match chart.heatmap_annotations_spec() {
        HeatmapAnnotations::Hidden => None,
        HeatmapAnnotations::Values(format) => Some(format),
    };
    for y_index in 0..y_count {
        let y0 = y_count as f64 - y_index as f64 - 1.0;
        let y1 = y0 + 1.0;
        if let Some(annotation) = &prepared.null_row_annotations[y_index] {
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
            draw_null_row_annotation(chart, annotation, left, right, bottom, top, backend)?;
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
                        chart.palette().foreground
                    } else {
                        (255, 255, 255)
                    },
                    stroke_width: if highlighted { 2.5 } else { 0.5 },
                    opacity: 1.0,
                },
            )?;
            if let Some(value) = value {
                if let Some(format) = annotation_format {
                    let style = TextStyle {
                        color: contrasting_text(color),
                        font_family: chart.palette().font_family,
                        font_size: ANNOTATION_FONT_SIZE,
                        weight: crate::TextWeight::Normal,
                        anchor: TextAnchor::Middle,
                        rotation: 0.0,
                    };
                    backend.draw_text(
                        f64::midpoint(left, right).round() as u32,
                        f64::midpoint(bottom, top).round() as u32,
                        &format.format(value),
                        &style,
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

fn draw_null_row_annotation(
    chart: &Plot,
    annotation: &str,
    left: f64,
    right: f64,
    bottom: f64,
    top: f64,
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    let block = crate::layout::TextBlock::layout(
        annotation,
        ANNOTATION_FONT_SIZE,
        right - left - 12.0,
        "heatmap fully-null row annotation",
    )?;
    let style = TextStyle {
        color: chart.palette().foreground,
        font_family: chart.palette().font_family,
        font_size: ANNOTATION_FONT_SIZE,
        weight: crate::TextWeight::Normal,
        anchor: TextAnchor::Middle,
        rotation: 0.0,
    };
    let first_y = f64::midpoint(bottom, top)
        + block.lines().len().saturating_sub(1) as f64 * block.line_height() / 2.0;
    for (index, line) in block.lines().iter().enumerate() {
        backend.draw_text(
            f64::midpoint(left, right).round() as u32,
            (first_y - index as f64 * block.line_height()).round() as u32,
            line,
            &style,
        )?;
    }
    Ok(())
}

fn validate_annotation_fit(
    chart: &Plot,
    prepared: &HeatmapData,
    layout: &Layout,
) -> Result<(), Error> {
    let cell_width = layout.projection.panel_box().width() / prepared.x_levels.len() as f64;
    let cell_height = layout.projection.panel_box().height() / prepared.y_levels.len() as f64;
    if let HeatmapAnnotations::Values(format) = chart.heatmap_annotations_spec() {
        let widest = prepared
            .cells
            .iter()
            .flatten()
            .map(|value| {
                crate::layout::estimated_text_width(&format.format(*value), ANNOTATION_FONT_SIZE)
            })
            .fold(0.0_f64, f64::max);
        if cell_height < f64::from(ANNOTATION_FONT_SIZE) + 5.0 || cell_width < widest + 6.0 {
            return Err(Error::Render(format!(
                "heatmap: cells are too small for value annotations at precision {} (cell {cell_width:.1}x{cell_height:.1}px, need at least {:.1}x{:.1}px); increase chart size, reduce precision, or hide annotations",
                format.configured_precision(),
                widest + 6.0,
                f64::from(ANNOTATION_FONT_SIZE) + 5.0
            )));
        }
    }
    let available_width = layout.projection.panel_box().width() - 12.0;
    let available_height = cell_height - 4.0;
    for (y_index, annotation) in prepared.null_row_annotations.iter().enumerate() {
        let Some(annotation) = annotation else {
            continue;
        };
        let block = crate::layout::TextBlock::layout(
            annotation,
            ANNOTATION_FONT_SIZE,
            available_width,
            "heatmap fully-null row annotation",
        )?;
        if block.width() > available_width || block.height() > available_height {
            return Err(Error::Render(format!(
                "heatmap: wrapped fully-null row annotation for {:?} does not fit its merged row ({available_width:.1}x{available_height:.1}px available, {:.1}x{:.1}px needed); increase chart size or shorten the annotation",
                prepared.y_levels[y_index],
                block.width(),
                block.height()
            )));
        }
    }
    Ok(())
}

fn draw_colorbar(
    chart: &Plot,
    prepared: &HeatmapData,
    layout: &Layout,
    color_label: &crate::layout::TextBlock,
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
    let palette = chart.palette();
    let value_style = TextStyle {
        color: palette.foreground,
        font_family: palette.font_family.clone(),
        font_size: palette.detail_font_size,
        weight: crate::TextWeight::Normal,
        anchor: TextAnchor::Start,
        rotation: 0.0,
    };
    backend.draw_text(
        (left + 18.0).round() as u32,
        (bottom + height - 4.0).max(0.0).round() as u32,
        &format_colorbar_value(prepared.domain.maximum),
        &value_style,
    )?;
    backend.draw_text(
        (left + 18.0).round() as u32,
        (bottom - 12.0).max(0.0).round() as u32,
        &format_colorbar_value(prepared.domain.minimum),
        &value_style,
    )?;
    if let Some(center) = prepared.domain.center {
        if center > prepared.domain.minimum && center < prepared.domain.maximum {
            backend.draw_text(
                (left + 18.0).round() as u32,
                (bottom
                    + height
                        * normalized_fraction(
                            center,
                            prepared.domain.minimum,
                            prepared.domain.maximum,
                        ))
                .max(0.0)
                .round() as u32,
                &format_colorbar_value(center),
                &value_style,
            )?;
        }
    }
    let title_style = TextStyle {
        anchor: TextAnchor::Middle,
        rotation: -90.0,
        ..value_style.clone()
    };
    for (index, line) in color_label.lines().iter().enumerate() {
        backend.draw_text(
            (left + 55.0 + index as f64 * color_label.line_height()).round() as u32,
            (bottom + height / 2.0).round() as u32,
            line,
            &title_style,
        )?;
    }

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
        backend.draw_text(
            (left + 18.0).round() as u32,
            (swatch_bottom + 6.0).round() as u32,
            "Missing",
            &value_style,
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
    let value = if value.abs().to_bits() == 0 {
        0.0
    } else {
        value
    };
    if value.abs() >= 1_000_000.0 || (value.abs().to_bits() != 0 && value.abs() < 0.0001) {
        format!("{value:.3e}")
    } else {
        format!("{value:.6}")
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}

fn fitted_label(label: &str, maximum_width: f64, font_size: u32) -> String {
    const ELLIPSIS: &str = "...";

    let text_width =
        |text: &str| crate::layout::deterministic_text_width(text, f64::from(font_size));
    if text_width(label) <= maximum_width {
        return label.to_string();
    }
    if text_width(ELLIPSIS) > maximum_width {
        return String::new();
    }
    let mut fitted = String::new();
    for grapheme in label.graphemes(true) {
        let mut candidate = fitted.clone();
        candidate.push_str(grapheme);
        candidate.push_str(ELLIPSIS);
        if text_width(&candidate) > maximum_width {
            break;
        }
        fitted.push_str(grapheme);
    }
    fitted.push_str(ELLIPSIS);
    fitted
}

fn fitted_unique_labels(
    labels: &[String],
    maximum_width: f64,
    font_size: u32,
    axis: &str,
) -> Result<Vec<String>, Error> {
    let fitted = labels
        .iter()
        .map(|label| fitted_label(label, maximum_width, font_size))
        .collect::<Vec<_>>();
    if fitted.iter().any(String::is_empty) {
        return Err(Error::Render(format!(
            "heatmap: {axis}-axis category labels have no room for an ellipsis; increase chart size or shorten labels"
        )));
    }
    let unique = fitted.iter().collect::<HashSet<_>>();
    if unique.len() != fitted.len() {
        return Err(Error::Render(format!(
            "heatmap: {axis}-axis category labels become ambiguous after elision; increase chart size or shorten labels"
        )));
    }
    Ok(fitted)
}

fn row_y(index: usize, count: usize) -> f64 {
    count as f64 - index as f64 - 0.5
}

#[cfg(test)]
mod tests;
