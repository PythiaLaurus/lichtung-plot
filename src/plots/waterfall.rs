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
    spans: Vec<(f64, f64, f64)>,
    pub(crate) totals: [(f64, String); 2],
}

pub(crate) fn prepare(
    chart: &Plot,
    data: crate::data::PlotData<'_>,
) -> Result<PreparedWaterfall, Error> {
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
        spans.push((running, next, *delta));
        running = next;
    }
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

pub(crate) fn measure(
    chart: &Plot,
    context: PlotRenderContext<'_>,
) -> Result<PlotRenderResult, crate::layout::fit::PlanError> {
    #[cfg(test)]
    count(1);
    let prepared = chart
        .prepared_waterfall
        .as_ref()
        .and_then(|inputs| {
            inputs
                .panels
                .iter()
                .find(|(scope, _)| scope.as_deref() == chart.axis_scope())
        })
        .map(|(_, marks)| marks)
        .ok_or_else(|| Error::Render("waterfall requires root-prepared semantics".into()))?;
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
    let y_ticks = || {
        data.collect_source(labels.iter().enumerate().map(|(index, label)| {
            data.claim_source_visits(1)?;
            Ok(AxisTick::new(
                index as f64 + 0.5,
                data.copy_source_string(label)?,
            ))
        }))
    };
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
    if annotations.height > panel.height() - MINIMUM_MARK_PANEL_HEIGHT {
        data.claim_source_bytes(context.fit_scope().address.storage_bytes())?;
        data.claim_source_bytes(1024)?;
        return Err(context.fit_scope().failure(crate::layout::fit::FitKind::SpecializedGeometry, format!(
            "waterfall: total annotations require {:.0}px while preserving a {MINIMUM_MARK_PANEL_HEIGHT:.0}px mark panel; increase chart height or shorten the configured labels",
            annotations.height
        )).into());
    }
    let mut mark_panel = panel;
    mark_panel.y1 -= annotations.height;
    let layout = context.axis_layout(|| {
        let x_axis_spec = crate::layout::display_axis_spec(chart.x_axis_spec(), x_range);
        context.layout_with_ticks(
            x_range,
            (0.0, labels.len() as f64),
            mark_panel,
            data.copy_source_string(chart.x_label_text().unwrap_or(""))?,
            data.copy_source_string(chart.y_label_text().unwrap_or(""))?,
            Vec::new(),
            y_ticks()?,
            &x_axis_spec,
            chart.y_axis_spec(),
            theme,
        )
    })?;
    let mut bars = Vec::new();
    let mut connectors = Vec::new();
    data.reserve_source(&mut bars, spans.len())?;
    data.reserve_source(&mut connectors, spans.len().saturating_sub(1))?;
    for (index, (start, end, delta)) in spans.iter().enumerate() {
        let y0 = index as f64 + 0.2;
        let y1 = index as f64 + 0.8;
        let (left, bottom) = layout.projection.project(*start, y0);
        let (right, top) = layout.projection.project(*end, y1);
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
        let color = if *delta >= 0.0 {
            theme.colors[0]
        } else {
            theme.colors[1 % theme.colors.len()]
        };
        bars.push((
            [
                Point2D::new(left, bottom),
                Point2D::new(right, bottom),
                Point2D::new(right, top),
                Point2D::new(left, top),
            ],
            FillStyle {
                color,
                stroke: theme.foreground,
                stroke_width: 0.8,
                opacity: 1.0,
            },
        ));
        if index + 1 < spans.len() {
            let (x, from_y) = layout.projection.project(*end, y1);
            let (_, to_y) = layout.projection.project(*end, index as f64 + 1.2);
            connectors.push((
                [Point2D::new(x, from_y), Point2D::new(x, to_y)],
                LineStyle::short_dashed(theme.foreground, 0.7),
            ));
        }
    }
    data.claim_source_visits(2)?;
    let references = [base, terminal].map(|value| {
        let (x, y0) = layout.projection.project(value, 0.0);
        let (_, y1) = layout.projection.project(value, labels.len() as f64);
        (
            [Point2D::new(x, y0), Point2D::new(x, y1)],
            LineStyle::medium_dashed(theme.foreground, 0.7),
        )
    });
    let AnnotationPlan {
        items, row_height, ..
    } = annotations;
    let mut text = Vec::new();
    data.reserve_source(&mut text, items.len())?;
    for annotation in items {
        let text_plan = annotation.text_plan;
        text.push(PlacedText {
            placement: PlannedTextPlacement::middle(
                Point2D::new(
                    annotation.x.trunc(),
                    (layout.projection.panel_box().y1
                        + ANNOTATION_PADDING
                        + row_height / 2.0
                        + annotation.row as f64 * (row_height + ANNOTATION_ROW_GAP))
                        .round(),
                ),
                TextAnchor::Middle,
                0.0,
            ),
            plan: text_plan,
        });
    }
    let payload = WaterfallPayload {
        clip: layout.projection.panel_box(),
        bars,
        connectors,
        references,
        text,
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
    let mut result = PlotRenderResult::new(layout, Vec::new());
    result.y_domain = crate::spec::AxisDomain::Categorical;
    result.producer_text = data.collect_source(payload.text.iter().map(|text| Ok(text.clone())))?;
    result.marks = Some(data.source_arc(crate::layout::interior::DeferredMarkPlan {
        payload: crate::layout::interior::MarkPayload::Waterfall(payload),
        geometry,
    })?);
    Ok(result)
}

#[derive(Debug)]
pub(crate) struct WaterfallPayload {
    clip: PanelBox,
    bars: Vec<([Point2D; 4], FillStyle)>,
    connectors: Vec<([Point2D; 2], LineStyle)>,
    references: [([Point2D; 2], LineStyle); 2],
    text: Vec<PlacedText>,
}

impl WaterfallPayload {
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

const ANNOTATION_PADDING: f64 = 4.0;
const ANNOTATION_GAP: f64 = 4.0;
const ANNOTATION_ROW_GAP: f64 = 4.0;
const MINIMUM_MARK_PANEL_HEIGHT: f64 = 24.0;

#[cfg(test)]
mod extraction;

struct Annotation {
    text_plan: ResolvedTextPlan,
    x: f64,
    row: usize,
}

struct AnnotationPlan {
    items: Vec<Annotation>,
    height: f64,
    row_height: f64,
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
    let mut rows: Vec<Vec<(f64, f64)>> = Vec::new();
    let mut items = Vec::new();
    let mut miss = None;
    data.reserve_source(&mut items, 2)?;
    data.reserve_source(&mut rows, 2)?;
    for (index, (value, text)) in candidates.iter().enumerate() {
        if !projection.contains_data(*value, source_y) {
            continue;
        }
        let text_plan =
            context
                .producer_text()?
                .plan(text, "waterfall total annotation", || {
                    Ok(theme.measure_legacy_text(
                        text,
                        theme.detail_font_size,
                        PhysicalTextAlignment::Center,
                        "waterfall total annotation",
                    )?)
                })?;
        let width = text_plan.width();
        if width + 2.0 * ANNOTATION_PADDING > panel.width() {
            let identity_bytes = scope
                .address
                .storage_bytes()
                .checked_add(std::mem::size_of::<crate::layout::kernel::AddressSegment>())
                .and_then(|bytes| bytes.checked_mul(2))
                .ok_or_else(|| {
                    Error::Render("waterfall: diagnostic identity size overflow".into())
                })?;
            data.claim_source_bytes(identity_bytes)?;
            // Debug escaping can expand each source byte; admit both the text
            // diagnostic and its child/failure address copies before allocation.
            let diagnostic_bytes = text
                .len()
                .checked_mul(10)
                .and_then(|bytes| bytes.checked_add(1024))
                .ok_or_else(|| Error::Render("waterfall: diagnostic text size overflow".into()))?;
            data.claim_source_visits(diagnostic_bytes)?;
            data.claim_source_bytes(diagnostic_bytes)?;
            crate::layout::fit::FitFailure::retain_least(&mut miss, scope.child(if index == 0 { crate::layout::AddressRole::WaterfallBaseLabel } else { crate::layout::AddressRole::WaterfallTerminalLabel }, 0)?.failure(crate::layout::fit::FitKind::Text, format!(
                "waterfall: total annotation {text:?} requires {:.0}px but only {:.0}px is available; increase chart width or shorten the configured label",
                width + 2.0 * ANNOTATION_PADDING,
                panel.width()
            )));
            continue;
        }
        let (projected_x, _) = projection.project(*value, source_y);
        let half_width = width / 2.0;
        let x = projected_x
            .clamp(
                panel.x0 + ANNOTATION_PADDING + half_width,
                panel.x1 - ANNOTATION_PADDING - half_width,
            )
            .round()
            .clamp(
                panel.x0 + ANNOTATION_PADDING + half_width,
                panel.x1 - ANNOTATION_PADDING - half_width,
            );
        let bounds = (x - half_width, x + half_width);
        let row = rows
            .iter()
            .position(|occupied| {
                occupied.iter().all(|(left, right)| {
                    bounds.1 + ANNOTATION_GAP <= *left || *right + ANNOTATION_GAP <= bounds.0
                })
            })
            .unwrap_or(rows.len());
        if row == rows.len() {
            rows.push(Vec::new());
        }
        data.reserve_source(&mut rows[row], 1)?;
        rows[row].push(bounds);
        items.push(Annotation { text_plan, x, row });
    }
    if let Some(failure) = miss {
        return Err(failure.into());
    }
    let row_height = items
        .iter()
        .map(|annotation| annotation.text_plan.height())
        .fold(0.0_f64, f64::max);
    let height = if rows.is_empty() {
        0.0
    } else {
        2.0 * ANNOTATION_PADDING
            + rows.len() as f64 * row_height
            + rows.len().saturating_sub(1) as f64 * ANNOTATION_ROW_GAP
    };
    Ok(AnnotationPlan {
        items,
        height,
        row_height,
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
                let (start, end, source_delta) =
                    inputs.waterfall.as_ref().unwrap().panels[0].1.spans[0];
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
