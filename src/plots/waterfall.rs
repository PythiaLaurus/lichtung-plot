//! Cumulative signed-delta waterfall plot.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::too_many_lines
)]

use crate::layout::{AxisTick, PanelBox, PhysicalTextAlignment, ResolvedTextPlan};
use crate::render::backend::{
    with_clip, Backend, FillStyle, LineStyle, PlacedText, PlannedTextPlacement, Point2D, TextAnchor,
};
use crate::spec::{PlotRenderContext, PlotRenderResult};
use crate::Error;
use crate::Plot;

#[cfg(test)]
thread_local! {
    pub(crate) static STAGES: std::cell::Cell<[usize; 2]> = const { std::cell::Cell::new([0; 2]) };
}

#[cfg(test)]
fn count(stage: usize) {
    STAGES.with(|counts| {
        let mut next = counts.get();
        next[stage] += 1;
        counts.set(next);
    });
}

#[derive(Debug)]
pub(crate) struct PreparedWaterfallInputs {
    pub(crate) panels: Vec<(Option<String>, PreparedWaterfall)>,
}

#[derive(Debug)]
pub(crate) struct PreparedWaterfall {
    pub(crate) labels: Vec<String>,
    pub(crate) range: (f64, f64),
    // Keep the source delta: subtracting rounded cumulative endpoints can lose its sign.
    spans: Vec<WaterfallSpan>,
    pub(crate) totals: [(f64, String); 2],
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct WaterfallSpan {
    source_row: usize,
    start: f64,
    end: f64,
    delta: f64,
}

fn retained<'a>(
    chart: &'a Plot,
    theme: &crate::theme::ResolvedTheme,
) -> Result<&'a PreparedWaterfall, Error> {
    let inputs = chart
        .prepared_waterfall
        .as_ref()
        .ok_or_else(|| Error::Render("Waterfall requires root-prepared spans".into()))?;
    crate::layout::interior::preference::panel(
        &inputs.panels,
        chart.axis_scope(),
        "Waterfall",
        &theme.prepared_font_environment()?.generation_resources(),
    )
}

fn mark_intrinsic_body_minimum(
    chart: &Plot,
    theme: &crate::theme::ResolvedTheme,
) -> Result<(f64, f64), Error> {
    let prepared = retained(chart, theme)?;
    if prepared.labels.is_empty() || prepared.labels.len() != prepared.spans.len() {
        return Err(Error::Render(
            "Waterfall categories and scientific spans differ".into(),
        ));
    }
    let resources = theme.prepared_font_environment()?.generation_resources();
    resources
        .claim_visits(prepared.spans.len() as u128)
        .map_err(|error| Error::Render(format!("Waterfall span intrinsic admission: {error:?}")))?;
    let positive = theme.marks.waterfall_bar.backend_style((0, 0, 0));
    let negative = positive.clone();
    let positive_ink = crate::render::geometry::rectangular_fill_ink_bounds(theme, &positive)?;
    let negative_ink = crate::render::geometry::rectangular_fill_ink_bounds(theme, &negative)?;
    let mut pitch = 1.0_f64 / 1024.0;
    let mut numeric = 0.0_f64;
    for span in &prepared.spans {
        let ink = if span.delta >= 0.0 {
            positive_ink
        } else {
            negative_ink
        };
        // One declared 0.6-category bar must retain distinct canonical edges.
        pitch = pitch.max((1.0 / 1024.0) / 0.6);
        if let Some(ink) = ink {
            pitch = pitch.max((-ink.bottom).max(0.0) / 0.2);
            pitch = pitch.max(ink.top.max(0.0) / 0.2);
            numeric = numeric.max(ink.right - ink.left);
        }
    }
    if prepared.spans.len() > 1 {
        let connector = crate::render::geometry::line_ink_width(
            theme,
            &theme.marks.waterfall_connector.backend_style(),
        )?;
        pitch = pitch.max(connector / 0.4);
        numeric = numeric.max(connector);
    }
    numeric = numeric.max(crate::render::geometry::line_ink_width(
        theme,
        &theme.marks.waterfall_reference.backend_style(),
    )?);
    let categories = f64::from(
        u32::try_from(prepared.labels.len())
            .map_err(|_| Error::Render("Waterfall category count exceeds spatial range".into()))?,
    );
    let mark_height = pitch * categories;
    let height = mark_height;
    if !height.is_finite() {
        return Err(Error::Render(
            "Waterfall mark/annotation hard span overflowed".into(),
        ));
    }
    Ok((
        if chart.user_x_limits().is_none() {
            numeric
        } else {
            0.0
        },
        height,
    ))
}

pub(crate) fn intrinsic_body_minimum(
    chart: &Plot,
    theme: &crate::theme::ResolvedTheme,
) -> Result<(f64, f64), Error> {
    let marks = mark_intrinsic_body_minimum(chart, theme)?;
    let axes =
        crate::layout::interior::coordinate::Intrinsics::prepared(chart, theme)?.minimum(chart);
    let prepared = retained(chart, theme)?;
    let resources = theme.prepared_font_environment()?.generation_resources();
    let mut height = 0.0_f64;
    for (value, text) in &prepared.totals {
        resources
            .claim_visits(1)
            .map_err(|error| Error::Render(format!("Waterfall total intrinsic: {error:?}")))?;
        if chart
            .user_x_limits()
            .is_some_and(|range| *value < range.0 || *value > range.1)
        {
            continue;
        }
        let plan = theme.measure_text(
            text,
            &theme.typography.waterfall_total,
            PhysicalTextAlignment::Center,
            "Waterfall total shelf intrinsic",
            false,
        )?;
        height = height.max(plan.height());
    }
    let shelf = if height > 0.0 {
        height + 2.0 * theme.annotations.waterfall_shelf_padding
    } else {
        0.0
    };
    Ok((marks.0.max(axes.0), marks.1.max(axes.1) + shelf))
}

pub(crate) fn natural_body_height(
    chart: &Plot,
    theme: &crate::theme::ResolvedTheme,
    width: f64,
) -> Result<f64, Error> {
    let resources = theme.prepared_font_environment()?.generation_resources();
    let inputs = chart
        .prepared_waterfall
        .as_ref()
        .ok_or_else(|| Error::Render("natural Waterfall rows missing".into()))?;
    let marks = crate::layout::interior::preference::panel(
        &inputs.panels,
        chart.axis_scope(),
        "natural Waterfall",
        &resources,
    )?;
    let axes = chart
        .prepared_axis_inputs()
        .ok_or_else(|| Error::Render("natural Waterfall axes missing".into()))?;
    let range = chart.x_axis_spec().limits.unwrap_or(axes.ranges[0]);
    let panel = PanelBox::new(0.0, width, 0.0, 1.0);
    let projection =
        crate::layout::Projection::new(range, (0.0, marks.labels.len() as f64), panel.as_tuple());
    let data = chart.data().with_axis_resources(&resources);
    let plan = plan_annotations(
        &projection,
        panel,
        marks.labels.len() as f64,
        &marks.totals,
        PlotRenderContext::new(data, panel, theme),
        data,
    )
    .map_err(crate::layout::fit::PlanError::into_error)?;
    let hard = mark_intrinsic_body_minimum(chart, theme)?.1.max(
        crate::layout::interior::coordinate::Intrinsics::prepared(chart, theme)?
            .minimum(chart)
            .1,
    );
    Ok(hard
        .max(2.0 * f64::from(theme.typography.tick_label.font_size) * marks.labels.len() as f64)
        .max(3.0 * width / 4.0)
        + plan.height)
}

pub(crate) fn prepare(
    chart: &Plot,
    data: crate::data::PlotData<'_>,
) -> Result<PreparedWaterfall, Error> {
    data.source_resources()?;
    #[cfg(test)]
    count(0);
    let label_col = chart
        .x_col()
        .ok_or_else(|| Error::Data("waterfall: label column not set".into()))?;
    let delta_col = chart
        .y_col()
        .ok_or_else(|| Error::Data("waterfall: delta column not set".into()))?;
    let base = chart
        .waterfall_base()
        .ok_or_else(|| Error::Data("waterfall: base value not set".into()))?;
    let labels = crate::data::strict_strings(data, label_col, "waterfall")?;
    let deltas = crate::data::strict_finite_f64(data, delta_col, "waterfall")?;
    if labels.is_empty() || labels.len() != deltas.len() {
        return Err(Error::Data(
            "waterfall: labels and deltas must have equal non-zero lengths".into(),
        ));
    }
    if !base.is_finite() {
        return Err(Error::Data("waterfall: base value must be finite".into()));
    }
    data.claim_mark_rows()?;
    // Category slots, connectors, and the two named total targets survive
    // caller-window clipping. Bar records are the original row slots above.
    data.claim_topology_slots(labels.len())?;
    data.claim_topology_slots(labels.len().saturating_sub(1))?;
    data.claim_topology_slots(2)?;
    let (spans, range) = prepare_spans(base, &deltas, data)?;
    let running = spans.last().map_or(base, |span| span.end);
    let (base_label, terminal_label) = chart
        .waterfall_labels_spec()
        .map_or(("base", "total"), |labels| {
            (labels.base(), labels.terminal())
        });
    let mut totals = [(base, String::new()), (running, String::new())];
    for ((value, text), label) in totals.iter_mut().zip([base_label, terminal_label]) {
        use std::fmt::Write;
        data.reserve_source_string(
            text,
            label
                .len()
                .checked_add(316)
                .ok_or_else(|| Error::Data("waterfall: label size overflow".into()))?,
        )?;
        write!(text, "{label} {value:.3}").expect("admitted total label");
    }
    Ok(PreparedWaterfall {
        labels,
        range,
        spans,
        totals,
    })
}

fn prepare_spans(
    base: f64,
    deltas: &[f64],
    data: crate::data::PlotData<'_>,
) -> Result<(Vec<WaterfallSpan>, (f64, f64)), Error> {
    data.source_resources()?;
    let mut running = base;
    let mut spans = Vec::new();
    data.reserve_source(&mut spans, deltas.len())?;
    data.claim_source_visits(deltas.len())?;
    let mut range = (base, base);
    for (index, delta) in deltas.iter().enumerate() {
        let next = running + delta;
        if !next.is_finite() {
            data.claim_source_bytes(128)?;
            return Err(Error::Data(format!(
                "waterfall: cumulative endpoint is non-finite at source row {}",
                data.source_row(index)
            )));
        }
        range.0 = range.0.min(next);
        range.1 = range.1.max(next);
        spans.push(WaterfallSpan {
            source_row: data.source_row(index),
            start: running,
            end: next,
            delta: *delta,
        });
        running = next;
    }
    Ok((spans, range))
}

pub(crate) fn measure(
    chart: &Plot,
    context: PlotRenderContext<'_>,
) -> Result<PlotRenderResult, crate::layout::fit::PlanError> {
    #[cfg(test)]
    count(1);
    let prepared = retained(chart, context.theme)?;
    let theme = context.theme;
    let resources = theme.prepared_font_environment()?.generation_resources();
    let data = context.data.with_axis_resources(&resources);
    let labels = &prepared.labels;
    let spans = &prepared.spans;
    let range = prepared.range;
    let base = prepared.totals[0].0;
    let terminal = prepared.totals[1].0;
    let x_range = context.axis_range(crate::layout::AxisSide::Bottom, || {
        crate::layout::continuous_auto_range(range, chart.x_axis_spec(), data.source_resources()?)
    })?;
    let panel = context.panel;
    let y_range = context.axis_range(crate::layout::AxisSide::Left, || {
        Ok(chart
            .y_axis_spec()
            .limits
            .unwrap_or((0.0, labels.len() as f64)))
    })?;
    let annotation_projection = crate::layout::Projection::new(x_range, y_range, panel.as_tuple())
        .with_categories([None, Some(labels.len())]);
    let annotations = plan_annotations(
        &annotation_projection,
        panel,
        labels.len() as f64,
        &prepared.totals,
        context,
        data,
    )?;
    let mark_minimum = mark_intrinsic_body_minimum(chart, theme)?;
    let axes_minimum =
        crate::layout::interior::coordinate::Intrinsics::prepared(chart, theme)?.minimum(chart);
    if annotations.height > panel.height() - mark_minimum.1.max(axes_minimum.1) {
        data.claim_source_bytes(1024)?;
        return Err(context.fit_scope().failure(crate::layout::fit::FitKind::SpecializedGeometry, format!(
            "waterfall: total annotations require {:.0}px while preserving the traced category tracks; increase chart height or shorten the configured labels",
            annotations.height
        )).into());
    }
    let mut mark_panel = panel;
    mark_panel.y1 -= annotations.height;
    let axes = chart
        .prepared_axis_inputs()
        .ok_or_else(|| Error::Render("Waterfall native axes missing".into()))?;
    let display = axes.display_categories[1]
        .as_ref()
        .ok_or_else(|| Error::Render("Waterfall category display missing".into()))?;
    let mut pitch = mark_minimum.1 / labels.len() as f64;
    let width = context.axis_label_width.max(1.0 / 1024.0);
    if chart.y_tick_labels_visible() {
        for label in display {
            data.claim_source_visits(1)?;
            let plan = crate::layout::fit::measured(crate::layout::fit::text(
                theme,
                label,
                &theme.typography.tick_label,
                width,
                PhysicalTextAlignment::End,
                "y-axis tick label",
                &context
                    .fit_scope()
                    .child(crate::layout::AddressRole::AxisLeft, 0, &resources)?,
            ))?;
            pitch = pitch.max(plan.height() + 2.0 * crate::layout::LABEL_CLEARANCE);
        }
    }
    let min = crate::layout::kernel::ceil_spatial_quanta(pitch)?;
    let preferred = min.max(crate::layout::kernel::ceil_spatial_quanta(
        2.0 * f64::from(theme.typography.tick_label.font_size),
    )?);
    let specs = data.collect_source(labels.iter().map(|_| {
        Ok(crate::layout::kernel::TrackSpec {
            min,
            preferred,
            max: None,
            flexible: true,
        })
    }))?;
    let bands = crate::layout::interior::tracks::assign(
        &specs,
        (mark_panel.y0, mark_panel.y1),
        0.0,
        false,
        data,
        context.fit_scope(),
    )?;
    let projection =
        crate::layout::Projection::new(x_range, (0.0, labels.len() as f64), mark_panel.as_tuple());
    let x = axes.specs[0].layout_ticks(
        Vec::new(),
        &projection,
        crate::layout::AxisSide::Bottom,
        theme,
        context.fit_scope(),
    )?;
    let ticks = data.collect_source(display.iter().zip(&bands).map(|(label, band)| {
        data.claim_source_visits(1)?;
        Ok(AxisTick::new(
            (f64::midpoint(band.0, band.1) - mark_panel.y0) / mark_panel.height()
                * labels.len() as f64,
            if chart.y_tick_labels_visible() {
                data.copy_source_string(label)?
            } else {
                String::new()
            },
        ))
    }))?;
    let layout = crate::layout::Layout {
        projection,
        x_axis: crate::layout::Axis {
            label: data.copy_source_string(&axes.labels[0])?,
            ticks: x.ticks,
            source: x.source,
            explicit_ticks: axes.specs[0].exact_ticks.is_some(),
            assigned_tick_width: None,
            assigned_cells: None,
        },
        y_axis: crate::layout::Axis {
            label: data.copy_source_string(&axes.labels[1])?,
            ticks,
            source: crate::layout::TickSource::SUPPLIED,
            explicit_ticks: false,
            assigned_tick_width: Some(width),
            assigned_cells: Some(data.collect_source(bands.iter().map(|band| {
                data.claim_source_visits(1)?;
                Ok(*band)
            }))?),
        },
    };
    let mut bars = Vec::new();
    let mut source_rows = Vec::new();
    let mut connectors = Vec::new();
    let accents = crate::theme::AccentColors::resolve(&theme.colors, 2, &resources)?;
    data.reserve_source(&mut bars, spans.len())?;
    data.reserve_source(&mut source_rows, spans.len())?;
    data.reserve_source(&mut connectors, spans.len().saturating_sub(1))?;
    for (index, span) in spans.iter().enumerate() {
        data.claim_source_visits(1)?;
        let (start, end, delta) = (&span.start, &span.end, &span.delta);
        let y0 = index as f64 + 0.2;
        let y1 = index as f64 + 0.8;
        let left = layout.projection.project(*start, y0).0;
        let right = layout.projection.project(*end, y1).0;
        let band = bands[index];
        let bottom = band.0 + 0.2 * (band.1 - band.0);
        let top = band.0 + 0.8 * (band.1 - band.0);
        layout.projection.validate_span(
            crate::layout::AxisSide::Bottom,
            (*start, *end),
            (left, right),
            *delta != 0.0,
            "waterfall polygon width",
        )?;
        layout.projection.validate_span(
            crate::layout::AxisSide::Left,
            (y0, y1),
            (bottom, top),
            true,
            "waterfall polygon height",
        )?;
        let color = accents.color(usize::from(*delta < 0.0))?;
        bars.push((
            [
                Point2D::new(left, bottom),
                Point2D::new(right, bottom),
                Point2D::new(right, top),
                Point2D::new(left, top),
            ],
            theme.marks.waterfall_bar.backend_style(color),
        ));
        source_rows.push(span.source_row);
        if index + 1 < spans.len() {
            let x = right;
            let from_y = top;
            let next = bands[index + 1];
            let to_y = next.0 + 0.2 * (next.1 - next.0);
            connectors.push((
                [Point2D::new(x, from_y), Point2D::new(x, to_y)],
                theme.marks.waterfall_connector.backend_style(),
            ));
        }
    }
    data.claim_source_visits(2)?;
    let references = [base, terminal].map(|value| {
        let (x, y0) = layout.projection.project(value, 0.0);
        let (_, y1) = layout.projection.project(value, labels.len() as f64);
        (
            [Point2D::new(x, y0), Point2D::new(x, y1)],
            theme.marks.waterfall_reference.backend_style(),
        )
    });
    let AnnotationPlan {
        items, baselines, ..
    } = annotations;
    let mut text = Vec::new();
    let mut text_targets = Vec::new();
    data.reserve_source(&mut text, items.len())?;
    data.reserve_source(&mut text_targets, items.len())?;
    for annotation in items {
        text_targets.push(annotation.target);
        let text_plan = annotation.text_plan;
        text.push(PlacedText {
            placement: PlannedTextPlacement::baseline(
                Point2D::new(
                    annotation.x,
                    layout.projection.panel_box().y1 + baselines[annotation.row],
                ),
                TextAnchor::Middle,
                0.0,
            ),
            plan: text_plan,
        });
    }
    let mut payload = WaterfallPayload {
        clip: layout.projection.panel_box(),
        bars,
        source_rows,
        connectors,
        references,
        text,
        text_targets,
        domain_expansions: [None; 2],
    };
    let mut geometry = crate::render::geometry::GeometrySnapshot::new(theme)?;
    let clip = crate::render::backend::validate_clip_region(payload.clip)?.panel();
    for (index, (points, style)) in payload.bars.iter().enumerate() {
        geometry.polygon(points, style, Some(clip))?;
        if let Some((points, style)) = payload.connectors.get(index) {
            geometry.line(points, style, Some(clip))?;
        }
    }
    for (points, style) in &payload.references {
        geometry.line(points, style, Some(clip))?;
    }
    for text in &payload.text {
        geometry.text(text.placement, &text.plan, None)?;
    }
    let mut label_ink = crate::render::geometry::GeometrySnapshot::new(theme)?;
    data.claim_source_visits(payload.text.len())?;
    for (index, text) in payload.text.iter().enumerate() {
        let mut one = crate::render::geometry::GeometrySnapshot::new(theme)?;
        one.text(text.placement, &text.plan, None)?;
        for ink in one.bounds() {
            if ink.bottom < mark_panel.y1
                || ink.top > panel.y1
                || (chart.user_x_limits().is_some()
                    && (ink.left < panel.x0 || ink.right > panel.x1))
            {
                return Err(context
                    .fit_scope()
                    .child(payload.text_targets[index], 0, &resources)?
                    .failure(
                        crate::layout::fit::FitKind::Text,
                        "required Waterfall total ink exceeds its assigned annotation shelf",
                    )
                    .into());
            }
        }
        label_ink.append(&one)?;
    }
    let mut full = crate::render::geometry::GeometrySnapshot::new(theme)?;
    data.claim_source_visits(payload.bars.len() + payload.connectors.len())?;
    for (points, style) in &payload.bars {
        full.polygon(points, style, None)?;
    }
    for (points, style) in &payload.connectors {
        full.line(points, style, None)?;
    }
    if let Some(bounds) = full
        .bounds()
        .iter()
        .copied()
        .reduce(crate::render::geometry::GeometryBox::union)
    {
        if bounds.bottom < clip.y0 || bounds.top > clip.y1 {
            return Err(context
                .fit_scope()
                .failure(
                    crate::layout::fit::FitKind::SpecializedGeometry,
                    "Waterfall painted bars/connectors exceed their assigned category panel",
                )
                .into());
        }
    }
    // Base/terminal reference strokes belong to the numeric axis but their
    // vertical ends intentionally meet the category frame.
    data.claim_source_visits(payload.references.len())?;
    for (points, style) in &payload.references {
        full.line(points, style, None)?;
    }
    if chart.user_x_limits().is_none() {
        if let Some(bounds) = label_ink
            .bounds()
            .iter()
            .copied()
            .reduce(crate::render::geometry::GeometryBox::union)
        {
            let totals = (
                prepared.totals[0].0.min(prepared.totals[1].0),
                prepared.totals[0].0.max(prepared.totals[1].0),
            );
            let requests = crate::layout::interior::automatic_absolute_ink_expansions(
                [totals, (0.0, labels.len() as f64)],
                &layout.projection,
                bounds,
                [true, false],
                data,
                context.fit_scope(),
            )?;
            payload.domain_expansions = crate::layout::interior::merge_mark_domain_expansions(
                payload.domain_expansions,
                requests,
            );
        }
        if let Some(bounds) = full
            .bounds()
            .iter()
            .copied()
            .reduce(crate::render::geometry::GeometryBox::union)
        {
            let count = u32::try_from(prepared.labels.len()).map_err(|_| {
                Error::Render("Waterfall category count exceeds spatial range".into())
            })?;
            let requests = crate::layout::interior::automatic_absolute_ink_expansions(
                [prepared.range, (0.0, f64::from(count))],
                &layout.projection,
                bounds,
                [true, false],
                data,
                context.fit_scope(),
            )?;
            payload.domain_expansions = crate::layout::interior::merge_mark_domain_expansions(
                payload.domain_expansions,
                requests,
            );
        }
    }
    let mut result = PlotRenderResult::new(layout, Vec::new());
    result.y_domain = crate::spec::AxisDomain::Categorical;
    result.y_tick_keys =
        Some(data.collect_source(labels.iter().map(|label| data.copy_source_string(label)))?);
    result.axes = Some(result.layout.measure_axes_with_inputs(
        theme,
        context.fit_scope(),
        chart.axis_semantic_inputs([result.x_domain, result.y_domain]),
        || crate::layout::StandardPanelInputs::new(theme),
    )?);
    result.producer_text = data.collect_source(payload.text.iter().map(|text| Ok(text.clone())))?;
    result.marks = Some(data.source_arc(crate::layout::interior::SelectedMarkPlan {
        payload: crate::layout::interior::MarkPayload::Waterfall(payload),
        geometry,
    })?);
    Ok(result)
}

#[derive(Debug)]
pub(crate) struct WaterfallPayload {
    clip: PanelBox,
    bars: Vec<([Point2D; 4], FillStyle)>,
    source_rows: Vec<usize>,
    connectors: Vec<([Point2D; 2], LineStyle)>,
    references: [([Point2D; 2], LineStyle); 2],
    text: Vec<PlacedText>,
    text_targets: Vec<crate::layout::AddressRole>,
    pub(crate) domain_expansions: [Option<crate::layout::interior::DomainExpansion>; 2],
}

impl WaterfallPayload {
    pub(crate) fn observe_sources(
        &self,
        round: &mut crate::layout::kernel::FeedbackRound,
    ) -> Result<(), Error> {
        round.observe_count(self.source_rows.len())?;
        for &row in &self.source_rows {
            round.observe_count(row)?;
        }
        Ok(())
    }
    pub(crate) fn draw(&self, backend: &mut dyn Backend) -> Result<(), Error> {
        with_clip(backend, self.clip, |backend| {
            for (index, (points, style)) in self.bars.iter().enumerate() {
                backend.draw_polygon(points, style)?;
                if let Some((points, style)) = self.connectors.get(index) {
                    backend.draw_line(points, style)?;
                }
            }
            for (points, style) in &self.references {
                backend.draw_line(points, style)?;
            }
            Ok(())
        })?;
        for text in &self.text {
            text.draw(backend)?;
        }
        Ok(())
    }
}

pub(crate) fn text_census_annotations(
    chart: &Plot,
    filter_to_range: bool,
    mut visit: impl FnMut(crate::font::canonical::CensusSource<'_>) -> Result<(), Error>,
) -> Result<(), Error> {
    let inputs = chart
        .prepared_waterfall
        .as_ref()
        .ok_or_else(|| Error::Render("waterfall census requires retained semantics".into()))?;
    for (scope, marks) in &inputs.panels {
        if chart.axis_scope().is_some() && scope.as_deref() != chart.axis_scope() {
            continue;
        }
        // Automatic domains contain all totals; only explicit limits can hide one.
        let range = chart.x_axis_spec().limits;
        for (value, label) in &marks.totals {
            if !filter_to_range || range.is_none_or(|range| *value >= range.0 && *value <= range.1)
            {
                visit(label.as_str().into())?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod extraction;

struct Annotation {
    target: crate::layout::AddressRole,
    text_plan: ResolvedTextPlan,
    x: f64,
    row: usize,
}

struct AnnotationPlan {
    items: Vec<Annotation>,
    height: f64,
    baselines: Vec<f64>,
}

struct ShelfRow {
    intervals: Vec<(i64, i64)>,
    above: i64,
    below: i64,
}

fn plan_annotations(
    projection: &crate::layout::Projection,
    panel: PanelBox,
    source_y: f64,
    candidates: &[(f64, String); 2],
    context: PlotRenderContext<'_>,
    data: crate::data::PlotData<'_>,
) -> Result<AnnotationPlan, crate::layout::fit::PlanError> {
    let theme = context.theme;
    let scope = context.fit_scope();
    let mut rows: Vec<ShelfRow> = Vec::new();
    let mut items = Vec::new();
    let mut miss = None;
    data.reserve_source(&mut items, 2)?;
    data.reserve_source(&mut rows, 2)?;
    let metrics = &theme.annotations;
    if [
        metrics.waterfall_shelf_padding,
        metrics.waterfall_shelf_gap,
        metrics.waterfall_shelf_row_gap,
    ]
    .into_iter()
    .any(|value| !value.is_finite() || value < 0.0)
    {
        return Err(
            Error::Render("Waterfall shelf spacing must be finite and nonnegative".into()).into(),
        );
    }
    let padding = crate::layout::kernel::ceil_spatial_quanta(metrics.waterfall_shelf_padding)?;
    let gap = crate::layout::kernel::ceil_spatial_quanta(metrics.waterfall_shelf_gap)?;
    let row_gap = crate::layout::kernel::ceil_spatial_quanta(metrics.waterfall_shelf_row_gap)?;
    let producer = context.producer_text()?;
    for (index, (value, text)) in candidates.iter().enumerate() {
        let target = if index == 0 {
            crate::layout::AddressRole::WaterfallBaseLabel
        } else {
            crate::layout::AddressRole::WaterfallTerminalLabel
        };
        data.claim_source_visits(1)?;
        if !projection.contains_data(*value, source_y) {
            continue;
        }
        let text_plan = producer.plan(text, "waterfall total annotation", || {
            Ok(theme.measure_text(
                text,
                &theme.typography.waterfall_total,
                PhysicalTextAlignment::Center,
                "waterfall total annotation",
                false,
            )?)
        })?;
        let width = text_plan.width();
        if width + 2.0 * metrics.waterfall_shelf_padding > panel.width() {
            // Debug escaping can expand each source byte; admit both the text
            // diagnostic before allocation; child paths have their own admission.
            let diagnostic_bytes = text
                .len()
                .checked_mul(10)
                .and_then(|bytes| bytes.checked_add(1024))
                .ok_or_else(|| Error::Render("waterfall: diagnostic text size overflow".into()))?;
            data.claim_source_visits(diagnostic_bytes)?;
            data.claim_source_bytes(diagnostic_bytes)?;
            crate::layout::fit::FitFailure::retain_least(&mut miss, scope.child(if index == 0 { crate::layout::AddressRole::WaterfallBaseLabel } else { crate::layout::AddressRole::WaterfallTerminalLabel }, 0, data.source_resources()?)?.failure(crate::layout::fit::FitKind::Text, format!(
                "waterfall: total annotation {text:?} requires {:.0}px but only {:.0}px is available; increase chart width or shorten the configured label",
                width + 2.0 * metrics.waterfall_shelf_padding,
                panel.width()
            )));
            continue;
        }
        let (projected_x, _) = projection.project(*value, source_y);
        let x_quanta = crate::layout::kernel::nearest_spatial_quanta(projected_x)?;
        let x = x_quanta as f64 / 1024.0;
        let placement =
            PlannedTextPlacement::baseline(Point2D::new(0.0, 0.0), TextAnchor::Middle, 0.0);
        let ink = crate::render::geometry::planned_text_ink_bounds(
            &text_plan,
            placement.position,
            placement.anchor,
            placement.rotation,
            placement.origin,
            placement.reverse_lines,
        );
        let above = text_plan.first_baseline_quanta.unwrap_or(0).max(
            crate::layout::kernel::ceil_spatial_quanta(ink.map_or(0.0, |ink| ink.top.max(0.0)))?,
        );
        let below = text_plan
            .height_quanta
            .checked_sub(text_plan.first_baseline_quanta.unwrap_or(0))
            .ok_or_else(|| Error::Render("Waterfall baseline extent overflow".into()))?
            .max(crate::layout::kernel::ceil_spatial_quanta(
                ink.map_or(0.0, |ink| (-ink.bottom).max(0.0)),
            )?);
        let half = crate::render::backend::nearest_even_text_half(text_plan.width_quanta)
            .ok_or_else(|| Error::Render("Waterfall text half width overflow".into()))?;
        let lower = (-half).min(-crate::layout::kernel::ceil_spatial_quanta(
            ink.map_or(0.0, |ink| (-ink.left).max(0.0)),
        )?);
        let upper = (text_plan.width_quanta - half).max(
            crate::layout::kernel::ceil_spatial_quanta(ink.map_or(0.0, |ink| ink.right.max(0.0)))?,
        );
        let bounds = (
            x_quanta
                .checked_add(lower)
                .ok_or_else(|| Error::Render("Waterfall label left edge overflow".into()))?,
            x_quanta
                .checked_add(upper)
                .ok_or_else(|| Error::Render("Waterfall label right edge overflow".into()))?,
        );
        let mut row = rows.len();
        for (candidate, occupied) in rows.iter().enumerate() {
            data.claim_source_visits(1)?;
            let mut fits = true;
            for &(left, right) in &occupied.intervals {
                data.claim_source_visits(1)?;
                if i128::from(bounds.1) + i128::from(gap) > i128::from(left)
                    && i128::from(right) + i128::from(gap) > i128::from(bounds.0)
                {
                    fits = false;
                    break;
                }
            }
            if fits {
                row = candidate;
                break;
            }
        }
        if row == rows.len() {
            rows.push(ShelfRow {
                intervals: Vec::new(),
                above: 0,
                below: 0,
            });
        }
        data.reserve_source(&mut rows[row].intervals, 1)?;
        rows[row].intervals.push(bounds);
        rows[row].above = rows[row].above.max(above);
        rows[row].below = rows[row].below.max(below);
        items.push(Annotation {
            target,
            text_plan,
            x,
            row,
        });
    }
    if let Some(failure) = miss {
        return Err(failure.into());
    }
    let mut baselines = Vec::new();
    data.reserve_source(&mut baselines, rows.len())?;
    let mut height = if rows.is_empty() {
        0_i128
    } else {
        i128::from(padding)
    };
    for (index, row) in rows.iter().enumerate() {
        data.claim_source_visits(1)?;
        let baseline = height
            .checked_add(i128::from(row.below))
            .ok_or_else(|| Error::Render("Waterfall shelf baseline overflow".into()))?;
        baselines.push(
            i64::try_from(baseline).map_err(|_| {
                Error::Render("Waterfall shelf baseline exceeds spatial range".into())
            })? as f64
                / 1024.0,
        );
        height = baseline
            .checked_add(i128::from(row.above))
            .and_then(|value| {
                value.checked_add(i128::from(if index + 1 == rows.len() {
                    padding
                } else {
                    row_gap
                }))
            })
            .ok_or_else(|| Error::Render("Waterfall shelf height overflow".into()))?;
    }
    let height = i64::try_from(height)
        .map_err(|_| Error::Render("Waterfall shelf height exceeds spatial range".into()))?
        as f64
        / 1024.0;
    Ok(AnnotationPlan {
        items,
        height,
        baselines,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::canonical::OperationGenerationResources;
    use crate::layout::fit::PlanError;
    use crate::layout::kernel::nearest_spatial_quanta;
    use crate::{CategoryValues, NumericValues};

    #[test]
    fn signed_waterfall_accents_extend_a_single_seed_without_color_cycling() {
        let source = Plot::waterfall(
            CategoryValues::new("step", ["rise", "fall"]).unwrap(),
            NumericValues::new("delta", [2.0, -1.0]),
            0.0,
        )
        .erase();
        let inputs =
            crate::plots::shared_x::prepare_plot(&source, &OperationGenerationResources::new())
                .unwrap();
        let chart = inputs.attach(source).unwrap();
        let mut theme = crate::Theme::Publication.resolve();
        theme.colors.truncate(1);
        let selected = chart
            .measure_selected_interior(PlotRenderContext::new(
                chart.data(),
                PanelBox::new(80.0, 700.0, 80.0, 700.0),
                &theme,
            ))
            .unwrap();
        let crate::layout::interior::MarkPayload::Waterfall(payload) =
            &selected.marks.as_ref().unwrap().payload
        else {
            panic!("selected Waterfall")
        };
        assert_eq!(payload.bars.len(), 2);
        let first = payload.bars[0].1.color;
        let second = payload.bars[1].1.color;
        assert_eq!(
            first,
            crate::theme::color_from_rgb(theme.colors[0])
                .with_alpha(theme.marks.waterfall_bar.fill_alpha)
        );
        assert_ne!(first, second);
    }

    #[test]
    fn overlapping_totals_use_separate_exact_baseline_shelves() {
        let source = Plot::waterfall(
            CategoryValues::new("step", ["a"]).unwrap(),
            NumericValues::new("delta", [0.0]),
            0.0,
        )
        .total_labels(crate::WaterfallLabels::new("Start", "Finish long label").unwrap())
        .x_label("")
        .y_label("")
        .erase();
        let inputs =
            crate::plots::shared_x::prepare_plot(&source, &OperationGenerationResources::new())
                .unwrap();
        let chart = inputs.attach(source).unwrap();
        let theme = crate::Theme::Publication.resolve();
        let panel = PanelBox::new(50.0, 850.0, 50.0, 550.0);
        let selected = chart
            .measure_selected_interior(PlotRenderContext::new(chart.data(), panel, &theme))
            .unwrap();
        let crate::layout::interior::MarkPayload::Waterfall(payload) =
            &selected.marks.as_ref().unwrap().payload
        else {
            panic!("Waterfall payload")
        };
        assert_eq!(payload.text.len(), 2);
        assert_eq!(
            payload.text[0].placement.position.x.to_bits(),
            payload.text[1].placement.position.x.to_bits()
        );
        assert!(payload.text[1].placement.position.y > payload.text[0].placement.position.y);
        let mut first = crate::render::geometry::GeometrySnapshot::new(&theme).unwrap();
        first
            .text(payload.text[0].placement, &payload.text[0].plan, None)
            .unwrap();
        let mut second = crate::render::geometry::GeometrySnapshot::new(&theme).unwrap();
        second
            .text(payload.text[1].placement, &payload.text[1].plan, None)
            .unwrap();
        let first_top = first
            .bounds()
            .iter()
            .map(|ink| ink.top)
            .fold(f64::NEG_INFINITY, f64::max);
        let second_bottom = second
            .bounds()
            .iter()
            .map(|ink| ink.bottom)
            .fold(f64::INFINITY, f64::min);
        assert!(second_bottom - first_top >= theme.annotations.waterfall_shelf_row_gap);
        assert!(first
            .bounds()
            .iter()
            .chain(second.bounds())
            .all(|ink| ink.bottom >= payload.clip.y1 && ink.top <= panel.y1));
    }

    #[test]
    fn traced_category_tracks_fit_below_the_retired_twenty_four_pixel_floor() {
        let source = Plot::waterfall(
            CategoryValues::new("step", ["a"]).unwrap(),
            NumericValues::new("delta", [0.0]),
            0.0,
        )
        .x_label("")
        .y_label("")
        .erase();
        let inputs =
            crate::plots::shared_x::prepare_plot(&source, &OperationGenerationResources::new())
                .unwrap();
        let chart = inputs.attach(source).unwrap();
        let theme = crate::Theme::Publication.resolve();
        let measure = |height| {
            chart.measure_selected_interior(PlotRenderContext::new(
                chart.data(),
                PanelBox::new(50.0, 850.0, 50.0, 50.0 + height),
                &theme,
            ))
        };
        let roomy = measure(500.0).unwrap();
        let band = 500.0 - roomy.layout.projection.panel_box().height();
        let selected = measure(band + 21.0).unwrap();
        assert!(selected.layout.projection.panel_box().height() < 24.0);
        let crate::layout::interior::MarkPayload::Waterfall(payload) =
            &selected.marks.as_ref().unwrap().payload
        else {
            panic!("Waterfall payload")
        };
        assert_eq!(payload.bars.len(), 1);
        assert_eq!(payload.text.len(), 2);
    }

    #[test]
    fn total_ink_keeps_exact_numeric_targets_and_only_expands_automatic_x() {
        let source = |exact| {
            let chart = Plot::waterfall(
                CategoryValues::new("step", ["gain", "loss"]).unwrap(),
                NumericValues::new("delta", [1.0, -0.5]),
                0.0,
            )
            .total_labels(crate::WaterfallLabels::new("Initial total", "Final total").unwrap())
            .size(900, 900);
            if exact {
                chart.x_limits(0.0, 1.0).erase()
            } else {
                chart.erase()
            }
        };
        let panel = PanelBox::new(180.0, 780.0, 80.0, 800.0);
        let mut theme = crate::Theme::Publication.resolve();
        theme.typography.waterfall_total.font_size = 48;
        for exact in [false, true] {
            let chart = source(exact);
            let inputs =
                crate::plots::shared_x::prepare_plot(&chart, &OperationGenerationResources::new())
                    .unwrap();
            let chart = inputs.attach(chart).unwrap();
            let selected = chart.measure_selected_interior(PlotRenderContext::new(
                chart.data(),
                panel,
                &theme,
            ));
            if exact {
                assert!(
                    matches!(selected, Err(crate::layout::fit::PlanError::Fit(failure))
                    if failure.reason.kind == crate::layout::fit::FitKind::Text)
                );
            } else {
                let selected = selected.unwrap();
                let crate::layout::interior::MarkPayload::Waterfall(payload) =
                    &selected.marks.as_ref().unwrap().payload
                else {
                    panic!("selected Waterfall")
                };
                assert!(payload.domain_expansions[0].is_some());
                let final_plan = crate::decorations::plan::plan_panel_for_test(
                    &chart,
                    crate::PlotKind::Waterfall,
                    panel,
                    crate::decorations::plan::LegendRole::Panel,
                    &theme,
                )
                .unwrap();
                let crate::layout::interior::MarkPayload::Waterfall(final_payload) =
                    &final_plan.result.marks.as_ref().unwrap().payload
                else {
                    panic!("final Waterfall")
                };
                assert!(final_payload.domain_expansions.iter().all(Option::is_none));
                let projected = final_plan.result.layout.projection.project(0.0, 0.0).0;
                let base_x = nearest_spatial_quanta(projected).unwrap() as f64 / 1024.0;
                assert_eq!(
                    final_payload.text[0].placement.position.x.to_bits(),
                    base_x.to_bits()
                );
            }
        }
    }

    #[test]
    fn styled_waterfall_bars_clear_automatic_numeric_ink() {
        let source = Plot::waterfall(
            CategoryValues::new("step", ["rise", "fall"]).unwrap(),
            NumericValues::new("delta", [2.0, -1.0]),
            0.0,
        )
        .size(800, 900)
        .erase();
        let mut theme = crate::Theme::Publication.resolve();
        theme.marks.waterfall_bar.outline_width = 80.0;
        theme.marks.waterfall_connector.width = 16.0;
        let resources = OperationGenerationResources::new();
        let inputs = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
        let chart = inputs.attach(source).unwrap();
        let cell = PanelBox::new(80.0, 700.0, 80.0, 800.0);
        let selected = crate::decorations::plan::plan_panel_for_test(
            &chart,
            crate::PlotKind::Waterfall,
            cell,
            crate::decorations::plan::LegendRole::Panel,
            &theme,
        )
        .unwrap();
        assert!(selected.planned_x_limits().is_some());
        let crate::layout::interior::MarkPayload::Waterfall(payload) =
            &selected.result.marks.as_ref().unwrap().payload
        else {
            panic!("selected Waterfall")
        };
        assert!(payload.domain_expansions.iter().all(Option::is_none));
        let panel = selected.result.layout.projection.panel_box();
        let mut ink = crate::render::geometry::GeometrySnapshot::new(&theme).unwrap();
        for (corners, style) in &payload.bars {
            ink.polygon(corners, style, None).unwrap();
        }
        for (points, style) in &payload.connectors {
            ink.line(points, style, None).unwrap();
        }
        for bounds in ink.bounds() {
            assert!(bounds.bottom >= panel.y0 && bounds.top <= panel.y1);
            assert!(bounds.left >= panel.x0 && bounds.right <= panel.x1);
        }
    }

    #[test]
    fn custom_total_typography_reallocates_shelf_without_changing_span_inputs() {
        let source = Plot::waterfall(
            CategoryValues::new("step", ["change"]).unwrap(),
            NumericValues::new("delta", [2.0]),
            1.0,
        )
        .size(1200, 800)
        .erase();
        let resources = OperationGenerationResources::new();
        let inputs = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
        let chart = inputs.attach(source).unwrap();
        let panel = PanelBox::new(80.0, 750.0, 60.0, 500.0);
        let mut theme = crate::Theme::Publication.resolve();
        let old = chart
            .measure_selected_interior(PlotRenderContext::new(chart.data(), panel, &theme))
            .unwrap();
        theme.typography.waterfall_total.font_size = 18;
        theme.typography.waterfall_total.paint = crate::theme::Color::rgb(3, 5, 7);
        let selected = chart
            .measure_selected_interior(PlotRenderContext::new(chart.data(), panel, &theme))
            .unwrap();
        let extract = |result: &PlotRenderResult| {
            let crate::layout::interior::MarkPayload::Waterfall(payload) =
                &result.marks.as_ref().unwrap().payload
            else {
                panic!("selected Waterfall")
            };
            (
                payload
                    .bars
                    .iter()
                    .map(|(corners, _)| *corners)
                    .collect::<Vec<_>>(),
                payload
                    .text
                    .iter()
                    .map(|text| text.plan.style.clone())
                    .collect::<Vec<_>>(),
            )
        };
        let (old_bars, old_text) = extract(&old);
        let (new_bars, new_text) = extract(&selected);
        assert_eq!(old_bars.len(), new_bars.len());
        assert_ne!(old_bars, new_bars);
        assert_eq!(old_bars[0][0].x.to_bits(), new_bars[0][0].x.to_bits());
        assert_eq!(old_text.len(), 2);
        assert_eq!(old_text[0].font_size, 14);
        assert!(new_text
            .iter()
            .all(|text| text.font_size == 18 && text.paint == crate::theme::Color::rgb(3, 5, 7)));
    }

    #[test]
    fn projected_spans_reject_nonzero_collapse_but_retain_signed_zero_deltas() {
        let range = (-10_000_000_000_000_000.0, 20_000_000_000_000_000.0);
        let panel = PanelBox::new(80.0, 700.0, 60.0, 500.0);
        for base in [0.0, 10_000_000_000_000_000.0] {
            for delta in [0.0, -0.0, 1.0, -1.0] {
                let plot = Plot::waterfall(
                    CategoryValues::new("step", ["change"]).unwrap(),
                    NumericValues::new("delta", [delta]),
                    base,
                )
                .x_limits(range.0, range.1)
                .erase();
                let inputs = crate::plots::shared_x::prepare_plot(
                    &plot,
                    &OperationGenerationResources::new(),
                )
                .unwrap();
                let span = inputs.waterfall.as_ref().unwrap().panels[0].1.spans[0];
                let (start, end, source_delta) = (span.start, span.end, span.delta);
                assert_eq!(source_delta.to_bits(), delta.to_bits());
                let projection =
                    crate::layout::Projection::new(range, (0.0, 1.0), panel.as_tuple());
                assert_eq!(
                    nearest_spatial_quanta(projection.project(start, 0.2).0).unwrap(),
                    nearest_spatial_quanta(projection.project(end, 0.8).0).unwrap()
                );
                let plot = inputs.attach(plot).unwrap();
                let theme = crate::Theme::Publication.resolve();
                let result = plot.measure_selected_interior(PlotRenderContext::new(
                    plot.data(),
                    panel,
                    &theme,
                ));
                if delta == 0.0 {
                    let selected = result.unwrap();
                    let crate::layout::interior::MarkPayload::Waterfall(payload) =
                        &selected.marks.as_ref().unwrap().payload
                    else {
                        panic!("wrong payload")
                    };
                    assert_eq!(payload.bars.len(), 1);
                    let points = &payload.bars[0].0;
                    assert!(points
                        .iter()
                        .all(|point| point.x.to_bits() == points[0].x.to_bits()));
                    assert!(points[0].y < points[3].y);
                } else {
                    let Err(PlanError::Fatal(Error::Render(message))) = result else {
                        panic!("nonzero span must fail fatally before retaining marks: {base} + {delta}")
                    };
                    assert!(
                        message.contains("polygon") && message.contains("collapse"),
                        "{message}"
                    );
                }
            }
        }
    }
}
