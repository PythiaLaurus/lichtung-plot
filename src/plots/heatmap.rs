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
use crate::theme::{
    ColorScales, ColorStop, ContinuousColorScale, DivergingGradient, OpaqueColor, ResolvedTheme,
};
use crate::Plot;

const MAX_CELLS: usize = 1_000_000;
#[derive(Debug, PartialEq)]
pub(crate) struct HeatmapData {
    pub(crate) x_levels: Vec<String>,
    pub(crate) y_levels: Vec<String>,
    cells: Vec<Option<f64>>,
    fills: Vec<Option<OpaqueColor>>,
    highlighted: Vec<bool>,
    null_row_annotations: Vec<Option<String>>,
    has_missing: bool,
    color: PreparedHeatmapColor,
    colorbar_stops: Vec<GradientStop>,
    layout: HeatmapLayout,
    label_fit: HeatmapLabelFit,
    value_labels: Vec<Option<String>>,
    value_owners: Vec<Option<String>>,
    colorbar_values: Vec<String>,
    pub(crate) color_label: String,
}

impl HeatmapData {
    pub(crate) fn guide_values(&self) -> &[String] {
        &self.colorbar_values
    }
    pub(crate) fn guide_semantics(
        &self,
        theme: &ResolvedTheme,
    ) -> Result<crate::decorations::legend::descriptor::ContinuousSemantics, Error> {
        let domain = self.color.domain();
        let missing = self
            .has_missing
            .then(|| -> Result<_, Error> {
                let label = self.colorbar_values.last().ok_or_else(|| {
                    Error::Render("heatmap missing key has no prepared label".into())
                })?;
                Ok(crate::decorations::legend::descriptor::MissingKey {
                    fill: theme.marks.heatmap_missing_fill,
                    outline: theme.marks.heatmap_missing_key_outline.clone(),
                    label: crate::data::source::copy_source_string(label)?,
                })
            })
            .transpose()?;
        Ok(
            crate::decorations::legend::descriptor::ContinuousSemantics {
                domain: (domain.minimum, domain.maximum),
                selection: self.color.selection.clone(),
                missing,
            },
        )
    }

    pub(crate) fn guide_samples(&self) -> Result<Vec<f64>, Error> {
        let mut samples = Vec::new();
        crate::data::source::reserve_source(&mut samples, self.cells.len())?;
        for &value in &self.cells {
            if let Some(value) = value {
                samples.push(value);
            }
        }
        Ok(samples)
    }

    pub(crate) fn install_shared_domain(
        &mut self,
        descriptor: &crate::decorations::legend::descriptor::ContinuousDescriptor,
    ) -> Result<(), Error> {
        self.replace_domain(ResolvedColorDomain {
            minimum: descriptor.domain.0,
            maximum: descriptor.domain.1,
            center: descriptor.center,
        })
    }

    fn replace_domain(&mut self, domain: ResolvedColorDomain) -> Result<(), Error> {
        let color = PreparedHeatmapColor::resolve(domain, self.color.selection.clone())?;
        let fills = crate::data::source::collect_source(
            self.cells
                .iter()
                .map(|value| value.map(|value| color.scale.color(value)).transpose()),
        )?;
        let colorbar_stops = color.guide_stops(&self.guide_samples()?)?;
        let colorbar_values = prepare_guide_labels(domain, self.has_missing)?;
        self.color = color;
        self.fills = fills;
        self.colorbar_stops = colorbar_stops;
        self.colorbar_values = colorbar_values;
        Ok(())
    }
}

#[expect(
    clippy::cast_precision_loss,
    reason = "selected CSS-pixel tracks are bounded by the exact u32 root dimensions"
)]
fn track_pixels(quanta: i64) -> f64 {
    quanta as f64 / 1024.0
}

/// Column maxima precede assigned-width cell text and row requirements.
fn assigned_columns(
    prepared: &HeatmapData,
    labels: &[String],
    panel: PanelBox,
    theme: &ResolvedTheme,

    scope: &crate::layout::fit::FitScope,
) -> Result<Vec<f64>, crate::layout::fit::PlanError> {
    use crate::layout::fit::FitOutcome;
    use crate::layout::kernel::{GridRequirement, TrackSpec};

    let count = prepared.x_levels.len();
    let cell = intrinsic_cell_ink(prepared, theme)?;
    let mut hard =
        crate::data::source::collect_source(labels.iter().map(|label| {
            Ok((cell.0 / count as f64).max(column_label_hard(prepared, label, theme)?))
        }))?;

    for (index, (source, fill)) in prepared
        .value_labels
        .iter()
        .zip(&prepared.fills)
        .enumerate()
    {
        let (Some(source), Some(fill)) = (source, fill) else {
            continue;
        };
        let appearance = theme.contrasting_text((*fill).into())?;
        let widths = crate::layout::fit::intrinsic_widths(
            theme,
            source,
            appearance,
            "Heatmap column value label",
        )?;
        hard[index % count] = hard[index % count].max(widths.0 + 6.0);
    }
    let preferred = crate::layout::kernel::ceil_spatial_quanta(
        2.0 * f64::from(theme.typography.axes.tick_label.font_size),
    )?;
    let base = crate::data::source::collect_source(hard.into_iter().map(|width| {
        let min = crate::layout::kernel::ceil_spatial_quanta(width)?;
        Ok(TrackSpec {
            min,
            preferred: min.max(preferred),
            max: None,
            flexible: true,
        })
    }))?;
    let start = crate::layout::kernel::nearest_spatial_quanta(panel.x0)?;
    let end = crate::layout::kernel::nearest_spatial_quanta(panel.x1)?;
    let total = end
        .checked_sub(start)
        .ok_or_else(|| Error::Render("Heatmap assigned column track span overflowed".into()))?;
    let children: [GridRequirement<'_>; 0] = [];
    let allocation = match crate::layout::kernel::allocate_grid_axis(
        &base,
        &children,
        total,
        [0, 0],
        0,
        None,
        true,
        scope,
    )? {
        FitOutcome::Measured(allocation) => allocation,
        FitOutcome::Miss(failure) => return Err(failure.into()),
    };
    let mut edges = Vec::new();
    crate::data::source::reserve_source(&mut edges, count + 1)?;
    let mut edge = start;
    edges.push(track_pixels(edge));
    for width in allocation.sizes {
        edge = edge
            .checked_add(width)
            .ok_or_else(|| Error::Render("Heatmap column edge overflowed spatial quanta".into()))?;
        edges.push(track_pixels(edge));
    }
    Ok(edges)
}

/// Measure each row only after the selected columns constrain its text.
fn assigned_rows(
    prepared: &HeatmapData,
    annotations: &AnnotationPlans,
    columns: &[f64],
    labels: &[String],
    tick_width: f64,
    context: PlotRenderContext<'_>,
) -> Result<Vec<f64>, crate::layout::fit::PlanError> {
    use crate::layout::fit::FitOutcome;
    use crate::layout::kernel::{GridRequirement, TrackSpec};

    let panel = context.panel;
    let theme = context.theme;

    let count = prepared.y_levels.len();
    let width = prepared.x_levels.len();
    if labels.len() != count {
        return Err(
            Error::Render("Heatmap prepared row label count differs from tracks".into()).into(),
        );
    }
    let ink = intrinsic_cell_ink(prepared, theme)?;
    let mut hard =
        crate::data::source::collect_source((0..count).map(|_| Ok(ink.1 / count as f64)))?;
    for (index, label) in labels.iter().enumerate() {
        if label.is_empty() {
            continue;
        }
        let text = crate::layout::fit::measured(crate::layout::fit::text(
            theme,
            label,
            &theme.typography.axes.tick_label,
            tick_width,
            PhysicalTextAlignment::End,
            "y-axis tick label",
            &context
                .fit_scope()
                .child(crate::layout::AddressRole::AxisLeft, 0)?,
        ))?;
        hard[index] = hard[index].max(text.height() + 2.0 * crate::layout::LABEL_CLEARANCE);
    }
    for (index, plan) in annotations.values.iter().enumerate() {
        if let Some(plan) = plan {
            hard[index / width] = hard[index / width].max(plan.height() + 5.0);
        }
    }
    for (index, plan) in annotations.rows.iter().enumerate() {
        if let Some(plan) = plan {
            hard[index] = hard[index].max(plan.height() + 4.0);
        }
    }

    let square = columns
        .windows(2)
        .map(|pair| pair[1] - pair[0])
        .fold(0.0_f64, f64::max);
    let preferred = crate::layout::kernel::ceil_spatial_quanta(
        square.max(2.0 * f64::from(theme.typography.axes.tick_label.font_size)),
    )?;
    let base = crate::data::source::collect_source(hard.into_iter().map(|height| {
        let min = crate::layout::kernel::ceil_spatial_quanta(height)?;
        Ok(TrackSpec {
            min,
            preferred: min.max(preferred),
            max: None,
            flexible: true,
        })
    }))?;
    let bottom = crate::layout::kernel::nearest_spatial_quanta(panel.y0)?;
    let top = crate::layout::kernel::nearest_spatial_quanta(panel.y1)?;
    let total = top
        .checked_sub(bottom)
        .ok_or_else(|| Error::Render("Heatmap assigned row track span overflowed".into()))?;
    let children: [GridRequirement<'_>; 0] = [];
    let allocation = match crate::layout::kernel::allocate_grid_axis(
        &base,
        &children,
        total,
        [0, 0],
        0,
        None,
        true,
        context.fit_scope(),
    )? {
        FitOutcome::Measured(allocation) => allocation,
        FitOutcome::Miss(failure) => return Err(failure.into()),
    };
    let mut edges = Vec::new();
    crate::data::source::reserve_source(&mut edges, count + 1)?;
    let mut edge = top;
    edges.push(track_pixels(edge));
    for height in allocation.sizes {
        edge = edge
            .checked_sub(height)
            .ok_or_else(|| Error::Render("Heatmap row edge overflowed spatial quanta".into()))?;
        edges.push(track_pixels(edge));
    }
    Ok(edges)
}

#[derive(Debug)]
pub(crate) struct PreparedHeatmapInputs {
    pub(crate) panels: Vec<(Option<String>, HeatmapData)>,
}

fn retained_in(chart: &Plot) -> Result<&HeatmapData, Error> {
    let inputs = chart
        .prepared_heatmap
        .as_ref()
        .ok_or_else(|| Error::Render("heatmap requires root-prepared semantics".into()))?;

    crate::layout::interior::preference::panel(&inputs.panels, chart.axis_scope(), "Heatmap")
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ResolvedColorDomain {
    minimum: f64,
    maximum: f64,
    center: Option<f64>,
}

#[derive(Debug, PartialEq)]
struct PreparedHeatmapColor {
    selection: ContinuousColorScale,
    scale: crate::theme::ContinuousColors,
}

impl PreparedHeatmapColor {
    fn resolve(
        domain: ResolvedColorDomain,
        selection: ContinuousColorScale,
    ) -> Result<Self, Error> {
        let semantic_stops = selection.stops()?;
        if selection.kind() == crate::theme::ContinuousScaleKind::Sequential
            && selection.sequential_gradient().is_none()
        {
            return Err(Error::Render(
                "prepared sequential Heatmap has no retained gradient".into(),
            ));
        }
        if (selection.kind() == crate::theme::ContinuousScaleKind::Diverging)
            != domain.center.is_some()
            || selection.center().map(f64::to_bits) != domain.center.map(f64::to_bits)
        {
            return Err(Error::Render(
                "resolved Heatmap domain disagrees with its retained scale selection".into(),
            ));
        }
        if domain.center.is_some()
            && (semantic_stops.len() != 3
                || semantic_stops[1].offset().to_bits() != 0.5_f64.to_bits())
        {
            return Err(Error::Data(
                "heatmap diverging gradient requires one neutral center stop".into(),
            ));
        }
        let low = semantic_stops[0].color();
        let neutral = selection.diverging_gradient().map_or_else(
            || semantic_stops[semantic_stops.len() / 2].color(),
            DivergingGradient::neutral,
        );
        let high = semantic_stops[semantic_stops.len() - 1].color();
        let (stops, count) = if let Some(center) = domain.center {
            if !center.is_finite() || center < domain.minimum || center > domain.maximum {
                return Err(Error::Data(
                    "heatmap diverging domain must contain its finite center".into(),
                ));
            }
            if same_value(domain.minimum, domain.maximum) {
                (
                    [
                        ColorStop::new(0.0, neutral)?,
                        ColorStop::new(1.0, neutral)?,
                        ColorStop::new(1.0, neutral)?,
                    ],
                    2,
                )
            } else if same_value(center, domain.minimum) {
                (
                    [
                        ColorStop::new(0.0, neutral)?,
                        ColorStop::new(1.0, high)?,
                        ColorStop::new(1.0, high)?,
                    ],
                    2,
                )
            } else if same_value(center, domain.maximum) {
                (
                    [
                        ColorStop::new(0.0, low)?,
                        ColorStop::new(1.0, neutral)?,
                        ColorStop::new(1.0, neutral)?,
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
                        ColorStop::new(0.0, low)?,
                        ColorStop::new(offset, neutral)?,
                        ColorStop::new(1.0, high)?,
                    ],
                    3,
                )
            }
        } else {
            (
                [
                    semantic_stops[0],
                    semantic_stops[semantic_stops.len() - 1],
                    semantic_stops[semantic_stops.len() - 1],
                ],
                semantic_stops.len(),
            )
        };
        Ok(Self {
            scale: crate::theme::ContinuousColors::resolve(
                (domain.minimum, domain.maximum),
                if domain.center.is_none() {
                    semantic_stops
                } else {
                    &stops[..count]
                },
            )?,
            selection,
        })
    }

    fn domain(&self) -> ResolvedColorDomain {
        let (minimum, maximum) = self.scale.domain();
        ResolvedColorDomain {
            minimum,
            maximum,
            center: self.selection.center(),
        }
    }

    fn guide_stops(&self, observed: &[f64]) -> Result<Vec<GradientStop>, Error> {
        self.scale.guide_stops(observed)
    }
}

/// The one prepared-axis fast path is usable only for the appearance and
/// continuous domain that actually selected the annotation glyph roles.
#[derive(Debug)]
pub(crate) struct HeatmapIntrinsicCache {
    minimum: (f64, f64),
    without_y_labels: (f64, f64),
    color_scale: crate::theme::ContinuousColors,
    contrast_light: crate::theme::TextTheme,
    contrast_dark: crate::theme::TextTheme,
    tick_label: crate::theme::TextTheme,
    axis_title: crate::theme::TextTheme,
    plot_panel: crate::theme::Color,
    canvas_root: crate::theme::Color,
    canvas_override: Option<crate::theme::Color>,
    backdrop: crate::theme::Color,
    missing_fill: crate::theme::Color,
    cell_outline: crate::theme::OutlineTheme,
    highlight_outline: crate::theme::OutlineTheme,
    missing_row_outline: crate::theme::OutlineTheme,
    missing_cross: LineStyle,
}

impl HeatmapIntrinsicCache {
    fn matches_inputs(&self, prepared: &HeatmapData, theme: &ResolvedTheme) -> bool {
        self.color_scale.same(&prepared.color.scale)
            && self
                .contrast_light
                .same(&theme.typography.annotations.contrast.light)
            && self
                .contrast_dark
                .same(&theme.typography.annotations.contrast.dark)
            && self.tick_label.same(&theme.typography.axes.tick_label)
            && self.axis_title.same(&theme.typography.axes.axis_title)
            && self.plot_panel == theme.surfaces.plot_panel
            && self.canvas_root == theme.surfaces.canvas_root
            && self.canvas_override == theme.canvas_surface_override
            && self.backdrop == theme.annotations.planning_backdrop
            && self.missing_fill == theme.marks.heatmap_missing_fill
            && self.cell_outline == theme.marks.heatmap_cell_outline
            && self.highlight_outline == theme.marks.heatmap_highlight_outline
            && self.missing_row_outline == theme.marks.heatmap_missing_row_outline
            && self.missing_cross == theme.marks.heatmap_missing_cross.backend_style()
    }

    fn new(
        minimum: (f64, f64),
        without_y_labels: (f64, f64),
        prepared: &HeatmapData,
        theme: &ResolvedTheme,
    ) -> Result<Self, Error> {
        let color_scale = prepared.color.scale.clone();
        let text = |role: &crate::theme::TextTheme| -> Result<_, Error> {
            crate::theme::TextTheme::new(
                crate::data::source::copy_font_stack(role.stack())?,
                role.color(),
                role.font_size(),
                role.weight(),
                role.line_height(),
            )
        };
        Ok(Self {
            minimum,
            without_y_labels,
            color_scale,
            contrast_light: text(&theme.typography.annotations.contrast.light)?,
            contrast_dark: text(&theme.typography.annotations.contrast.dark)?,
            tick_label: text(&theme.typography.axes.tick_label)?,
            axis_title: text(&theme.typography.axes.axis_title)?,
            plot_panel: theme.surfaces.plot_panel,
            canvas_root: theme.surfaces.canvas_root,
            canvas_override: theme.canvas_surface_override,
            backdrop: theme.annotations.planning_backdrop,
            missing_fill: theme.marks.heatmap_missing_fill,
            cell_outline: theme.marks.heatmap_cell_outline.clone(),
            highlight_outline: theme.marks.heatmap_highlight_outline.clone(),
            missing_row_outline: theme.marks.heatmap_missing_row_outline.clone(),
            missing_cross: theme.marks.heatmap_missing_cross.backend_style(),
        })
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
    let (minimum, without_y_labels) =
        analyze_intrinsics(retained_in(chart)?, axes, theme, chart.axis_scope())?;
    if chart.y_tick_labels_visible() {
        Ok(minimum)
    } else {
        Ok(without_y_labels)
    }
}

pub(crate) fn natural_body_preference(
    chart: &Plot,
    theme: &ResolvedTheme,
) -> Result<(f64, f64), Error> {
    let marks = retained_in(chart)?;
    let hard = intrinsic_body_minimum(chart, theme)?;
    let pitch = 2.0 * f64::from(theme.typography.axes.tick_label.font_size);
    Ok((
        hard.0.max(pitch * marks.x_levels.len() as f64),
        hard.1.max(pitch * marks.y_levels.len() as f64),
    ))
}

pub(crate) fn natural_body_height(
    chart: &Plot,
    theme: &ResolvedTheme,
    width: f64,
) -> Result<f64, Error> {
    let marks = retained_in(chart)?;
    let axes = chart
        .prepared_axis_inputs()
        .ok_or_else(|| Error::Render("natural Heatmap axes missing".into()))?;
    let resources = theme.prepared_font_environment()?.operation();
    let data = chart.data().with_operation(&resources);
    let labels = axes.display_categories[0]
        .as_ref()
        .ok_or_else(|| Error::Render("natural Heatmap columns missing".into()))?;
    let columns = assigned_columns(
        marks,
        labels,
        PanelBox::new(0.0, width, 0.0, 1.0),
        theme,
        &crate::layout::fit::FitScope::root(),
    )
    .map_err(crate::layout::fit::PlanError::into_error)?;
    let text =
        PlotRenderContext::new(data, PanelBox::new(0.0, width, 0.0, 1.0), theme).producer_text()?;
    let annotations = measure_annotations(
        chart,
        marks,
        &columns,
        theme,
        &crate::layout::fit::FitScope::root(),
        &text,
    )
    .map_err(crate::layout::fit::PlanError::into_error)?;
    let preferred = columns.windows(2).map(|edge| edge[1] - edge[0]).fold(
        2.0 * f64::from(theme.typography.axes.tick_label.font_size),
        f64::max,
    );
    let mut heights =
        crate::data::source::collect_source(marks.y_levels.iter().map(|_| Ok(preferred)))?;
    for (index, plan) in annotations.values.iter().enumerate() {
        if let Some(plan) = plan {
            heights[index / marks.x_levels.len()] =
                heights[index / marks.x_levels.len()].max(plan.height() + 5.0);
        }
    }
    for (index, plan) in annotations.rows.iter().enumerate() {
        if let Some(plan) = plan {
            heights[index] = heights[index].max(plan.height() + 4.0);
        }
    }
    Ok(heights.into_iter().sum())
}

fn intrinsic_cell_ink(prepared: &HeatmapData, theme: &ResolvedTheme) -> Result<(f64, f64), Error> {
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
    let cell = |style: &crate::theme::OutlineTheme| -> Result<_, Error> {
        let fill = style.backend_style(crate::theme::Color::rgb(0, 0, 0));
        let (x, y) = crate::render::geometry::rectangular_fill_ink_outset(&fill)?;
        Ok((x + quantum, y + quantum))
    };
    let mut normal = None;
    let mut highlight = None;
    let mut missing_row = None;
    let mut cross = None;
    let mut pitch = (quantum, quantum);

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

fn column_label_hard(
    prepared: &HeatmapData,
    label: &str,
    theme: &ResolvedTheme,
) -> Result<f64, Error> {
    let hard = if prepared.label_fit == HeatmapLabelFit::Wrap {
        crate::layout::fit::intrinsic_widths(
            theme,
            label,
            &theme.typography.axes.tick_label,
            "Heatmap category intrinsic",
        )?
        .0
    } else {
        let full = theme.measure_text(
            label,
            &theme.typography.axes.tick_label,
            PhysicalTextAlignment::Center,
            "Heatmap category intrinsic",
            false,
        )?;
        let ellipsis = theme.measure_text(
            "...",
            &theme.typography.axes.tick_label,
            PhysicalTextAlignment::Center,
            "Heatmap ellipsis intrinsic",
            false,
        )?;
        full.fit_width()
            .max(full.width())
            .min(ellipsis.fit_width().max(ellipsis.width()))
    };
    Ok(hard + 2.0 * crate::layout::LABEL_CLEARANCE)
}

pub(crate) fn prepare_intrinsics(
    prepared: &HeatmapData,
    axes: &crate::plots::shared_x::PreparedAxes,
    theme: &ResolvedTheme,
    scope: Option<&str>,
) -> Result<(f64, f64), Error> {
    Ok(analyze_intrinsics(prepared, axes, theme, scope)?.0)
}

#[expect(
    clippy::type_complexity,
    reason = "visible and hidden-y summaries share one appearance-keyed analysis"
)]
fn analyze_intrinsics(
    prepared: &HeatmapData,
    axes: &crate::plots::shared_x::PreparedAxes,
    theme: &ResolvedTheme,
    diagnostic_scope: Option<&str>,
) -> Result<((f64, f64), (f64, f64)), Error> {
    theme.prepared_font_environment()?;
    #[cfg(not(test))]
    let _ = diagnostic_scope;

    if let Some(cached) = axes.heatmap_intrinsics.get() {
        if cached.matches_inputs(prepared, theme) {
            return Ok((cached.minimum, cached.without_y_labels));
        }
    }
    #[cfg(test)]
    crate::orchestrator::measurements::intrinsic(
        crate::orchestrator::measurements::Counter::HeatmapIntrinsic,
        diagnostic_scope,
    );
    let mut minimum = (1.0_f64 / 1024.0, 1.0_f64 / 1024.0);
    let cell = intrinsic_cell_ink(prepared, theme)?;
    let columns = prepared.x_levels.len();
    let base_column = (cell.0 / columns as f64).max(1.0 / 1024.0);
    let mut column_hard =
        crate::data::source::collect_source((0..columns).map(|_| Ok(base_column)))?;
    let mut row_hard = crate::data::source::collect_source(
        (0..prepared.y_levels.len()).map(|_| Ok(cell.1 / prepared.y_levels.len() as f64)),
    )?;
    let x_labels = axes.display_categories[0]
        .as_ref()
        .ok_or_else(|| Error::Render("Heatmap x intrinsic labels missing".into()))?;
    let y_labels = axes.display_categories[1]
        .as_ref()
        .ok_or_else(|| Error::Render("Heatmap y intrinsic labels missing".into()))?;
    for (index, label) in x_labels.iter().enumerate() {
        column_hard[index] = column_hard[index].max(column_label_hard(prepared, label, theme)?);
    }
    let mut y_label_hard =
        crate::data::source::collect_source((0..y_labels.len()).map(|_| Ok(0.0_f64)))?;
    for (index, label) in y_labels.iter().enumerate() {
        let plan = theme.measure_text(
            label,
            &theme.typography.axes.tick_label,
            PhysicalTextAlignment::End,
            "Heatmap row intrinsic",
            false,
        )?;
        if !label.is_empty() {
            y_label_hard[index] = plan.height() + 2.0 * crate::layout::LABEL_CLEARANCE;
        }
    }

    for (index, (source, fill)) in prepared
        .value_labels
        .iter()
        .zip(&prepared.fills)
        .enumerate()
        .filter_map(|(index, (source, fill))| Some((index, (source.as_deref()?, (*fill)?))))
    {
        let fill = crate::theme::Color::from(fill);
        let appearance = theme.contrasting_text(fill)?;
        let plan = theme.measure_legacy_appearance_text(
            source,
            appearance,
            PhysicalTextAlignment::Center,
            "heatmap value intrinsic",
            false,
        )?;
        column_hard[index % columns] = column_hard[index % columns].max(plan.width() + 6.0);
        row_hard[index / columns] = row_hard[index / columns].max(plan.height() + 5.0);
    }
    let total_columns = column_hard.into_iter().try_fold(0.0_f64, |total, width| {
        let next = total + width;
        if !next.is_finite() {
            return Err(Error::Render(
                "heatmap column hard widths overflowed".into(),
            ));
        }
        Ok(next)
    })?;
    minimum.0 = minimum.0.max(total_columns);
    let appearance = theme.contrasting_text(theme.marks.heatmap_missing_fill)?;
    for (row, source) in prepared
        .null_row_annotations
        .iter()
        .enumerate()
        .filter_map(|(row, text)| Some((row, text.as_deref()?)))
    {
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
        row_hard[row] = row_hard[row].max(unwrapped.height() + 4.0);
    }
    minimum.0 = minimum.0.max(cell.0);
    let without_y_height = row_hard.iter().sum::<f64>().max(cell.1);
    minimum.1 = row_hard
        .iter()
        .zip(&y_label_hard)
        .map(|(row, label)| row.max(*label))
        .sum::<f64>()
        .max(cell.1);
    let mut without_y_labels = (minimum.0, without_y_height);
    for (side, title) in axes.labels.iter().enumerate() {
        if title.is_empty() {
            continue;
        }
        let hard = crate::layout::fit::intrinsic_widths(
            theme,
            title,
            &theme.typography.axes.axis_title,
            "Heatmap axis title intrinsic",
        )?
        .0;
        if side == 0 {
            minimum.0 = minimum.0.max(hard);
            without_y_labels.0 = without_y_labels.0.max(hard);
        } else {
            minimum.1 = minimum.1.max(hard);
            without_y_labels.1 = without_y_labels.1.max(hard);
        }
    }
    if axes.heatmap_intrinsics.get().is_none() {
        let cache = HeatmapIntrinsicCache::new(minimum, without_y_labels, prepared, theme)?;
        let _ = axes.heatmap_intrinsics.set(cache);
    }
    Ok((minimum, without_y_labels))
}

impl HeatmapData {
    pub(crate) fn guide_labels(&self) -> impl ExactSizeIterator<Item = &str> {
        self.colorbar_values[..self.colorbar_values.len() - usize::from(self.has_missing)]
            .iter()
            .map(String::as_str)
    }

    pub(crate) fn missing_guide_label(&self) -> Option<&str> {
        self.has_missing
            .then(|| self.colorbar_values.last().map(String::as_str))
            .flatten()
    }

    pub(crate) fn annotation_labels(&self) -> impl Iterator<Item = (&str, crate::theme::Color)> {
        self.value_labels
            .iter()
            .zip(&self.fills)
            .filter_map(|(label, fill)| {
                Some((label.as_deref()?, crate::theme::Color::from((*fill)?)))
            })
    }
}

pub(crate) fn measure(
    chart: &Plot,
    context: PlotRenderContext<'_>,
) -> Result<PlotRenderResult, crate::layout::fit::PlanError> {
    let prepared = retained_in(chart)?;
    let theme = context.theme;

    let text = context.producer_text()?;
    let minimum = intrinsic_body_minimum(chart, theme)?;
    let (body, colorbar) = if chart.continuous_guide_suppressed() {
        (context.panel, None)
    } else {
        let (body, guide) = guide::measure(
            prepared,
            context.panel,
            minimum.0,
            theme,
            &context
                .fit_scope()
                .child(crate::layout::AddressRole::OutsideGuideRight, 0)?,
            context.content_height.then_some(chart),
        )?;
        (body, Some(guide))
    };
    let mut annotations = None;
    let mut columns = None;
    let mut rows = None;
    let layout = (|| -> Result<_, crate::layout::fit::PlanError> {
        let x_labels = crate::data::source::collect_source(
            context
                .prepared_category_labels(0)
                .ok_or_else(|| Error::Render("heatmap x labels require prepared axes".into()))?
                .iter()
                .map(|label| crate::data::source::copy_source_string(label)),
        )?;
        let y_labels = crate::data::source::collect_source(
            context
                .prepared_category_labels(1)
                .ok_or_else(|| Error::Render("heatmap y labels require prepared axes".into()))?
                .iter()
                .map(|label| {
                    if chart.y_tick_labels_visible() {
                        crate::data::source::copy_source_string(label)
                    } else {
                        Ok(String::new())
                    }
                }),
        )?;
        let wrap_labels = chart.heatmap_label_fit_spec() == HeatmapLabelFit::Wrap;
        let available = context.axis_label_width.max(1.0 / 1024.0);
        if available <= 0.0 {
            return Err(context
                .fit_scope()
                .failure(
                    crate::layout::fit::FitKind::TrackOrResidual,
                    "Heatmap y-axis labels have no assigned perpendicular capacity",
                )
                .into());
        }
        let mut natural = 1.0 / 1024.0;
        for label in &y_labels {
            let plan = theme.measure_text(
                label,
                &theme.typography.axes.tick_label,
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
        let panel = body;
        if panel.width() < minimum.0 || panel.height() < minimum.1 {
            return Err(context.fit_scope().failure(crate::layout::fit::FitKind::SpecializedGeometry,
            "heatmap: canvas is too small after reserving categorical labels and the colorbar; increase chart size"
        ).into());
        }

        let x_count = prepared.x_levels.len();
        let y_count = prepared.y_levels.len();
        let assigned = assigned_columns(prepared, &x_labels, panel, theme, context.fit_scope())?;
        columns = Some(assigned);
        let column_edges = columns.as_ref().expect("assigned Heatmap columns");
        let measured = measure_annotations(
            chart,
            prepared,
            column_edges,
            theme,
            context.fit_scope(),
            &text,
        )?;
        let y_labels = if wrap_labels {
            y_labels
        } else {
            fitted_unique_labels(
                theme,
                &y_labels,
                |_| tick_width,
                "y",
                &context
                    .fit_scope()
                    .child(crate::layout::AddressRole::AxisLeft, 0)?,
            )?
        };
        let assigned = assigned_rows(
            prepared,
            &measured,
            column_edges,
            &y_labels,
            tick_width,
            context.with_panel(panel),
        )?;
        check_annotation_tracks(
            chart,
            prepared,
            &measured,
            column_edges,
            &assigned,
            context.fit_scope(),
        )?;
        annotations = Some(measured);
        rows = Some(assigned);
        let row_edges = rows.as_ref().expect("assigned Heatmap rows");
        let x_ticks = crate::data::source::collect_source(
            prepared.x_levels.iter().enumerate().map(|(index, _)| {
                let middle = f64::midpoint(column_edges[index], column_edges[index + 1]);
                let x = (middle - panel.x0) / panel.width() * x_count as f64;
                Ok(AxisTick::new(x, ""))
            }),
        )?;
        let y_ticks = crate::data::source::collect_source(y_labels.into_iter().enumerate().map(
            |(index, label)| {
                let middle = f64::midpoint(row_edges[index], row_edges[index + 1]);
                let y = (middle - panel.y0) / panel.height() * y_count as f64;
                Ok(AxisTick::new(y, label))
            },
        ))?;
        let axes = chart
            .prepared_axis_inputs()
            .ok_or_else(|| Error::Render("Heatmap requires prepared native axes".into()))?;
        let mut layout = Layout {
            projection: crate::layout::Projection::new(
                (0.0, x_count as f64),
                (0.0, y_count as f64),
                panel.as_tuple(),
            ),
            x_axis: crate::layout::Axis {
                label: crate::data::source::copy_source_string(&axes.labels[0])?,
                ticks: x_ticks,
                explicit_ticks: false,
                assigned_tick_width: None,
                assigned_cells: None,
                source: crate::layout::TickSource::SUPPLIED,
            },
            y_axis: crate::layout::Axis {
                label: crate::data::source::copy_source_string(&axes.labels[1])?,
                ticks: y_ticks,
                explicit_ticks: false,
                assigned_tick_width: None,
                assigned_cells: None,
                source: crate::layout::TickSource::SUPPLIED,
            },
        };
        // Elision already selected the complete displayed labels; do not wrap them again.
        layout.y_axis.explicit_ticks = !wrap_labels;
        layout.y_axis.assigned_tick_width = Some(tick_width);
        layout.x_axis.assigned_cells = Some(crate::data::source::collect_source(
            column_edges.windows(2).map(|edge| Ok((edge[0], edge[1]))),
        )?);
        layout.y_axis.assigned_cells = Some(crate::data::source::collect_source(
            row_edges.windows(2).map(|edge| Ok((edge[1], edge[0]))),
        )?);
        // Row text and exact cell fit preceded this axis selection.
        let x_labels = if wrap_labels {
            x_labels
        } else {
            fitted_unique_labels(
                theme,
                &x_labels,
                |index| (column_edges[index + 1] - column_edges[index]) * 0.9,
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
    })()?;
    let panel = layout.projection.panel_box();
    let columns = columns
        .as_ref()
        .ok_or_else(|| Error::Render("selected Heatmap has no assigned column tracks".into()))?;
    let rows = rows
        .as_ref()
        .ok_or_else(|| Error::Render("selected Heatmap has no assigned row tracks".into()))?;
    let annotations = annotations
        .ok_or_else(|| Error::Render("selected Heatmap has no measured cell text".into()))?;

    let payload = HeatmapPayload {
        clip: panel,
        cross_style: theme.marks.heatmap_missing_cross.backend_style(),
        cells: select_cells(prepared, columns, rows, theme, &annotations)?,
        colorbar: colorbar
            .as_ref()
            .map(|guide| guide::select(guide, panel, theme))
            .transpose()?,
    };
    payload.validate_annotation_ink(context.fit_scope())?;
    let geometry = payload.geometry()?;

    let mut result = PlotRenderResult::new(layout, Vec::new()).with_resolved_axis_keys(
        crate::data::source::collect_source(
            prepared
                .x_levels
                .iter()
                .map(|s| crate::data::source::copy_source_string(s)),
        )?,
        crate::data::source::collect_source(
            prepared
                .y_levels
                .iter()
                .map(|s| crate::data::source::copy_source_string(s)),
        )?,
    );
    result.x_domain = crate::spec::AxisDomain::Categorical;
    result.y_domain = crate::spec::AxisDomain::Categorical;
    result.axes = Some(result.layout.measure_axes_with_inputs(
        theme,
        context.fit_scope(),
        chart.axis_semantic_inputs([result.x_domain, result.y_domain]),
        || crate::layout::StandardPanelInputs::new(theme),
    )?);

    let texts = payload
        .cells
        .iter()
        .filter_map(|cell| cell.text.as_ref())
        .chain(payload.colorbar.iter().flat_map(|bar| {
            bar.calls.iter().filter_map(|call| match call {
                crate::decorations::legend::LegendCall::Text(text) => Some(text),
                _ => None,
            })
        }));
    for text in texts {
        crate::data::source::reserve_source(&mut result.producer_text, 1)?;
        result.producer_text.push(text.clone());
    }
    result.marks = Some(crate::data::source::source_arc(
        crate::layout::interior::SelectedMarkPlan {
            payload: crate::layout::interior::MarkPayload::Heatmap(payload),
            geometry,
        },
    )?);
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
                &crate::data::source::source_levels(x)?,
                chart.x_category_order_spec(),
                "x",
                !data.is_full_plot(),
            )?,
            resolve_axis_order(
                &crate::data::source::source_levels(y)?,
                chart.y_category_order_spec(),
                "y",
                !data.is_full_plot(),
            )?,
        ]),
        layout => {
            let observed = triangular_family_order(x, y, layout)?;
            let family = resolve_triangular_order(chart, &observed, !data.is_full_plot())?;
            let copy = crate::data::source::collect_source(
                family
                    .iter()
                    .map(|level| crate::data::source::copy_source_string(level)),
            )?;
            Ok([copy, family])
        }
    }
}

pub(crate) fn prepare<'a>(
    chart: &Plot,
    data: impl Into<crate::data::PlotData<'a>>,
    color_scales: &ColorScales,
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

    let explicit_triangular_order = heatmap_layout != HeatmapLayout::Cartesian
        && (!matches!(chart.x_category_order_spec(), CategoryOrder::Observed)
            || !matches!(chart.y_category_order_spec(), CategoryOrder::Observed));
    let mut x_indices = crate::data::source::collect_source(
        x_levels
            .iter()
            .enumerate()
            .map(|(index, level)| Ok((level.as_str(), index))),
    )?;
    let mut y_indices = crate::data::source::collect_source(
        y_levels
            .iter()
            .enumerate()
            .map(|(index, level)| Ok((level.as_str(), index))),
    )?;
    crate::data::source::sort_source_by(&mut x_indices, |left, right| left.0.cmp(right.0));
    crate::data::source::sort_source_by(&mut y_indices, |left, right| left.0.cmp(right.0));
    data.validate_row_source()?;

    let mut seen = crate::data::source::collect_source((0..cell_count).map(|_| Ok(false)))?;
    let mut cells = crate::data::source::collect_source((0..cell_count).map(|_| Ok(None)))?;
    let mut highlighted = crate::data::source::collect_source((0..cell_count).map(|_| Ok(false)))?;

    for (row, ((x, y), fill)) in x_values.iter().zip(&y_values).zip(&fills).enumerate() {
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
    )?;

    let mut finite = Vec::new();
    crate::data::source::reserve_source(&mut finite, fills.len())?;

    finite.extend(fills.iter().flatten().copied());
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

    let exact_domain = chart
        .heatmap_color_limits_spec()
        .map(|(minimum, maximum)| crate::theme::ColorDomain::exact(minimum, maximum))
        .transpose()?;
    let override_gradient = chart.heatmap_gradient_spec()?;

    let selection = if let Some(center) = domain.center {
        let gradient = if let Some(gradient) = override_gradient {
            DivergingGradient::from_sequence(gradient)?
        } else {
            color_scales.diverging().clone()
        };
        ContinuousColorScale::diverging(center, Some(gradient), exact_domain)?
    } else {
        ContinuousColorScale::sequential(
            Some(
                override_gradient
                    .unwrap_or(color_scales.sequential())
                    .clone(),
            ),
            exact_domain,
        )?
    };
    let color = PreparedHeatmapColor::resolve(domain, selection)?;
    let resolved_fills = crate::data::source::collect_source(
        cells
            .iter()
            .map(|value| value.map(|value| color.scale.color(value)).transpose()),
    )?;

    let has_missing = cells.iter().enumerate().any(|(index, value)| {
        let x_index = index % x_levels.len();
        let y_index = index / x_levels.len();
        active_cell(heatmap_layout, x_index, y_index) && value.is_none()
    });

    let value_labels = crate::data::source::collect_source(cells.iter().map(|value| {
        match (value, chart.heatmap_annotations_spec()) {
            (Some(value), HeatmapAnnotations::Values(format)) => {
                format.format_source(*value).map(Some)
            }
            _ => Ok(None),
        }
    }))?;
    let value_owners = crate::data::source::collect_source(value_labels.iter().enumerate().map(
        |(index, label)| {
            use std::fmt::Write;
            if label.is_none() {
                return Ok(None);
            }
            let mut owner = String::new();
            crate::data::source::reserve_source_string(&mut owner, 44)?;
            write!(owner, "heatmap value annotation {index}")
                .expect("reserved annotation identity");
            Ok(Some(owner))
        },
    ))?;
    let colorbar_values = prepare_guide_labels(domain, has_missing)?;
    let mut prepared = HeatmapData {
        color_label: chart.prepare_color_label(None)?,
        layout: heatmap_layout,
        label_fit: chart.heatmap_label_fit_spec(),
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
        colorbar_stops: Vec::new(),
    };
    prepared.colorbar_stops = prepared.color.guide_stops(&prepared.guide_samples()?)?;
    Ok(prepared)
}

fn prepare_guide_labels(
    domain: ResolvedColorDomain,
    has_missing: bool,
) -> Result<Vec<String>, Error> {
    let center = domain
        .center
        .filter(|center| *center > domain.minimum && *center < domain.maximum);
    let values = [
        domain.minimum,
        domain.maximum,
        center.unwrap_or(domain.minimum),
    ];
    let mut labels = crate::decorations::legend::descriptor::numeric_labels(
        &values[..2 + usize::from(center.is_some())],
    )?;
    if has_missing {
        crate::data::source::reserve_source(&mut labels, 1)?;
        labels.push(crate::data::source::copy_source_string("Missing")?);
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
) -> Result<Vec<Option<String>>, Error> {
    let annotations = chart.heatmap_null_row_annotations_spec();
    let mut resolved = crate::data::source::collect_source(y_levels.iter().map(|_| Ok(None)))?;
    if annotations.is_empty() {
        return Ok(resolved);
    }
    if layout != HeatmapLayout::Cartesian {
        return Err(Error::Data(
            "heatmap: fully-null row annotations require HeatmapLayout::Cartesian".into(),
        ));
    }

    let mut seen = Vec::new();
    crate::data::source::reserve_source(&mut seen, annotations.len())?;
    for annotation in annotations {
        let row_label = crate::data::source::normalize_source_string(annotation.row_label())?;

        if seen.contains(&row_label) {
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
        resolved[y_index] = Some(crate::data::source::copy_source_string(annotation.text())?);
        seen.push(row_label);
    }
    Ok(resolved)
}

fn normalized_categories(
    data: crate::data::PlotData<'_>,
    column: &str,
    axis: &str,
) -> Result<Vec<String>, Error> {
    crate::data::source::collect_source(
        crate::data::strict_strings(data, column, "heatmap")?
            .into_iter()
            .enumerate()
            .map(|(row, value)| {
                let normalized = crate::data::source::normalize_source_string(&value)?;
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
) -> Result<Vec<String>, Error> {
    match order {
        CategoryOrder::Observed => crate::data::source::collect_source(observed.iter().map(|level| crate::data::source::copy_source_string(level))),
        CategoryOrder::Explicit(requested) => {
            let requested = crate::data::source::collect_source(requested.iter().map(|level| crate::data::source::normalize_source_string(level)))?;

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
        (CategoryOrder::Observed, CategoryOrder::Observed) => crate::data::source::collect_source(
            observed
                .iter()
                .map(|level| crate::data::source::copy_source_string(level)),
        ),
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
) -> Result<Vec<String>, Error> {
    let mut levels = Vec::new();
    for (x, y) in x_values.iter().zip(y_values) {
        let pair = match layout {
            HeatmapLayout::UpperTriangle { .. } => [y, x],
            HeatmapLayout::LowerTriangle { .. } | HeatmapLayout::Cartesian => [x, y],
        };
        for value in pair {
            if !levels.contains(value) {
                crate::data::source::reserve_source(&mut levels, 1)?;
                levels.push(crate::data::source::copy_source_string(value)?);
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
) -> Result<ResolvedColorDomain, Error> {
    let center = match scale {
        HeatmapScale::Sequential => None,
        HeatmapScale::Diverging { center } => {
            if !center.is_finite() {
                return Err(Error::Data(format!(
                    "heatmap: diverging center must be finite, got {center}"
                )));
            }
            Some(if center == 0.0 { 0.0 } else { center })
        }
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
        return Ok(ResolvedColorDomain {
            minimum,
            maximum,
            center,
        });
    }

    let (minimum, maximum) = crate::data::min_max(values.iter().copied(), "heatmap")?;
    let Some(center) = center else {
        return Ok(ResolvedColorDomain {
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
        return Ok(ResolvedColorDomain {
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
    Ok(ResolvedColorDomain {
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
    colorbar: Option<crate::decorations::legend::LegendDrawPlan>,
}

#[derive(Debug)]
struct CellPayload {
    corners: [Point2D; 4],
    fill: FillStyle,
    text: Option<PlacedText>,
    cross: Option<[[Point2D; 2]; 2]>,
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

        scope: &crate::layout::fit::FitScope,
    ) -> Result<(), crate::layout::fit::PlanError> {
        for (index, cell) in self.cells.iter().enumerate() {
            let Some(text) = &cell.text else { continue };
            let mut ink = crate::render::geometry::GeometrySnapshot::new();
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
        with_clip(backend, self.clip, |backend| {
            for cell in &self.cells {
                cell.draw(backend, &self.cross_style)?;
            }
            Ok(())
        })?;
        if let Some(bar) = &self.colorbar {
            bar.draw(backend)?;
        }
        Ok(())
    }

    fn geometry(&self) -> Result<crate::render::geometry::GeometrySnapshot, Error> {
        let mut geometry = crate::render::geometry::GeometrySnapshot::new();
        let clip = crate::render::backend::validate_clip_region(self.clip)?.panel();
        for cell in &self.cells {
            cell.geometry(&mut geometry, Some(clip), &self.cross_style)?;
        }
        if let Some(bar) = &self.colorbar {
            bar.geometry(&mut geometry)?;
        }
        Ok(geometry)
    }
}

fn select_cells(
    prepared: &HeatmapData,
    columns: &[f64],
    rows: &[f64],
    theme: &ResolvedTheme,
    annotations: &AnnotationPlans,
) -> Result<Vec<CellPayload>, Error> {
    let x_count = prepared.x_levels.len();
    let y_count = prepared.y_levels.len();
    let mut cross_width = None;
    let mut cells = Vec::new();
    crate::data::source::reserve_source(&mut cells, prepared.cells.len())?;
    for y_index in 0..y_count {
        let (bottom, top) = (rows[y_index + 1], rows[y_index]);
        if let Some(annotation) = &annotations.rows[y_index] {
            let (left, right) = (columns[0], columns[x_count]);
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
                    .backend_style(theme.marks.heatmap_missing_fill),
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
            if !active_cell(prepared.layout, x_index, y_index) {
                continue;
            }
            let (left, right) = (columns[x_index], columns[x_index + 1]);
            let value = prepared.cells[y_index * x_count + x_index];
            let highlighted = prepared.highlighted[y_index * x_count + x_index];
            let color = prepared.fills[y_index * x_count + x_index]
                .map_or(theme.marks.heatmap_missing_fill, crate::theme::Color::from);
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
                .backend_style(color),
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

fn measure_annotations(
    chart: &Plot,
    prepared: &HeatmapData,
    columns: &[f64],
    theme: &ResolvedTheme,
    scope: &crate::layout::fit::FitScope,
    text: &crate::spec::ProducerText<'_>,
) -> Result<AnnotationPlans, crate::layout::fit::PlanError> {
    use crate::layout::fit::{self, FitFailure, FitKind, FitOutcome};
    use crate::layout::AddressRole;
    let mut miss = None;

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
    if let HeatmapAnnotations::Values(_) = chart.heatmap_annotations_spec() {
        for (index, value) in prepared.cells.iter().enumerate() {
            let Some(_) = value else { continue };
            let column = index % prepared.x_levels.len();
            let cell_width = columns[column + 1] - columns[column];
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
            let cell_scope = scope.child(AddressRole::HeatmapValueLabel, index)?;
            let measured = text.plan(source, owner, || {
                fit::measured(fit::text(
                    theme,
                    source,
                    appearance,
                    (cell_width - 6.0).max(0.0),
                    PhysicalTextAlignment::Center,
                    owner,
                    &cell_scope,
                ))
            });
            match measured {
                Ok(plan) => plans.values[index] = Some(plan),
                Err(crate::layout::fit::PlanError::Fit(failure)) => {
                    FitFailure::retain_least(&mut miss, failure);
                }
                Err(error) => return Err(error),
            }
        }
    }
    let available_width = columns[columns.len() - 1] - columns[0] - 12.0;
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
                        theme.contrasting_text(theme.marks.heatmap_missing_fill)?,
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
        if block.width() > available_width {
            FitFailure::retain_least(
                &mut miss,
                row_scope.failure(
                    FitKind::Text,
                    "heatmap: wrapped fully-null row annotation exceeds its assigned Grid width",
                ),
            );
        }
        plans.rows[y_index] = Some(block.plan().clone());
    }
    match miss {
        Some(failure) => Err(failure.into()),
        None => Ok(plans),
    }
}

fn check_annotation_tracks(
    chart: &Plot,
    prepared: &HeatmapData,
    annotations: &AnnotationPlans,
    columns: &[f64],
    rows: &[f64],
    scope: &crate::layout::fit::FitScope,
) -> Result<(), crate::layout::fit::PlanError> {
    use crate::layout::fit::{FitFailure, FitKind};
    use crate::layout::AddressRole;

    let mut miss = None;
    let count = prepared.x_levels.len();
    for (index, plan) in annotations.values.iter().enumerate() {
        let Some(plan) = plan else { continue };
        let col = index % count;
        let row = index / count;
        let width = columns[col + 1] - columns[col];
        let height = rows[row] - rows[row + 1];
        if width < plan.width() + 6.0 || height < plan.height() + 5.0 {
            let precision = match chart.heatmap_annotations_spec() {
                HeatmapAnnotations::Values(format) => format.configured_precision(),
                HeatmapAnnotations::Hidden => 0,
            };
            FitFailure::retain_least(&mut miss, scope.child(AddressRole::HeatmapValueLabel, index)?.failure(FitKind::Text, format!(
                "heatmap: cell {col},{row} is too small for required value annotation at precision {precision}; increase chart size or hide annotations"
            )));
        }
    }
    for (row, plan) in annotations.rows.iter().enumerate() {
        let Some(plan) = plan else { continue };
        if rows[row] - rows[row + 1] < plan.height() + 4.0 {
            FitFailure::retain_least(&mut miss, scope.child(AddressRole::HeatmapMergedRow, row)?.failure(FitKind::Text,
                "heatmap: required merged row text exceeds its assigned row; increase chart height"));
        }
    }
    miss.map_or(Ok(()), |failure| Err(failure.into()))
}

fn same_value(left: f64, right: f64) -> bool {
    left.total_cmp(&right).is_eq() || (left.abs().to_bits() == 0 && right.abs().to_bits() == 0)
}

fn fitted_label(theme: &ResolvedTheme, label: &str, maximum_width: f64) -> Result<String, Error> {
    const ELLIPSIS: &str = "...";

    let text_width = |text: &str| {
        theme
            .measure_text(
                text,
                &theme.typography.axes.tick_label,
                PhysicalTextAlignment::End,
                "heatmap category label",
                false,
            )
            .map(|plan| plan.fit_width())
    };
    if text_width(label)? <= maximum_width {
        return crate::data::source::copy_source_string(label);
    }
    if text_width(ELLIPSIS)? > maximum_width {
        return Ok(String::new());
    }
    let mut fitted = String::new();
    let capacity = label
        .len()
        .checked_add(ELLIPSIS.len())
        .ok_or_else(|| Error::Render("heatmap elision length overflow".into()))?;
    crate::data::source::reserve_source_string(&mut fitted, capacity)?;
    let mut candidate = String::new();
    crate::data::source::reserve_source_string(&mut candidate, capacity)?;

    for grapheme in label.graphemes(true) {
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
    theme: &ResolvedTheme,
    labels: &[String],
    maximum_width: impl Fn(usize) -> f64,
    axis: &str,
    scope: &crate::layout::fit::FitScope,
) -> Result<Vec<String>, crate::layout::fit::PlanError> {
    let fitted = crate::data::source::collect_source(
        labels
            .iter()
            .enumerate()
            .map(|(index, label)| fitted_label(theme, label, maximum_width(index))),
    )?;

    if fitted.iter().any(String::is_empty) {
        return Err(scope.failure(crate::layout::fit::FitKind::Tick, format!(
            "heatmap: {axis}-axis category labels have no room for an ellipsis; increase chart size or shorten labels"
        )).into());
    }
    for (index, label) in fitted.iter().enumerate() {
        if fitted[..index].contains(label) {
            return Err(scope.failure(crate::layout::fit::FitKind::Tick, format!(
            "heatmap: {axis}-axis category labels become ambiguous after elision; increase chart size or shorten labels"
        )).into());
        }
    }
    Ok(fitted)
}

mod guide;
