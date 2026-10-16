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

fn retained(chart: &Plot) -> Result<&PreparedWaterfall, Error> {
    let inputs = chart
        .prepared_waterfall
        .as_ref()
        .ok_or_else(|| Error::Render("Waterfall requires root-prepared spans".into()))?;
    crate::layout::interior::preference::panel(&inputs.panels, chart.axis_scope(), "Waterfall")
}

fn mark_intrinsic_body_minimum(
    chart: &Plot,
    theme: &crate::theme::ResolvedTheme,
) -> Result<(f64, f64), Error> {
    let prepared = retained(chart)?;
    if prepared.labels.is_empty() || prepared.labels.len() != prepared.spans.len() {
        return Err(Error::Render(
            "Waterfall categories and scientific spans differ".into(),
        ));
    }

    let positive = theme
        .marks
        .fills()
        .bodies()
        .waterfall_bar()
        .backend_style((0, 0, 0));
    let negative = positive.clone();
    let positive_ink = crate::render::geometry::rectangular_fill_ink_bounds(&positive)?;
    let negative_ink = crate::render::geometry::rectangular_fill_ink_bounds(&negative)?;
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
            &theme
                .marks
                .strokes()
                .connectors()
                .waterfall_connector()
                .backend_style(),
        )?;
        pitch = pitch.max(connector / 0.4);
        numeric = numeric.max(connector);
    }
    numeric = numeric.max(crate::render::geometry::line_ink_width(
        &theme
            .marks
            .strokes()
            .references()
            .waterfall_reference()
            .backend_style(),
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
    let prepared = retained(chart)?;

    let mut height = 0.0_f64;
    for (value, text) in &prepared.totals {
        if chart
            .user_x_limits()
            .is_some_and(|range| *value < range.0 || *value > range.1)
        {
            continue;
        }
        let plan = theme.measure_text(
            text,
            &theme.typography.annotations.waterfall_total,
            PhysicalTextAlignment::Center,
            "Waterfall total shelf intrinsic",
            false,
        )?;
        height = height.max(plan.height());
    }
    let shelf = if height > 0.0 {
        theme
            .annotations
            .waterfall_shelf()
            .lengths(theme.typography.annotations.waterfall_total.font_size())?
            .padding
            .times(2)
            .add_pixels(height)?
            .pixels_rounded(crate::layout::kernel::SpatialRounding::Ceil)?
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
    let resources = theme.prepared_font_environment()?.operation();
    let inputs = chart
        .prepared_waterfall
        .as_ref()
        .ok_or_else(|| Error::Render("natural Waterfall rows missing".into()))?;
    let marks = crate::layout::interior::preference::panel(
        &inputs.panels,
        chart.axis_scope(),
        "natural Waterfall",
    )?;
    let axes = chart
        .prepared_axis_inputs()
        .ok_or_else(|| Error::Render("natural Waterfall axes missing".into()))?;
    let range = chart.x_axis_spec().limits.unwrap_or(axes.ranges[0]);
    let panel = PanelBox::new(0.0, width, 0.0, 1.0);
    let projection =
        crate::layout::Projection::new(range, (0.0, marks.labels.len() as f64), panel.as_tuple());
    let data = chart.data().with_operation(&resources);
    let plan = plan_annotations(
        &projection,
        panel,
        marks.labels.len() as f64,
        &marks.totals,
        PlotRenderContext::new(data, panel, theme),
    )
    .map_err(crate::layout::fit::PlanError::into_error)?;
    let hard = mark_intrinsic_body_minimum(chart, theme)?.1.max(
        crate::layout::interior::coordinate::Intrinsics::prepared(chart, theme)?
            .minimum(chart)
            .1,
    );
    Ok(hard
        .max(
            2.0 * f64::from(theme.typography.axes.tick_label.font_size) * marks.labels.len() as f64,
        )
        .max(3.0 * width / 4.0)
        + plan.height)
}

pub(crate) fn prepare(
    chart: &Plot,
    data: crate::data::PlotData<'_>,
) -> Result<PreparedWaterfall, Error> {
    data.source_resources()?;
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
    data.validate_row_source()?;
    // Category slots, connectors, and the two named total targets survive
    // caller-window clipping. Bar records are the original row slots above.

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
        crate::data::source::reserve_source_string(
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
    crate::data::source::reserve_source(&mut spans, deltas.len())?;

    let mut range = (base, base);
    for (index, delta) in deltas.iter().enumerate() {
        let next = running + delta;
        if !next.is_finite() {
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
    let prepared = retained(chart)?;
    let theme = context.theme;

    let labels = &prepared.labels;
    let spans = &prepared.spans;
    let range = prepared.range;
    let base = prepared.totals[0].0;
    let terminal = prepared.totals[1].0;
    let x_range = context.axis_range(crate::layout::AxisSide::Bottom, || {
        crate::layout::continuous_auto_range(range, chart.x_axis_spec())
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
    )?;
    let mark_minimum = mark_intrinsic_body_minimum(chart, theme)?;
    let axes_minimum =
        crate::layout::interior::coordinate::Intrinsics::prepared(chart, theme)?.minimum(chart);
    if annotations.height > panel.height() - mark_minimum.1.max(axes_minimum.1) {
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
            let plan = crate::layout::fit::measured(crate::layout::fit::text(
                theme,
                label,
                &theme.typography.axes.tick_label,
                width,
                PhysicalTextAlignment::End,
                "y-axis tick label",
                &context
                    .fit_scope()
                    .child(crate::layout::AddressRole::AxisLeft, 0)?,
            ))?;
            pitch = pitch.max(plan.height() + 2.0 * crate::layout::LABEL_CLEARANCE);
        }
    }
    let min = crate::layout::kernel::ceil_spatial_quanta(pitch)?;
    let preferred = min.max(crate::layout::kernel::ceil_spatial_quanta(
        2.0 * f64::from(theme.typography.axes.tick_label.font_size),
    )?);
    let specs = crate::data::source::collect_source(labels.iter().map(|_| {
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
    let ticks =
        crate::data::source::collect_source(display.iter().zip(&bands).map(|(label, band)| {
            Ok(AxisTick::new(
                (f64::midpoint(band.0, band.1) - mark_panel.y0) / mark_panel.height()
                    * labels.len() as f64,
                if chart.y_tick_labels_visible() {
                    crate::data::source::copy_source_string(label)?
                } else {
                    String::new()
                },
            ))
        }))?;
    let layout = crate::layout::Layout {
        projection,
        x_axis: crate::layout::Axis {
            label: crate::data::source::copy_source_string(&axes.labels[0])?,
            ticks: x.ticks,
            source: x.source,
            explicit_ticks: axes.specs[0].exact_ticks.is_some(),
            assigned_tick_width: None,
            assigned_cells: None,
        },
        y_axis: crate::layout::Axis {
            label: crate::data::source::copy_source_string(&axes.labels[1])?,
            ticks,
            source: crate::layout::TickSource::SUPPLIED,
            explicit_ticks: false,
            assigned_tick_width: Some(width),
            assigned_cells: Some(crate::data::source::collect_source(
                bands.iter().map(|band| Ok(*band)),
            )?),
        },
    };
    let mut bars = Vec::new();
    let mut source_rows = Vec::new();
    let mut connectors = Vec::new();
    let accents = crate::theme::AccentColors::resolve(theme.marks.accents(), 2)?;
    crate::data::source::reserve_source(&mut bars, spans.len())?;
    crate::data::source::reserve_source(&mut source_rows, spans.len())?;
    crate::data::source::reserve_source(&mut connectors, spans.len().saturating_sub(1))?;
    for (index, span) in spans.iter().enumerate() {
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
            theme
                .marks
                .fills()
                .bodies()
                .waterfall_bar()
                .backend_style(color),
        ));
        source_rows.push(span.source_row);
        if index + 1 < spans.len() {
            let x = right;
            let from_y = top;
            let next = bands[index + 1];
            let to_y = next.0 + 0.2 * (next.1 - next.0);
            connectors.push((
                [Point2D::new(x, from_y), Point2D::new(x, to_y)],
                theme
                    .marks
                    .strokes()
                    .connectors()
                    .waterfall_connector()
                    .backend_style(),
            ));
        }
    }

    let references = [base, terminal].map(|value| {
        let (x, y0) = layout.projection.project(value, 0.0);
        let (_, y1) = layout.projection.project(value, labels.len() as f64);
        (
            [Point2D::new(x, y0), Point2D::new(x, y1)],
            theme
                .marks
                .strokes()
                .references()
                .waterfall_reference()
                .backend_style(),
        )
    });
    let AnnotationPlan {
        items, baselines, ..
    } = annotations;
    let mut text = Vec::new();
    let mut text_targets = Vec::new();
    crate::data::source::reserve_source(&mut text, items.len())?;
    crate::data::source::reserve_source(&mut text_targets, items.len())?;
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
    let mut geometry = crate::render::geometry::GeometrySnapshot::new();
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
    let mut label_ink = crate::render::geometry::GeometrySnapshot::new();

    for (index, text) in payload.text.iter().enumerate() {
        let mut one = crate::render::geometry::GeometrySnapshot::new();
        one.text(text.placement, &text.plan, None)?;
        for ink in one.bounds() {
            if ink.bottom < mark_panel.y1
                || ink.top > panel.y1
                || (chart.user_x_limits().is_some()
                    && (ink.left < panel.x0 || ink.right > panel.x1))
            {
                return Err(context
                    .fit_scope()
                    .child(payload.text_targets[index], 0)?
                    .failure(
                        crate::layout::fit::FitKind::Text,
                        "required Waterfall total ink exceeds its assigned annotation shelf",
                    )
                    .into());
            }
        }
        label_ink.append(&one)?;
    }
    let mut full = crate::render::geometry::GeometrySnapshot::new();

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
        Some(crate::data::source::collect_source(labels.iter().map(
            |label| crate::data::source::copy_source_string(label),
        ))?);
    result.axes = Some(result.layout.measure_axes_with_inputs(
        theme,
        context.fit_scope(),
        chart.axis_semantic_inputs([result.x_domain, result.y_domain]),
        || crate::layout::StandardPanelInputs::new(theme),
    )?);
    result.producer_text =
        crate::data::source::collect_source(payload.text.iter().map(|text| Ok(text.clone())))?;
    result.marks = Some(crate::data::source::source_arc(
        crate::layout::interior::SelectedMarkPlan {
            payload: crate::layout::interior::MarkPayload::Waterfall(payload),
            geometry,
        },
    )?);
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
) -> Result<AnnotationPlan, crate::layout::fit::PlanError> {
    let theme = context.theme;
    let scope = context.fit_scope();
    let mut rows: Vec<ShelfRow> = Vec::new();
    let mut items = Vec::new();
    let mut miss = None;
    crate::data::source::reserve_source(&mut items, 2)?;
    crate::data::source::reserve_source(&mut rows, 2)?;
    let metrics = theme
        .annotations
        .waterfall_shelf()
        .lengths(theme.typography.annotations.waterfall_total.font_size())?;
    let gap = metrics
        .gap
        .round(crate::layout::kernel::SpatialRounding::Ceil)?
        .quanta();
    let producer = context.producer_text()?;
    for (index, (value, text)) in candidates.iter().enumerate() {
        let target = if index == 0 {
            crate::layout::AddressRole::WaterfallBaseLabel
        } else {
            crate::layout::AddressRole::WaterfallTerminalLabel
        };

        if !projection.contains_data(*value, source_y) {
            continue;
        }
        let text_plan = producer.plan(text, "waterfall total annotation", || {
            Ok(theme.measure_text(
                text,
                &theme.typography.annotations.waterfall_total,
                PhysicalTextAlignment::Center,
                "waterfall total annotation",
                false,
            )?)
        })?;
        let width = text_plan.width();
        let required_width = metrics
            .padding
            .times(2)
            .add_pixels(width)?
            .pixels_rounded(crate::layout::kernel::SpatialRounding::Ceil)?;
        if required_width > panel.width() {
            // Debug escaping can expand each source byte; admit both the text
            // diagnostic before allocation; child paths have their own admission.

            crate::layout::fit::FitFailure::retain_least(&mut miss, scope.child(if index == 0 { crate::layout::AddressRole::WaterfallBaseLabel } else { crate::layout::AddressRole::WaterfallTerminalLabel }, 0)?.failure(crate::layout::fit::FitKind::Text, format!(
                "waterfall: total annotation {text:?} requires {:.0}px but only {:.0}px is available; increase chart width or shorten the configured label",
                required_width,
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
            let mut fits = true;
            for &(left, right) in &occupied.intervals {
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
        crate::data::source::reserve_source(&mut rows[row].intervals, 1)?;
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
    crate::data::source::reserve_source(&mut baselines, rows.len())?;
    let mut height = metrics.padding.times(u32::from(!rows.is_empty()));
    for (index, row) in rows.iter().enumerate() {
        let baseline =
            height.add_quanta(crate::layout::kernel::LayoutScalar::from_quanta(row.below))?;
        baselines
            .push(baseline.pixels_rounded(crate::layout::kernel::SpatialRounding::NearestEven)?);
        height = baseline
            .add_quanta(crate::layout::kernel::LayoutScalar::from_quanta(row.above))?
            .add_length(if index + 1 == rows.len() {
                &metrics.padding
            } else {
                &metrics.row_gap
            });
    }
    let height = height.pixels_rounded(crate::layout::kernel::SpatialRounding::Ceil)?;
    Ok(AnnotationPlan {
        items,
        height,
        baselines,
    })
}
