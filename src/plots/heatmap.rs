//! Long-form categorical heatmap with a continuous colorbar.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::too_many_lines
)]

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
const MISSING_FILL: (u8, u8, u8) = (224, 224, 224);
const MISSING_STROKE: (u8, u8, u8) = (120, 120, 120);

pub(crate) fn missing_fill() -> crate::render::backend::Rgba {
    MISSING_FILL.into()
}

#[derive(Debug, PartialEq)]
pub(crate) struct HeatmapData {
    pub(crate) x_levels: Vec<String>,
    pub(crate) y_levels: Vec<String>,
    cells: Vec<Option<f64>>,
    fills: Vec<Option<(u8, u8, u8)>>,
    highlighted: Vec<bool>,
    null_row_annotations: Vec<Option<String>>,
    has_missing: bool,
    minimum: DeferredHeatmapMinimum,
    color: PreparedHeatmapColor,
    layout: HeatmapLayout,
    value_labels: Vec<Option<String>>,
    value_owners: Vec<Option<String>>,
    colorbar_values: Vec<String>,
    pub(crate) color_label: String,
}

impl HeatmapData {
    pub(crate) fn guide_semantics(
        &self,
        data: crate::data::PlotData<'_>,
    ) -> Result<crate::decorations::legend::descriptor::ContinuousSemantics, Error> {
        let domain = self.color.domain;
        let center = domain.center;
        let resources = data.source_resources()?;
        let palette = crate::decorations::legend::descriptor::Palette {
            low: self.color.scale.color(domain.minimum, resources)?,
            center: center
                .map(|value| self.color.scale.color(value, resources))
                .transpose()?,
            high: self.color.scale.color(domain.maximum, resources)?,
        };
        let missing = self
            .has_missing
            .then(|| -> Result<_, Error> {
                let label = self.colorbar_values.last().ok_or_else(|| {
                    Error::Render("heatmap missing key has no prepared label".into())
                })?;
                Ok(crate::decorations::legend::descriptor::MissingKey {
                    fill: MISSING_FILL,
                    outline: MISSING_STROKE,
                    label: data.copy_source_string(label)?,
                })
            })
            .transpose()?;
        Ok(
            crate::decorations::legend::descriptor::ContinuousSemantics {
                domain: (domain.minimum, domain.maximum),
                center,
                palette,
                missing,
            },
        )
    }

    pub(crate) fn install_shared_domain(
        &mut self,
        descriptor: &crate::decorations::legend::descriptor::ContinuousDescriptor,
        data: crate::data::PlotData<'_>,
    ) -> Result<(), Error> {
        self.replace_domain(
            ColorDomain {
                minimum: descriptor.domain.0,
                maximum: descriptor.domain.1,
                center: descriptor.center,
            },
            data,
        )
    }

    fn replace_domain(
        &mut self,
        domain: ColorDomain,
        data: crate::data::PlotData<'_>,
    ) -> Result<(), Error> {
        let color = PreparedHeatmapColor::resolve(domain, data.source_resources()?)?;
        let fills = data.collect_source(self.cells.iter().map(|value| {
            value
                .map(|value| color.scale.color(value, data.source_resources()?))
                .transpose()
        }))?;
        let colorbar_values = prepare_colorbar_values(domain, self.has_missing, data)?;
        self.color = color;
        self.fills = fills;
        self.colorbar_values = colorbar_values;
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct PreparedHeatmapInputs {
    pub(crate) panels: Vec<(Option<String>, HeatmapData)>,
}

fn retained(chart: &Plot) -> Result<&HeatmapData, Error> {
    chart
        .prepared_heatmap
        .as_ref()
        .and_then(|inputs| {
            inputs
                .panels
                .iter()
                .find(|(scope, _)| scope.as_deref() == chart.axis_scope())
        })
        .map(|(_, marks)| marks)
        .ok_or_else(|| Error::Render("heatmap requires root-prepared semantics".into()))
}

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

#[derive(Debug, Clone, Copy, PartialEq)]
struct ColorDomain {
    minimum: f64,
    maximum: f64,
    center: Option<f64>,
}

#[derive(Debug, PartialEq)]
struct PreparedHeatmapColor {
    domain: ColorDomain,
    scale: crate::theme::ContinuousColors,
}

impl PreparedHeatmapColor {
    fn resolve(
        domain: ColorDomain,
        resources: &crate::font::canonical::OperationGenerationResources,
    ) -> Result<Self, Error> {
        let low = (49, 130, 189);
        let neutral = (247, 247, 247);
        let high = (222, 45, 38);
        let (stops, count) = if let Some(center) = domain.center {
            if !center.is_finite() || center < domain.minimum || center > domain.maximum {
                return Err(Error::Data(
                    "heatmap diverging domain must contain its finite center".into(),
                ));
            }
            if same_value(domain.minimum, domain.maximum) {
                (
                    [
                        GradientStop::new(0.0, neutral),
                        GradientStop::new(1.0, neutral),
                        GradientStop::new(1.0, neutral),
                    ],
                    2,
                )
            } else if same_value(center, domain.minimum) {
                (
                    [
                        GradientStop::new(0.0, neutral),
                        GradientStop::new(1.0, high),
                        GradientStop::new(1.0, high),
                    ],
                    2,
                )
            } else if same_value(center, domain.maximum) {
                (
                    [
                        GradientStop::new(0.0, low),
                        GradientStop::new(1.0, neutral),
                        GradientStop::new(1.0, neutral),
                    ],
                    2,
                )
            } else {
                let offset = crate::theme::ContinuousColors::normalized_fraction(
                    (domain.minimum, domain.maximum),
                    center,
                )?;
                if !(0.0..1.0).contains(&offset) || offset.to_bits() == 0 {
                    return Err(Error::Data(
                        "heatmap diverging center has no representable gradient stop".into(),
                    ));
                }
                (
                    [
                        GradientStop::new(0.0, low),
                        GradientStop::new(offset, neutral),
                        GradientStop::new(1.0, high),
                    ],
                    3,
                )
            }
        } else {
            (
                [
                    GradientStop::new(0.0, (247, 251, 255)),
                    GradientStop::new(1.0, (8, 48, 107)),
                    GradientStop::new(1.0, (8, 48, 107)),
                ],
                2,
            )
        };
        Ok(Self {
            domain,
            scale: crate::theme::ContinuousColors::resolve(
                (domain.minimum, domain.maximum),
                &stops[..count],
                resources,
            )?,
        })
    }

    fn guide_stops(&self, data: crate::data::PlotData<'_>) -> Result<Vec<GradientStop>, Error> {
        let color = if same_value(self.domain.minimum, self.domain.maximum) {
            Some(
                self.scale
                    .color(self.domain.minimum, data.source_resources()?)?,
            )
        } else {
            None
        };
        data.collect_source(
            self.scale
                .stops()
                .iter()
                .map(|stop| Ok(GradientStop::new(stop.offset, color.unwrap_or(stop.color)))),
        )
    }
}

/// The one prepared-axis fast path is usable only for the appearance and
/// continuous domain that actually selected the annotation glyph roles.
#[derive(Debug)]
pub(crate) struct HeatmapIntrinsicCache {
    minimum: (f64, f64),
    domain: ColorDomain,
    contrast_light: crate::theme::TextAppearance,
    contrast_dark: crate::theme::TextAppearance,
    plot_panel: crate::render::backend::Rgba,
    canvas_root: crate::render::backend::Rgba,
    canvas_override: Option<crate::render::backend::Rgba>,
    backdrop: crate::render::backend::Rgba,
    cell_outline: crate::theme::FillOutlineAppearance,
    highlight_outline: crate::theme::FillOutlineAppearance,
    missing_row_outline: crate::theme::FillOutlineAppearance,
    missing_cross: LineStyle,
}

impl HeatmapIntrinsicCache {
    fn matches(&self, prepared: &HeatmapData, theme: &ResolvedTheme) -> bool {
        self.domain == prepared.color.domain
            && self.contrast_light == theme.typography.contrast_light
            && self.contrast_dark == theme.typography.contrast_dark
            && self.plot_panel == theme.surfaces.plot_panel
            && self.canvas_root == theme.surfaces.canvas_root
            && self.canvas_override == theme.canvas_surface_override
            && self.backdrop == theme.annotations.planning_backdrop
            && self.cell_outline == theme.marks.heatmap_cell_outline
            && self.highlight_outline == theme.marks.heatmap_highlight_outline
            && self.missing_row_outline == theme.marks.heatmap_missing_row_outline
            && self.missing_cross == theme.marks.heatmap_missing_cross.backend_style()
    }

    fn new(
        minimum: (f64, f64),
        prepared: &HeatmapData,
        theme: &ResolvedTheme,
        data: crate::data::PlotData<'_>,
    ) -> Result<Self, Error> {
        data.claim_source_bytes(std::mem::size_of::<Self>())?;
        let text = |role: &crate::theme::TextAppearance| -> Result<_, Error> {
            Ok(crate::theme::TextAppearance {
                stack: data.copy_source_string(&role.stack)?,
                paint: role.paint,
                font_size: role.font_size,
                weight: role.weight,
                line_height_multiplier: role.line_height_multiplier,
            })
        };
        Ok(Self {
            minimum,
            domain: prepared.color.domain,
            contrast_light: text(&theme.typography.contrast_light)?,
            contrast_dark: text(&theme.typography.contrast_dark)?,
            plot_panel: theme.surfaces.plot_panel,
            canvas_root: theme.surfaces.canvas_root,
            canvas_override: theme.canvas_surface_override,
            backdrop: theme.annotations.planning_backdrop,
            cell_outline: theme.marks.heatmap_cell_outline.clone(),
            highlight_outline: theme.marks.heatmap_highlight_outline.clone(),
            missing_row_outline: theme.marks.heatmap_missing_row_outline.clone(),
            missing_cross: theme.marks.heatmap_missing_cross.backend_style(),
        })
    }
}

/// Current specialized packing floor, not T5e category-pitch intrinsics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct DeferredHeatmapMinimum {
    missing: bool,
}

impl DeferredHeatmapMinimum {
    const WIDTH: f64 = 40.0;

    pub(crate) const fn body(self) -> (f64, f64) {
        (Self::WIDTH, if self.missing { 90.0 } else { 60.0 })
    }
}

/// Hard consequences of the current cell-label algorithm, before a colorbar,
/// frame, or cell is assigned. Category packing and elision remain T5e.
pub(crate) fn intrinsic_body_minimum(
    chart: &Plot,
    theme: &ResolvedTheme,
) -> Result<(f64, f64), Error> {
    let axes = chart
        .prepared_axis_inputs()
        .ok_or_else(|| Error::Render("heatmap intrinsics require root-prepared axes".into()))?;
    prepare_intrinsics(
        retained(chart)?,
        axes,
        chart.data(),
        theme,
        chart.axis_scope(),
    )
}

fn intrinsic_cell_ink(
    prepared: &HeatmapData,
    theme: &ResolvedTheme,
    data: crate::data::PlotData<'_>,
) -> Result<(f64, f64), Error> {
    let (columns, rows) = (prepared.x_levels.len(), prepared.y_levels.len());
    let slots = columns.checked_mul(rows).ok_or_else(|| {
        Error::Render("heatmap declared cell topology exceeds spatial capacity".into())
    })?;
    if columns == 0
        || rows == 0
        || prepared.highlighted.len() != prepared.cells.len()
        || prepared.null_row_annotations.len() != rows
        || prepared.cells.len() != slots
    {
        return Err(Error::Render(
            "heatmap cell topology differs from prepared category tracks".into(),
        ));
    }
    let quantum = 1.0 / 1024.0;
    let cell = |style: &crate::theme::FillOutlineAppearance| -> Result<_, Error> {
        let fill = style.backend_style((0, 0, 0).into());
        let (x, y) = crate::render::geometry::rectangular_fill_ink_outset(theme, &fill)?;
        Ok((x + quantum, y + quantum))
    };
    let mut normal = None;
    let mut highlight = None;
    let mut missing_row = None;
    let mut cross = None;
    let mut pitch = (quantum, quantum);
    data.claim_source_visits(prepared.cells.len())?;
    for y in 0..rows {
        if prepared.null_row_annotations[y].is_some() {
            let ink = if let Some(ink) = missing_row {
                ink
            } else {
                let ink = cell(&theme.marks.heatmap_missing_row_outline)?;
                missing_row = Some(ink);
                ink
            };
            pitch.0 = pitch.0.max(ink.0);
            pitch.1 = pitch.1.max(ink.1);
        }
        for x in 0..columns {
            if !active_cell(prepared.layout, x, y) {
                continue;
            }
            if prepared.null_row_annotations[y].is_some() {
                continue;
            }
            let index = y * columns + x;
            let ink = if prepared.highlighted[index] {
                if let Some(ink) = highlight {
                    ink
                } else {
                    let ink = cell(&theme.marks.heatmap_highlight_outline)?;
                    highlight = Some(ink);
                    ink
                }
            } else if let Some(ink) = normal {
                ink
            } else {
                let ink = cell(&theme.marks.heatmap_cell_outline)?;
                normal = Some(ink);
                ink
            };
            pitch.0 = pitch.0.max(ink.0);
            pitch.1 = pitch.1.max(ink.1);
            if prepared.cells[index].is_none() {
                let cross = if let Some(stroke) = cross {
                    stroke
                } else {
                    let stroke = crate::render::geometry::line_ink_width(
                        theme,
                        &theme.marks.heatmap_missing_cross.backend_style(),
                    )?;
                    cross = Some(stroke);
                    stroke
                };
                let span = (2.0_f64).max(cross + 2.0 * quantum) + quantum;
                pitch.0 = pitch.0.max(span);
                pitch.1 = pitch.1.max(span);
            }
        }
    }
    let columns = f64::from(
        u32::try_from(columns)
            .map_err(|_| Error::Render("heatmap column count exceeds spatial range".into()))?,
    );
    let rows = f64::from(
        u32::try_from(rows)
            .map_err(|_| Error::Render("heatmap row count exceeds spatial range".into()))?,
    );
    let result = (pitch.0 * columns, pitch.1 * rows);
    if !result.0.is_finite() || !result.1.is_finite() {
        return Err(Error::Render(
            "heatmap painted cell hard span overflowed".into(),
        ));
    }
    Ok(result)
}

pub(crate) fn prepare_intrinsics(
    prepared: &HeatmapData,
    axes: &crate::plots::shared_x::PreparedAxes,
    data: crate::data::PlotData<'_>,
    theme: &ResolvedTheme,
    scope: Option<&str>,
) -> Result<(f64, f64), Error> {
    let environment = theme.prepared_font_environment()?;
    environment.claim_measurement_call().map_err(|error| {
        Error::Render(format!(
            "heatmap intrinsic call admission {scope:?}: {error:?}"
        ))
    })?;
    if let Some(cached) = axes.heatmap_intrinsics.get() {
        if cached.matches(prepared, theme) {
            return Ok(cached.minimum);
        }
    }
    #[cfg(test)]
    crate::orchestrator::measurements::intrinsic(
        crate::orchestrator::measurements::Counter::HeatmapIntrinsic,
        scope,
    );
    let mut minimum = prepared.minimum.body();
    let resources = environment.generation_resources();
    let data = data.with_axis_resources(&resources);
    data.claim_source_visits(prepared.value_labels.len() + prepared.null_row_annotations.len())?;
    for (source, fill) in prepared.annotation_labels() {
        let appearance = theme.contrasting_text(fill)?;
        let plan = theme.measure_legacy_appearance_text(
            source,
            appearance,
            PhysicalTextAlignment::Center,
            "heatmap value intrinsic",
            false,
        )?;
        minimum.0 = minimum
            .0
            .max((plan.width() + 6.0) * prepared.x_levels.len() as f64);
        minimum.1 = minimum
            .1
            .max((plan.height() + 5.0) * prepared.y_levels.len() as f64);
    }
    let appearance = theme.contrasting_text(missing_fill())?;
    for source in prepared.null_row_annotations.iter().flatten() {
        let hard = crate::layout::fit::intrinsic_widths(
            theme,
            source,
            appearance,
            "heatmap fully-null row annotation",
        )?
        .0;
        let unwrapped = theme.measure_legacy_appearance_text(
            source,
            appearance,
            PhysicalTextAlignment::Center,
            "heatmap fully-null row annotation",
            false,
        )?;
        minimum.0 = minimum.0.max(hard + 12.0);
        minimum.1 = minimum
            .1
            .max((unwrapped.height() + 4.0) * prepared.y_levels.len() as f64);
    }
    let cell = intrinsic_cell_ink(prepared, theme, data)?;
    minimum.0 = minimum.0.max(cell.0);
    minimum.1 = minimum.1.max(cell.1);
    if axes.heatmap_intrinsics.get().is_none() {
        environment.claim_cache_entries(1).map_err(|error| {
            Error::Render(format!("heatmap intrinsic cache admission: {error:?}"))
        })?;
        let cache = HeatmapIntrinsicCache::new(minimum, prepared, theme, data)?;
        let _ = axes.heatmap_intrinsics.set(cache);
    }
    Ok(minimum)
}

impl HeatmapData {
    pub(crate) fn guide_labels(&self) -> impl Iterator<Item = &str> {
        self.colorbar_values[..self.colorbar_values.len() - usize::from(self.has_missing)]
            .iter()
            .map(String::as_str)
    }

    pub(crate) fn missing_guide_label(&self) -> Option<&str> {
        self.has_missing
            .then(|| self.colorbar_values.last().map(String::as_str))
            .flatten()
    }

    pub(crate) fn annotation_labels(
        &self,
    ) -> impl Iterator<Item = (&str, crate::render::backend::Rgba)> {
        self.value_labels
            .iter()
            .zip(&self.fills)
            .filter_map(|(label, fill)| Some((label.as_deref()?, (*fill)?.into())))
    }
}

pub(crate) fn measure(
    chart: &Plot,
    context: PlotRenderContext<'_>,
) -> Result<PlotRenderResult, crate::layout::fit::PlanError> {
    #[cfg(test)]
    count(1);
    let prepared = retained(chart)?;
    let theme = context.theme;
    let resources = theme.prepared_font_environment()?.generation_resources();
    let data = context.data.with_axis_resources(&resources);
    let text = context.producer_text()?;
    let colorbar = if chart.continuous_guide_suppressed() {
        None
    } else {
        let color_label = &prepared.color_label;
        let guide_scope = context
            .fit_scope()
            .child(crate::layout::AddressRole::OutsideGuideRight, 0)?;
        let title_width = if prepared.has_missing {
            (context.panel.height() - 34.0).clamp(40.0, 160.0)
        } else {
            context.panel.height().clamp(40.0, 180.0)
        };
        let title =
            TextBlock::from_resolved(text.plan(color_label, "heatmap colorbar title", || {
                crate::layout::fit::measured(crate::layout::fit::text(
                    theme,
                    color_label,
                    &theme.typography.legend_title,
                    title_width,
                    PhysicalTextAlignment::Center,
                    "heatmap colorbar title",
                    &guide_scope.child(crate::layout::AddressRole::GuideTitle, 0)?,
                ))
            })?);
        let mut values = Vec::new();
        data.reserve_source(&mut values, prepared.colorbar_values.len())?;
        for label in &prepared.colorbar_values {
            values.push(text.plan(label, "heatmap colorbar value", || {
                Ok(theme.measure_legacy_appearance_text(
                    label,
                    &theme.typography.legend_label,
                    PhysicalTextAlignment::Start,
                    "heatmap colorbar value",
                    false,
                )?)
            })?);
        }
        let value_width = values
            .iter()
            .map(crate::layout::ResolvedTextPlan::width)
            .fold(0.0_f64, f64::max);
        let reserve = (36.0 + value_width).max(
            73.0 + title.lines().len().saturating_sub(1) as f64 * title.line_height()
                + f64::from(theme.typography.legend_label.font_size) / 2.0,
        ) + 4.0;
        Some((title, values, reserve))
    };
    let minimum = prepared.minimum;
    let colorbar_reserve = colorbar.as_ref().map_or(0.0, |(_, _, reserve)| *reserve);
    if context.panel.width() - colorbar_reserve < minimum.body().0 {
        let guide_scope = context
            .fit_scope()
            .child(crate::layout::AddressRole::OutsideGuideRight, 0)?;
        return Err(guide_scope.failure(crate::layout::fit::FitKind::LegendOrGuide, "heatmap: colorbar title does not fit while preserving the minimum 40px plot panel; increase chart width").into());
    }
    let mut body = context.panel;
    body.x1 -= colorbar_reserve;
    let mut annotations = None;
    let layout = context.axis_layout(|| -> Result<_, crate::layout::fit::PlanError> {
    let x_labels = data.collect_source(context.prepared_category_labels(0)
        .ok_or_else(|| Error::Render("heatmap x labels require prepared axes".into()))?
        .iter().map(|label| data.copy_source_string(label)))?;
    let y_labels = data.collect_source(context.prepared_category_labels(1)
        .ok_or_else(|| Error::Render("heatmap y labels require prepared axes".into()))?
        .iter().map(|label| data.copy_source_string(label)))?;
    let wrap_labels = chart.heatmap_label_fit_spec() == HeatmapLabelFit::Wrap;
    // Retain the legacy rotated/elided packing until T5e.
    let chrome = crate::layout::y_axis_left_inset_for_tick_labels(
        0.0,
        theme.axes.ticks.as_ref().map_or(0, |ticks| ticks.length),
        theme.typography.axis_title.font_size,
        &theme.axes.spacing,
    );
    let base_left = chart.legacy_root_axes().insets(theme, true).left.min(body.x0);
    let available = body.width() - minimum.body().0 + base_left - chrome;
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
    if panel.width() < minimum.body().0
        || panel.height() < minimum.body().1
        || panel.x1 + colorbar_reserve > f64::from(chart.width())
    {
        return Err(context.fit_scope().failure(crate::layout::fit::FitKind::SpecializedGeometry,
            "heatmap: canvas is too small after reserving categorical labels and the colorbar; increase chart size"
        ).into());
    }

    let x_count = prepared.x_levels.len();
    let y_count = prepared.y_levels.len();
    let x_ticks = data.collect_source(prepared
        .x_levels
        .iter()
        .enumerate()
        .map(|(index, _)| { data.claim_source_visits(1)?; Ok(AxisTick::new(index as f64 + 0.5, "")) }))?;
    let y_labels = if wrap_labels {
        y_labels
    } else {
        fitted_unique_labels(
            data,
            theme,
            &y_labels,
            tick_width,
            "y",
            &context
                .fit_scope()
                .child(crate::layout::AddressRole::AxisLeft, 0)?,
        )?
    };
    let y_ticks = data.collect_source(y_labels
        .into_iter()
        .enumerate()
        .map(|(index, label)| { data.claim_source_visits(1)?; Ok(AxisTick::new(row_y(index, y_count), label)) }))?;
    let x_spec = AxisSpec::default().limits(0.0, x_count as f64);
    let y_spec = AxisSpec::default().limits(0.0, y_count as f64);
        let mut layout = Layout::new_with_ticks_and_axis_specs_in_panel_box(
            (0.0, x_count as f64),
            (0.0, y_count as f64),
            panel,
            data.copy_source_string(chart.x_label_text().unwrap_or(""))?,
            data.copy_source_string(chart.y_label_text().unwrap_or(""))?,
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
            chart, prepared, &layout, theme, context.fit_scope(), &text,
        )?);
    let x_labels = if wrap_labels {
        x_labels
    } else {
        fitted_unique_labels(
            data,
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
        || plan_annotations(chart, prepared, &layout, theme, context.fit_scope(), &text),
        Ok,
    )?;

    let payload = HeatmapPayload {
        clip: panel,
        cross_style: theme.marks.heatmap_missing_cross.backend_style(),
        cells: select_cells(prepared, &layout, theme, &annotations, data)?,
        colorbar: colorbar
            .as_ref()
            .map(|(title, values, _)| {
                select_colorbar(prepared, &layout, title, values, theme, data)
            })
            .transpose()?,
    };
    payload.validate_annotation_ink(theme, context.fit_scope(), data)?;
    let geometry = payload.geometry(theme)?;

    let mut result = PlotRenderResult::new(layout, Vec::new()).with_resolved_axis_keys(
        data.collect_source(prepared.x_levels.iter().map(|s| data.copy_source_string(s)))?,
        data.collect_source(prepared.y_levels.iter().map(|s| data.copy_source_string(s)))?,
    );
    result.x_domain = crate::spec::AxisDomain::Categorical;
    result.y_domain = crate::spec::AxisDomain::Categorical;
    let texts = payload
        .cells
        .iter()
        .filter_map(|cell| cell.text.as_ref())
        .chain(payload.colorbar.iter().flat_map(|bar| &bar.text))
        .chain(
            payload
                .colorbar
                .iter()
                .filter_map(|bar| bar.missing.as_ref())
                .filter_map(|cell| cell.text.as_ref()),
        );
    for text in texts {
        data.reserve_source(&mut result.producer_text, 1)?;
        result.producer_text.push(text.clone());
    }
    result.marks = Some(data.source_arc(crate::layout::interior::SelectedMarkPlan {
        payload: crate::layout::interior::MarkPayload::Heatmap(payload),
        geometry,
    })?);
    Ok(result)
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

pub(crate) fn prepare<'a>(
    chart: &Plot,
    data: impl Into<crate::data::PlotData<'a>>,
) -> Result<HeatmapData, Error> {
    #[cfg(test)]
    count(0);
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

    let explicit_triangular_order = heatmap_layout != HeatmapLayout::Cartesian
        && (!matches!(chart.x_category_order_spec(), CategoryOrder::Observed)
            || !matches!(chart.y_category_order_spec(), CategoryOrder::Observed));
    let mut x_indices = data.collect_source(
        x_levels
            .iter()
            .enumerate()
            .map(|(index, level)| Ok((level.as_str(), index))),
    )?;
    let mut y_indices = data.collect_source(
        y_levels
            .iter()
            .enumerate()
            .map(|(index, level)| Ok((level.as_str(), index))),
    )?;
    crate::data::source::sort_source_by(&mut x_indices, Some(data), |left, right| {
        left.0.cmp(right.0)
    })?;
    crate::data::source::sort_source_by(&mut y_indices, Some(data), |left, right| {
        left.0.cmp(right.0)
    })?;
    let mut seen = data.collect_source((0..cell_count).map(|_| Ok(false)))?;
    let mut cells = data.collect_source((0..cell_count).map(|_| Ok(None)))?;
    let mut highlighted = data.collect_source((0..cell_count).map(|_| Ok(false)))?;
    data.claim_source_visits(x_values.len())?;
    for (row, ((x, y), fill)) in x_values.iter().zip(&y_values).zip(&fills).enumerate() {
        data.claim_source_visits(x_indices.len().ilog2() as usize + 2)?;
        data.claim_source_visits(y_indices.len().ilog2() as usize + 2)?;
        let mut x_index = x_indices[x_indices
            .binary_search_by_key(&x.as_str(), |entry| entry.0)
            .expect("observed x level")]
        .1;
        let mut y_index = y_indices[y_indices
            .binary_search_by_key(&y.as_str(), |entry| entry.0)
            .expect("observed y level")]
        .1;
        if explicit_triangular_order && !active_cell(heatmap_layout, x_index, y_index) {
            std::mem::swap(&mut x_index, &mut y_index);
        }
        let index = y_index * x_levels.len() + x_index;
        if std::mem::replace(&mut seen[index], true) {
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
        data,
    )?;

    let mut finite = Vec::new();
    data.reserve_source(&mut finite, fills.len())?;
    data.claim_source_visits(fills.len())?;
    finite.extend(fills.iter().flatten().copied());
    if finite.is_empty() && chart.heatmap_color_limits_spec().is_none() {
        return Err(Error::Data(
            "heatmap: color column must contain at least one finite value unless explicit color limits are set"
                .into(),
        ));
    }
    data.claim_source_visits(finite.len())?;
    data.claim_source_visits(finite.len())?;
    let domain = color_domain(
        &finite,
        chart.heatmap_scale_spec(),
        chart.heatmap_color_limits_spec(),
    )?;
    let color = PreparedHeatmapColor::resolve(domain, data.source_resources()?)?;
    let resolved_fills = data.collect_source(cells.iter().map(|value| {
        value
            .map(|value| color.scale.color(value, data.source_resources()?))
            .transpose()
    }))?;
    data.claim_source_visits(cells.len())?;
    let has_missing = cells.iter().enumerate().any(|(index, value)| {
        let x_index = index % x_levels.len();
        let y_index = index / x_levels.len();
        active_cell(heatmap_layout, x_index, y_index) && value.is_none()
    });

    let value_labels = data.collect_source(cells.iter().map(|value| {
        match (value, chart.heatmap_annotations_spec()) {
            (Some(value), HeatmapAnnotations::Values(format)) => {
                format.format_source(*value, data).map(Some)
            }
            _ => Ok(None),
        }
    }))?;
    let value_owners =
        data.collect_source(value_labels.iter().enumerate().map(|(index, label)| {
            use std::fmt::Write;
            if label.is_none() {
                return Ok(None);
            }
            let mut owner = String::new();
            data.reserve_source_string(&mut owner, 44)?;
            write!(owner, "heatmap value annotation {index}")
                .expect("reserved annotation identity");
            Ok(Some(owner))
        }))?;
    let colorbar_values = prepare_colorbar_values(domain, has_missing, data)?;
    Ok(HeatmapData {
        minimum: DeferredHeatmapMinimum {
            missing: has_missing,
        },
        color_label: chart.prepare_color_label(data, None)?,
        layout: heatmap_layout,
        value_labels,
        value_owners,
        colorbar_values,
        x_levels,
        y_levels,
        cells,
        fills: resolved_fills,
        highlighted,
        null_row_annotations,
        has_missing,
        color,
    })
}

fn prepare_colorbar_values(
    domain: ColorDomain,
    has_missing: bool,
    data: crate::data::PlotData<'_>,
) -> Result<Vec<String>, Error> {
    let mut labels = Vec::new();
    data.reserve_source(&mut labels, 4)?;
    for value in [
        Some(domain.minimum),
        Some(domain.maximum),
        domain
            .center
            .filter(|center| *center > domain.minimum && *center < domain.maximum),
    ]
    .into_iter()
    .flatten()
    {
        use std::fmt::Write;
        let mut label = String::new();
        data.reserve_source_string(&mut label, 32)?;
        write!(label, "{}", ColorbarValue(value)).expect("reserved colorbar number");
        labels.push(label);
    }
    if has_missing {
        labels.push(data.copy_source_string("Missing")?);
    }
    Ok(labels)
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
    data: crate::data::PlotData<'_>,
) -> Result<Vec<Option<String>>, Error> {
    let annotations = chart.heatmap_null_row_annotations_spec();
    let mut resolved = data.collect_source(y_levels.iter().map(|_| Ok(None)))?;
    if annotations.is_empty() {
        return Ok(resolved);
    }
    if layout != HeatmapLayout::Cartesian {
        return Err(Error::Data(
            "heatmap: fully-null row annotations require HeatmapLayout::Cartesian".into(),
        ));
    }

    let mut seen = Vec::new();
    data.reserve_source(&mut seen, annotations.len())?;
    for annotation in annotations {
        let row_label = data.normalize_source_string(annotation.row_label())?;
        data.claim_source_visits(seen.len())?;
        if seen.contains(&row_label) {
            return Err(Error::Data(format!(
                "heatmap: duplicate fully-null row annotation for normalized y category {row_label:?}"
            )));
        }
        data.claim_source_visits(y_levels.len())?;
        let y_index = y_levels
            .iter()
            .position(|level| level == &row_label)
            .ok_or_else(|| {
                Error::Data(format!(
                    "heatmap: fully-null row annotation targets unknown normalized y category {row_label:?}"
                ))
            })?;
        let row = &cells[y_index * x_count..(y_index + 1) * x_count];
        data.claim_source_visits(row.len())?;
        if row.iter().any(Option::is_some) {
            return Err(Error::Data(format!(
                "heatmap: row {row_label:?} must be fully null before it can receive a merged row annotation"
            )));
        }
        data.claim_source_visits(x_count)?;
        if highlighted[y_index * x_count..(y_index + 1) * x_count]
            .iter()
            .any(|value| *value)
        {
            return Err(Error::Data(format!(
                "heatmap: fully-null row {row_label:?} cannot combine a merged annotation with true cell highlights"
            )));
        }
        resolved[y_index] = Some(data.copy_source_string(annotation.text())?);
        seen.push(row_label);
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
        if !minimum.is_finite() || !maximum.is_finite() || minimum > maximum {
            return Err(Error::Data(format!(
                "heatmap: color limits must be finite and nondecreasing; got {minimum}..{maximum}"
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

#[derive(Debug)]
pub(crate) struct HeatmapPayload {
    clip: PanelBox,
    cross_style: LineStyle,
    cells: Vec<CellPayload>,
    colorbar: Option<ColorbarPayload>,
}

#[derive(Debug)]
struct CellPayload {
    corners: [Point2D; 4],
    fill: FillStyle,
    text: Option<PlacedText>,
    cross: Option<[[Point2D; 2]; 2]>,
}

#[derive(Debug)]
struct ColorbarPayload {
    bounds: PanelBox,
    start: Point2D,
    end: Point2D,
    stops: Vec<GradientStop>,
    text: Vec<PlacedText>,
    missing: Option<CellPayload>,
}

impl CellPayload {
    fn draw(&self, backend: &mut dyn Backend, cross_style: &LineStyle) -> Result<(), Error> {
        backend.draw_polygon(&self.corners, &self.fill)?;
        if let Some(text) = &self.text {
            text.draw(backend)?;
        }
        if let Some(cross) = &self.cross {
            for line in cross {
                backend.draw_line(line, cross_style)?;
            }
        }
        Ok(())
    }

    fn geometry(
        &self,
        geometry: &mut crate::render::geometry::GeometrySnapshot,
        clip: Option<PanelBox>,
        cross_style: &LineStyle,
    ) -> Result<(), Error> {
        geometry.polygon(&self.corners, &self.fill, clip)?;
        if let Some(text) = &self.text {
            geometry.text(text.placement, &text.plan, clip)?;
        }
        if let Some(cross) = &self.cross {
            for line in cross {
                geometry.line(line, cross_style, clip)?;
            }
        }
        Ok(())
    }
}

impl HeatmapPayload {
    fn validate_annotation_ink(
        &self,
        theme: &ResolvedTheme,
        scope: &crate::layout::fit::FitScope,
        data: crate::data::PlotData<'_>,
    ) -> Result<(), crate::layout::fit::PlanError> {
        data.claim_source_visits(self.cells.len())?;
        for (index, cell) in self.cells.iter().enumerate() {
            let Some(text) = &cell.text else { continue };
            let mut ink = crate::render::geometry::GeometrySnapshot::new(theme)?;
            ink.text(text.placement, &text.plan, None)?;
            let containing = PanelBox::new(
                cell.corners[0].x,
                cell.corners[2].x,
                cell.corners[0].y,
                cell.corners[2].y,
            );
            if ink.bounds().iter().any(|bounds| {
                bounds.left < containing.x0
                    || bounds.right > containing.x1
                    || bounds.bottom < containing.y0
                    || bounds.top > containing.y1
            }) {
                return Err(scope
                    .child(crate::layout::AddressRole::HeatmapCell, index)?
                    .failure(
                        crate::layout::fit::FitKind::Text,
                        "required Heatmap annotation ink exceeds its assigned cell",
                    )
                    .into());
            }
        }
        Ok(())
    }

    pub(crate) fn draw(&self, backend: &mut dyn Backend) -> Result<(), Error> {
        #[cfg(test)]
        count(2);
        with_clip(backend, self.clip, |backend| {
            for cell in &self.cells {
                cell.draw(backend, &self.cross_style)?;
            }
            Ok(())
        })?;
        if let Some(bar) = &self.colorbar {
            backend.draw_linear_gradient(bar.bounds, bar.start, bar.end, &bar.stops)?;
            for text in &bar.text {
                text.draw(backend)?;
            }
            if let Some(missing) = &bar.missing {
                missing.draw(backend, &self.cross_style)?;
            }
        }
        Ok(())
    }

    fn geometry(
        &self,
        theme: &ResolvedTheme,
    ) -> Result<crate::render::geometry::GeometrySnapshot, Error> {
        let mut geometry = crate::render::geometry::GeometrySnapshot::new(theme)?;
        let clip = crate::render::backend::validate_clip_region(self.clip)?.panel();
        for cell in &self.cells {
            cell.geometry(&mut geometry, Some(clip), &self.cross_style)?;
        }
        if let Some(bar) = &self.colorbar {
            geometry.linear_gradient(bar.bounds, bar.start, bar.end, &bar.stops, None)?;
            for text in &bar.text {
                geometry.text(text.placement, &text.plan, None)?;
            }
            if let Some(missing) = &bar.missing {
                missing.geometry(&mut geometry, None, &self.cross_style)?;
            }
        }
        Ok(geometry)
    }
}

#[test]
fn required_heatmap_cell_text_must_fit_its_unclipped_cell() {
    let theme = crate::Theme::Publication.resolve();
    let text = theme
        .measure_legacy_appearance_text(
            "Required",
            &theme.typography.contrast_dark,
            crate::layout::PhysicalTextAlignment::Center,
            "heatmap cell annotation",
            false,
        )
        .unwrap();
    let mut payload = HeatmapPayload {
        clip: PanelBox::new(0.0, 20.0, 0.0, 30.0),
        cross_style: theme.marks.heatmap_missing_cross.backend_style(),
        cells: vec![CellPayload {
            corners: [
                Point2D::new(0.0, 0.0),
                Point2D::new(20.0, 0.0),
                Point2D::new(20.0, 30.0),
                Point2D::new(0.0, 30.0),
            ],
            fill: theme
                .marks
                .heatmap_cell_outline
                .backend_style((0, 0, 0).into()),
            text: Some(PlacedText::middle(
                text,
                Point2D::new(10.0, 15.0),
                TextAnchor::Middle,
                0.0,
            )),
            cross: None,
        }],
        colorbar: None,
    };
    let chart = Plot::heatmap(
        crate::CategoryValues::new("x", ["a"]).unwrap(),
        crate::CategoryValues::new("y", ["b"]).unwrap(),
        crate::NumericValues::new("value", [1.0]),
    )
    .erase();
    let resources = theme
        .prepared_font_environment()
        .unwrap()
        .generation_resources();
    let data = chart.data().with_axis_resources(&resources);
    let error = payload
        .validate_annotation_ink(&theme, &crate::layout::fit::ROOT_SCOPE, data)
        .unwrap_err();
    assert!(matches!(error, crate::layout::fit::PlanError::Fit(failure)
        if failure.reason.kind == crate::layout::fit::FitKind::Text
            && failure.reason.address == crate::layout::ComponentAddress::root()
                .child(crate::layout::AddressRole::HeatmapCell, 0).unwrap()));
    let cell = &mut payload.cells[0];
    cell.corners[1].x = 200.0;
    cell.corners[2].x = 200.0;
    cell.text.as_mut().unwrap().placement.position.x = 100.0;
    assert!(payload
        .validate_annotation_ink(&theme, &crate::layout::fit::ROOT_SCOPE, data)
        .is_ok());
}

fn select_cells(
    prepared: &HeatmapData,
    layout: &Layout,
    theme: &ResolvedTheme,
    annotations: &AnnotationPlans,
    data: crate::data::PlotData<'_>,
) -> Result<Vec<CellPayload>, Error> {
    let x_count = prepared.x_levels.len();
    let y_count = prepared.y_levels.len();
    let mut cross_width = None;
    let mut cells = Vec::new();
    data.reserve_source(&mut cells, prepared.cells.len())?;
    for y_index in 0..y_count {
        data.claim_source_visits(1)?;
        let y0 = y_count as f64 - y_index as f64 - 1.0;
        let y1 = y0 + 1.0;
        if let Some(annotation) = &annotations.rows[y_index] {
            let (left, bottom) = layout.projection.project(0.0, y0);
            let (right, top) = layout.projection.project(x_count as f64, y1);
            cells.push(CellPayload {
                corners: [
                    Point2D::new(left, bottom),
                    Point2D::new(right, bottom),
                    Point2D::new(right, top),
                    Point2D::new(left, top),
                ],
                fill: theme
                    .marks
                    .heatmap_missing_row_outline
                    .backend_style(MISSING_FILL.into()),
                cross: None,
                text: Some(PlacedText {
                    plan: annotation.clone(),
                    placement: crate::render::backend::PlannedTextPlacement::middle(
                        Point2D::new(
                            f64::midpoint(left, right).round(),
                            f64::midpoint(bottom, top).round(),
                        ),
                        TextAnchor::Middle,
                        0.0,
                    ),
                }),
            });
            continue;
        }
        for x_index in 0..x_count {
            data.claim_source_visits(1)?;
            if !active_cell(prepared.layout, x_index, y_index) {
                continue;
            }
            let x0 = x_index as f64;
            let x1 = x0 + 1.0;
            let (left, bottom) = layout.projection.project(x0, y0);
            let (right, top) = layout.projection.project(x1, y1);
            let value = prepared.cells[y_index * x_count + x_index];
            let highlighted = prepared.highlighted[y_index * x_count + x_index];
            let color = prepared.fills[y_index * x_count + x_index].unwrap_or(MISSING_FILL);
            let mut cell = CellPayload {
                corners: [
                    Point2D::new(left, bottom),
                    Point2D::new(right, bottom),
                    Point2D::new(right, top),
                    Point2D::new(left, top),
                ],
                fill: if highlighted {
                    &theme.marks.heatmap_highlight_outline
                } else {
                    &theme.marks.heatmap_cell_outline
                }
                .backend_style(color.into()),
                text: None,
                cross: None,
            };
            if value.is_some() {
                if let Some(plan) = &annotations.values[y_index * x_count + x_index] {
                    cell.text = Some(PlacedText::middle(
                        plan.clone(),
                        Point2D::new(
                            f64::midpoint(left, right).round(),
                            f64::midpoint(bottom, top).round(),
                        ),
                        TextAnchor::Middle,
                        0.0,
                    ));
                }
            } else {
                let stroke = if let Some(stroke) = cross_width {
                    stroke
                } else {
                    let stroke = crate::render::geometry::line_ink_width(
                        theme,
                        &theme.marks.heatmap_missing_cross.backend_style(),
                    )?;
                    cross_width = Some(stroke);
                    stroke
                };
                let inset = ((right - left).min(top - bottom) * 0.16)
                    .clamp(1.0, 4.0)
                    .max(stroke / 2.0 + 1.0 / 1024.0);
                cell.cross = Some([
                    [
                        Point2D::new(left + inset, bottom + inset),
                        Point2D::new(right - inset, top - inset),
                    ],
                    [
                        Point2D::new(left + inset, top - inset),
                        Point2D::new(right - inset, bottom + inset),
                    ],
                ]);
            }
            cells.push(cell);
        }
    }
    Ok(cells)
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
        .claim_processing_visits(count as u128)
        .map_err(|error| Error::Render(format!("heatmap annotation work: {error:?}")))?;
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
            let Some(_) = value else { continue };
            let source = prepared.value_labels[index]
                .as_deref()
                .ok_or_else(|| Error::Render("heatmap value text identity missing".into()))?;
            let fill = prepared.fills[index].ok_or_else(|| {
                Error::Render("heatmap value has no prepared resolved fill".into())
            })?;
            let appearance = theme.contrasting_text(fill.into())?;
            let owner = prepared.value_owners[index]
                .as_deref()
                .ok_or_else(|| Error::Render("heatmap value owner missing".into()))?;
            let plan = text.plan(source, owner, || {
                Ok(theme.measure_legacy_appearance_text(
                    source,
                    appearance,
                    PhysicalTextAlignment::Center,
                    owner,
                    false,
                )?)
            })?;
            let width = plan.width();
            let height = plan.height();
            plans.values[index] = Some(plan);
            if cell_height < height + 5.0 || cell_width < width + 6.0 {
                FitFailure::retain_least(&mut miss, scope.child(AddressRole::HeatmapValueLabel, index)?.failure(FitKind::Text, format!(
                "heatmap: cells are too small for value annotations at precision {} (cell {cell_width:.1}x{cell_height:.1}px, need at least {:.1}x{:.1}px); increase chart size, reduce precision, or hide annotations",
                format.configured_precision(),
                width + 6.0,
                height + 5.0
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
        let block =
            match fit::outcome(
                text.plan(annotation, "heatmap fully-null row annotation", || {
                    fit::measured(fit::text(
                        theme,
                        annotation,
                        theme.contrasting_text(missing_fill())?,
                        available_width,
                        PhysicalTextAlignment::Center,
                        "heatmap fully-null row annotation",
                        &row_scope,
                    ))
                }),
            )? {
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

fn select_colorbar(
    prepared: &HeatmapData,
    layout: &Layout,
    color_label: &crate::layout::TextBlock,
    values: &[crate::layout::ResolvedTextPlan],
    theme: &ResolvedTheme,
    data: crate::data::PlotData<'_>,
) -> Result<ColorbarPayload, Error> {
    let (_, panel_right, panel_bottom, panel_top) = layout.projection.pixel_box();
    let left = panel_right + 18.0;
    let available_height = panel_top - panel_bottom;
    let height = if prepared.has_missing {
        (available_height - 34.0).clamp(40.0, 160.0)
    } else {
        available_height.clamp(40.0, 180.0)
    };
    let bottom = panel_bottom + (available_height - height) / 2.0;
    data.claim_source_bytes(3 * std::mem::size_of::<GradientStop>())?;
    data.claim_source_visits(3)?;
    let stops = prepared.color.guide_stops(data)?;
    let mut bar = ColorbarPayload {
        bounds: PanelBox::new(left, left + 12.0, bottom, bottom + height),
        start: Point2D::new(left + 6.0, bottom),
        end: Point2D::new(left + 6.0, bottom + height),
        stops,
        text: Vec::new(),
        missing: None,
    };
    data.reserve_source(&mut bar.text, 4)?;
    bar.text.push(PlacedText::from_deferred_first_line(
        values[1].clone(),
        Point2D::new(
            (left + 18.0).round(),
            (bottom + height - 4.0).max(0.0).round(),
        ),
        TextAnchor::Start,
        0.0,
    ));
    bar.text.push(PlacedText::from_deferred_first_line(
        values[0].clone(),
        Point2D::new((left + 18.0).round(), (bottom - 12.0).max(0.0).round()),
        TextAnchor::Start,
        0.0,
    ));
    if let Some(center) = prepared.color.domain.center {
        if center > prepared.color.domain.minimum && center < prepared.color.domain.maximum {
            bar.text.push(PlacedText::from_deferred_first_line(
                values[2].clone(),
                Point2D::new(
                    (left + 18.0).round(),
                    (bottom + height * prepared.color.scale.stops()[1].offset)
                        .max(0.0)
                        .round(),
                ),
                TextAnchor::Start,
                0.0,
            ));
        }
    }
    bar.text.push(PlacedText::from_deferred_first_line(
        color_label.plan().clone(),
        Point2D::new((left + 55.0).round(), (bottom + height / 2.0).round()),
        TextAnchor::Middle,
        -90.0,
    ));

    if prepared.has_missing {
        let swatch_bottom = (bottom - 32.0).max(3.0);
        bar.missing = Some(CellPayload {
            corners: [
                Point2D::new(left, swatch_bottom),
                Point2D::new(left + 12.0, swatch_bottom),
                Point2D::new(left + 12.0, swatch_bottom + 12.0),
                Point2D::new(left, swatch_bottom + 12.0),
            ],
            fill: theme
                .marks
                .heatmap_missing_key_outline
                .backend_style(MISSING_FILL.into()),
            cross: None,
            text: Some(PlacedText::from_deferred_first_line(
                values
                    .last()
                    .ok_or_else(|| Error::Render("missing colorbar text plan".into()))?
                    .clone(),
                Point2D::new((left + 18.0).round(), (swatch_bottom + 6.0).round()),
                TextAnchor::Start,
                0.0,
            )),
        });
    }
    Ok(bar)
}

fn same_value(left: f64, right: f64) -> bool {
    left.total_cmp(&right).is_eq() || (left.abs().to_bits() == 0 && right.abs().to_bits() == 0)
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

fn fitted_label(
    data: crate::data::PlotData<'_>,
    theme: &ResolvedTheme,
    label: &str,
    maximum_width: f64,
) -> Result<String, Error> {
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
        return data.copy_source_string(label);
    }
    if text_width(ELLIPSIS)? > maximum_width {
        return Ok(String::new());
    }
    let mut fitted = String::new();
    let capacity = label
        .len()
        .checked_add(ELLIPSIS.len())
        .ok_or_else(|| Error::Render("heatmap elision length overflow".into()))?;
    data.reserve_source_string(&mut fitted, capacity)?;
    let mut candidate = String::new();
    data.reserve_source_string(&mut candidate, capacity)?;
    data.claim_source_visits(label.len())?;
    for grapheme in label.graphemes(true) {
        data.claim_source_visits(fitted.len() + grapheme.len() + ELLIPSIS.len())?;
        candidate.clear();
        candidate.push_str(&fitted);
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
    data: crate::data::PlotData<'_>,
    theme: &ResolvedTheme,
    labels: &[String],
    maximum_width: f64,
    axis: &str,
    scope: &crate::layout::fit::FitScope,
) -> Result<Vec<String>, crate::layout::fit::PlanError> {
    let fitted = data.collect_source(
        labels
            .iter()
            .map(|label| fitted_label(data, theme, label, maximum_width)),
    )?;
    data.claim_source_visits(fitted.len())?;
    if fitted.iter().any(String::is_empty) {
        return Err(scope.failure(crate::layout::fit::FitKind::Tick, format!(
            "heatmap: {axis}-axis category labels have no room for an ellipsis; increase chart size or shorten labels"
        )).into());
    }
    for (index, label) in fitted.iter().enumerate() {
        data.claim_source_visits(index)?;
        if fitted[..index].contains(label) {
            return Err(scope.failure(crate::layout::fit::FitKind::Tick, format!(
            "heatmap: {axis}-axis category labels become ambiguous after elision; increase chart size or shorten labels"
        )).into());
        }
    }
    Ok(fitted)
}

fn row_y(index: usize, count: usize) -> f64 {
    count as f64 - index as f64 - 0.5
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod extraction;
