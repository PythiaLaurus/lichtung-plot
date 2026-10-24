//! Cumulative signed-delta waterfall plot.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::too_many_lines
)]

use crate::alpha::{
    marks::{fill_style, ObservationAlpha},
    AlphaTarget, AlphaUnitKind,
};
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
    total_candidates: [(f64, crate::Text); 2],
    total_format: Option<std::sync::Arc<crate::numeric::format::NumericFormat>>,
    total_variants: Vec<Option<totals::TotalVariant>>,
}

impl PreparedWaterfall {
    pub(crate) fn alpha_guide(
        &self,
        _chart: &Plot,
        theme: &crate::theme::ResolvedTheme,
        guide: &mut crate::decorations::legend::alpha::AlphaGuideBuilder<'_>,
    ) -> crate::Result<()> {
        use crate::decorations::legend::alpha::{AlphaGlyphRole, AlphaUnit};
        let fill = theme.marks.get_fills().get_bodies().get_waterfall_bar();
        for span in &self.spans {
            guide.fill(
                AlphaGlyphRole::Fill,
                AlphaUnitKind::Observation,
                AlphaUnit::observation(span.alpha),
                span.alpha.value,
                fill.backend_style((0, 0, 0)),
                fill.get_outline()
                    .is_some_and(|outline| outline.mapped_width().is_some()),
            )?;
        }
        Ok(())
    }
}

mod totals;

#[derive(Debug, Clone, Copy, PartialEq)]
struct WaterfallSpan {
    alpha: ObservationAlpha,
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

struct BodyGeometry {
    breadth: f64,
    low: f64,
    high: f64,
    connector_room: f64,
}

#[cfg(test)]
mod geometry_tests {
    use super::*;

    #[test]
    fn authored_totals_keep_independent_styles_at_equal_sources_and_size_the_shared_shelf() {
        use crate::{FontFamily, FontStack, Text, TextAlignment, WaterfallLabels};
        let mut theme = crate::Theme::publication();
        let role = theme.typography.annotations.waterfall_total.clone();
        let first = Text::new("Endpoint\nshort")
            .style(role.clone().font_size(23).unwrap())
            .alignment(TextAlignment::Start);
        let second = Text::new("Endpoint\nshort")
            .style(role.font_size(17).unwrap())
            .alignment(TextAlignment::End);
        let labels = WaterfallLabels::new(first.clone(), second.clone()).unwrap();
        assert_eq!(labels.base(), labels.terminal());
        assert_ne!(labels, WaterfallLabels::new(first.clone(), first).unwrap());
        theme.typography.annotations.waterfall_total.stack =
            FontStack::new([FontFamily::named("Unused Waterfall total font").unwrap()]).unwrap();
        let one = crate::RelativeSpacing::ratio(1, 1).unwrap();
        theme.annotations = theme
            .annotations
            .clone()
            .waterfall_shelf(crate::ValueShelfSpacing::new(one.clone(), one.clone(), one).unwrap())
            .unwrap();
        let source = Plot::waterfall(
            crate::CategoryValues::new("step", ["one"]).unwrap(),
            crate::NumericValues::new("delta", [0.0]),
            10.0,
        )
        .total_labels(labels)
        .total_format(
            crate::NumericFormat::new(
                crate::NumericNotation::Decimal,
                crate::NumericPrecision::Fractional(0),
            )
            .unwrap(),
        )
        .x_limits(0.0, 20.0)
        .theme(theme.clone());
        source.preflight().unwrap();
        let resolved = theme.resolve().unwrap();
        let chart =
            crate::plots::shared_x::prepare_panel_for_test(&source.erase(), &resolved).unwrap();
        let marks = retained(&chart).unwrap();
        let panel = PanelBox::new(0.0, 800.0, 0.0, 400.0);
        let projection = crate::layout::Projection::new((0.0, 20.0), (0.0, 1.0), panel.as_tuple());
        let totals = marks.selected_totals(&projection).unwrap();
        assert!(totals
            .totals
            .iter()
            .all(|(_, text)| text.as_ref().unwrap().as_str() == "Endpoint\nshort 10"));
        let shelf = plan_annotations(
            &projection,
            panel,
            1.0,
            &totals.totals,
            PlotRenderContext::new(chart.data(), panel, &resolved),
        )
        .unwrap();
        assert_eq!(shelf.items.len(), 2);
        assert_ne!(shelf.items[0].row, shelf.items[1].row);
        assert_eq!(shelf.items[0].text_plan.style.font_size, 23);
        assert_eq!(shelf.items[1].text_plan.style.font_size, 17);
        assert_eq!(shelf.items[0].anchor, TextAnchor::Start);
        assert_eq!(shelf.items[1].anchor, TextAnchor::End);
        assert!(shelf
            .items
            .iter()
            .all(|item| item.text_plan.style.exact_override));
        let text_height: f64 = shelf.items.iter().map(|item| item.text_plan.height()).sum();
        assert!((shelf.height - text_height - 3.0 * 23.0).abs() <= 1.0 / 1024.0);
    }

    #[test]
    fn selected_breadth_controls_polygons_and_refuses_only_positive_ink_without_room() {
        let source = Plot::waterfall(
            crate::CategoryValues::new("step", ["first", "second"]).unwrap(),
            crate::NumericValues::new("delta", [2.0, -1.0]),
            10.0,
        )
        .source_rows(crate::SourceRows::new([400, 100]).unwrap())
        .with_alpha(
            AlphaTarget::Fill,
            crate::alpha::AlphaEncoding::observations(
                crate::alpha::AlphaValues::new("step confidence", [0, 230]).unwrap(),
            ),
        )
        .x_limits(0.0, 20.0)
        .erase();
        let operation = crate::font::canonical::OperationContext::new();
        let inputs = crate::plots::shared_x::prepare_plot(&source, &operation, None).unwrap();
        let mut theme = crate::theme::Theme::publication().resolve().unwrap();
        let chart = inputs.attach_with_guides_for_test(source, &theme).unwrap();
        let default = BodyGeometry::new(&theme);
        assert_eq!(
            [
                default.breadth,
                default.low,
                default.high,
                default.connector_room
            ]
            .map(f64::to_bits),
            [0.6_f64, 0.2, 0.8, 0.4].map(f64::to_bits)
        );
        let panel = crate::layout::PanelBox::new(10.0, 810.0, 20.0, 620.0);
        let measure = |theme: &crate::theme::ResolvedTheme| {
            chart
                .measure_selected_interior(PlotRenderContext::new(chart.data(), panel, theme))
                .unwrap()
        };
        let original = measure(&theme);
        let crate::layout::interior::MarkPayload::Waterfall(original_payload) =
            &original.marks.as_ref().unwrap().payload
        else {
            panic!("Waterfall payload")
        };
        assert_eq!(original_payload.source_rows, [400, 100]);
        assert_eq!(
            original_payload
                .bars
                .iter()
                .map(|(_, style)| style.color.get_alpha())
                .collect::<Vec<_>>(),
            [0, 230]
        );
        assert_eq!(
            original_payload.connectors[0].1,
            theme
                .marks
                .get_strokes()
                .get_connectors()
                .get_waterfall_connector()
                .backend_style()
        );
        assert!(original_payload.references.iter().all(|(_, style)| *style
            == theme
                .marks
                .get_strokes()
                .get_references()
                .get_waterfall_reference()
                .backend_style()));
        let prepared = retained(&chart).unwrap();
        assert_eq!(
            prepared
                .spans
                .iter()
                .map(|span| (
                    span.alpha.slot,
                    span.alpha.source_row,
                    span.start.to_bits(),
                    span.end.to_bits()
                ))
                .collect::<Vec<_>>(),
            [
                (0, 400, 10.0_f64.to_bits(), 12.0_f64.to_bits()),
                (1, 100, 12.0_f64.to_bits(), 11.0_f64.to_bits())
            ]
        );
        let extents = theme
            .marks
            .get_extents()
            .clone()
            .waterfall_body_category_breadth(crate::theme::RelativeSpacing::new(0.3).unwrap())
            .unwrap();
        theme.marks = theme.marks.clone().extents(extents).unwrap();
        let narrower = measure(&theme);
        let payload = |plan: &PlotRenderResult| {
            let crate::layout::interior::MarkPayload::Waterfall(payload) =
                &plan.marks.as_ref().unwrap().payload
            else {
                panic!("Waterfall payload")
            };
            (payload.bars.clone(), payload.connectors.clone())
        };
        let (before, _) = payload(&original);
        let (after, connectors) = payload(&narrower);
        assert_eq!(after.len(), 2);
        assert_eq!(connectors.len(), 1);
        for ((before, _), (after, _)) in before.iter().zip(&after) {
            assert_eq!(before[0].x.to_bits(), after[0].x.to_bits());
            assert_eq!(before[1].x.to_bits(), after[1].x.to_bits());
            assert!(after[2].y - after[0].y < before[2].y - before[0].y);
        }
        assert_eq!(connectors[0].0[0].y.to_bits(), after[0].0[2].y.to_bits());
        assert_eq!(connectors[0].0[1].y.to_bits(), after[1].0[0].y.to_bits());
        let extents = theme
            .marks
            .get_extents()
            .clone()
            .waterfall_body_category_breadth(crate::theme::RelativeSpacing::ratio(1, 1).unwrap())
            .unwrap();
        theme.marks = theme.marks.clone().extents(extents).unwrap();
        assert!(mark_intrinsic_body_minimum(&chart, &theme).is_err());
        let body = theme
            .marks
            .get_fills()
            .get_bodies()
            .get_waterfall_bar()
            .clone()
            .outline(None)
            .unwrap();
        let bodies = theme
            .marks
            .get_fills()
            .get_bodies()
            .clone()
            .waterfall_bar(body)
            .unwrap();
        let fills = theme.marks.get_fills().clone().bodies(bodies).unwrap();
        theme.marks = theme.marks.clone().fills(fills).unwrap();
        assert!(mark_intrinsic_body_minimum(&chart, &theme).is_err());
        let connector = theme
            .marks
            .get_strokes()
            .get_connectors()
            .get_waterfall_connector()
            .clone()
            .color(crate::theme::Color::rgba(0, 0, 0, 0))
            .unwrap();
        let connectors = theme
            .marks
            .get_strokes()
            .get_connectors()
            .clone()
            .waterfall_connector(connector)
            .unwrap();
        let strokes = theme
            .marks
            .get_strokes()
            .clone()
            .connectors(connectors)
            .unwrap();
        theme.marks = theme.marks.clone().strokes(strokes).unwrap();
        assert!(mark_intrinsic_body_minimum(&chart, &theme).is_ok());
    }
}

impl BodyGeometry {
    fn new(theme: &crate::theme::ResolvedTheme) -> Self {
        let breadth = theme
            .marks
            .get_extents()
            .get_waterfall_body_category_breadth()
            .as_f64();
        let free = 1.0 - breadth;
        let low = free / 2.0;
        let high = 1.0 - low;
        Self {
            breadth,
            low,
            high,
            connector_room: free,
        }
    }
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

    let body = theme
        .marks
        .get_fills()
        .get_bodies()
        .get_waterfall_bar()
        .backend_style((0, 0, 0));
    let geometry = BodyGeometry::new(theme);
    let mut pitch = 1.0_f64 / 1024.0;
    let mut numeric = 0.0_f64;
    for span in &prepared.spans {
        let ink = crate::render::geometry::rectangular_fill_ink_bounds(&fill_style(
            span.alpha.value,
            body.clone(),
        ))?;
        pitch = pitch.max(crate::layout::interior::ink_pitch(
            1.0 / 1024.0,
            geometry.breadth,
        )?);
        if let Some(ink) = ink {
            pitch = pitch.max(crate::layout::interior::ink_pitch(
                (-ink.bottom).max(0.0),
                geometry.low,
            )?);
            pitch = pitch.max(crate::layout::interior::ink_pitch(
                ink.top.max(0.0),
                geometry.low,
            )?);
            numeric = numeric.max(ink.right - ink.left);
        }
    }
    if prepared.spans.len() > 1 {
        let connector = crate::render::geometry::line_ink_width(
            &theme
                .marks
                .get_strokes()
                .get_connectors()
                .get_waterfall_connector()
                .backend_style(),
        )?;
        pitch = pitch.max(crate::layout::interior::ink_pitch(
            connector,
            geometry.connector_room,
        )?);
        numeric = numeric.max(connector);
    }
    numeric = numeric.max(crate::render::geometry::line_ink_width(
        &theme
            .marks
            .get_strokes()
            .get_references()
            .get_waterfall_reference()
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
    let input = chart
        .prepared_axis_inputs()
        .ok_or_else(|| Error::Render("Waterfall total axes missing".into()))?;
    let projection = crate::layout::Projection::new(
        chart.x_axis_spec().limits.unwrap_or(input.ranges[0]),
        chart.y_axis_spec().limits.unwrap_or(input.ranges[1]),
        (0.0, 1.0, 0.0, 1.0),
    );
    let totals = &prepared.selected_totals(&projection)?.totals;
    for text in totals.iter().filter_map(|(_, text)| text.as_ref()) {
        let appearance = text.appearance(&theme.typography.annotations.waterfall_total);
        let plan = theme.measure_text(
            text.as_str(),
            appearance.theme,
            text.resolved_alignment(crate::TextAlignment::Center).into(),
            "Waterfall total shelf intrinsic",
            appearance.exact_override,
        )?;
        height = height.max(plan.height());
    }
    let shelf = if height > 0.0 {
        theme
            .annotations
            .get_waterfall_shelf()
            .lengths(total_shelf_size(totals, theme))?
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
    let projection = crate::layout::Projection::new(
        range,
        chart.y_axis_spec().limits.unwrap_or(axes.ranges[1]),
        panel.as_tuple(),
    );
    let data = chart.data().with_operation(&resources);
    let plan = plan_annotations(
        &projection,
        panel,
        marks.labels.len() as f64,
        &marks.selected_totals(&projection)?.totals,
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

    let alpha = chart.prepare_alpha(AlphaTarget::Fill, AlphaUnitKind::Observation, &[])?;
    let (spans, range) = prepare_spans(base, &deltas, data, &alpha)?;
    let running = spans.last().map_or(base, |span| span.end);
    let [base_label, terminal_label] = chart.waterfall_labels_spec().map_or_else(
        || {
            Ok::<_, Error>([
                crate::Text::from(crate::data::source::copy_source_string("base")?),
                crate::Text::from(crate::data::source::copy_source_string("total")?),
            ])
        },
        |labels| {
            let [base, terminal] = labels.contents();
            Ok([base.try_copy()?, terminal.try_copy()?])
        },
    )?;
    let total_candidates = [(base, base_label), (running, terminal_label)];
    Ok(PreparedWaterfall {
        labels,
        range,
        spans,
        total_candidates,
        total_format: chart.total_format_spec().cloned(),
        total_variants: crate::data::source::collect_source((0..4).map(|_| Ok(None)))?,
    })
}

fn prepare_spans(
    base: f64,
    deltas: &[f64],
    data: crate::data::PlotData<'_>,
    alpha: &crate::alpha::PreparedAlpha,
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
            alpha: ObservationAlpha::resolve(data, alpha, index)?,
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
    let base = prepared.total_candidates[0].0;
    let terminal = prepared.total_candidates[1].0;
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
    let selected_totals = prepared.selected_totals(&annotation_projection)?;
    let annotations = plan_annotations(
        &annotation_projection,
        panel,
        labels.len() as f64,
        &selected_totals.totals,
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
                crate::data::source::copy_source_string(label)?,
            ))
        }))?;
    let layout = crate::layout::Layout {
        projection,
        x_axis: crate::layout::Axis {
            label: axes.copy_label(0)?,
            ticks: x.ticks,
            tick_labels_visible: true,
            source: x.source,
            numeric: x.numeric,
            explicit_ticks: axes.specs[0].exact_ticks.is_some(),
            assigned_tick_width: None,
            assigned_cells: None,
        },
        y_axis: crate::layout::Axis {
            label: axes.copy_label(1)?,
            ticks,
            tick_labels_visible: chart.y_tick_labels_visible(),
            source: crate::layout::TickSource::SUPPLIED,
            numeric: None,
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
    let accents = crate::theme::AccentColors::resolve(theme.marks.get_accents(), 2)?;
    crate::data::source::reserve_source(&mut bars, spans.len())?;
    crate::data::source::reserve_source(&mut source_rows, spans.len())?;
    crate::data::source::reserve_source(&mut connectors, spans.len().saturating_sub(1))?;
    let geometry = BodyGeometry::new(theme);
    for (index, span) in spans.iter().enumerate() {
        let (start, end, delta) = (&span.start, &span.end, &span.delta);
        let y0 = index as f64 + geometry.low;
        let y1 = index as f64 + geometry.high;
        let left = layout.projection.project(*start, y0).0;
        let right = layout.projection.project(*end, y1).0;
        let band = bands[index];
        let bottom = band.0 + geometry.low * (band.1 - band.0);
        let top = band.0 + geometry.high * (band.1 - band.0);
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
            fill_style(
                span.alpha.value,
                theme
                    .marks
                    .get_fills()
                    .get_bodies()
                    .get_waterfall_bar()
                    .backend_style(color),
            ),
        ));
        source_rows.push(span.alpha.source_row);
        if index + 1 < spans.len() {
            let x = right;
            let from_y = top;
            let next = bands[index + 1];
            let to_y = next.0 + geometry.low * (next.1 - next.0);
            connectors.push((
                [Point2D::new(x, from_y), Point2D::new(x, to_y)],
                theme
                    .marks
                    .get_strokes()
                    .get_connectors()
                    .get_waterfall_connector()
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
                .get_strokes()
                .get_references()
                .get_waterfall_reference()
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
                annotation.anchor,
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
                prepared.total_candidates[0]
                    .0
                    .min(prepared.total_candidates[1].0),
                prepared.total_candidates[0]
                    .0
                    .max(prepared.total_candidates[1].0),
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
    let mut result = PlotRenderResult::new(layout, chart.prepared_discrete_guide()?);
    result.numeric_presentation = Some(selected_totals.numeric.clone());
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

struct Annotation {
    target: crate::layout::AddressRole,
    text_plan: ResolvedTextPlan,
    x: f64,
    anchor: TextAnchor,
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

fn total_shelf_size(
    candidates: &[(f64, Option<crate::Text>); 2],
    theme: &crate::theme::ResolvedTheme,
) -> u32 {
    candidates
        .iter()
        .filter_map(|(_, text)| text.as_ref())
        .map(|text| {
            text.appearance(&theme.typography.annotations.waterfall_total)
                .theme
                .get_font_size()
        })
        .max()
        .unwrap_or(theme.typography.annotations.waterfall_total.get_font_size())
}

fn plan_annotations(
    projection: &crate::layout::Projection,
    panel: PanelBox,
    source_y: f64,
    candidates: &[(f64, Option<crate::Text>); 2],
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
        .get_waterfall_shelf()
        .lengths(total_shelf_size(candidates, theme))?;
    let gap = metrics
        .gap
        .round(crate::layout::kernel::SpatialRounding::Ceil)?
        .quanta();
    let producer = context.producer_text()?;
    for (index, (value, text)) in candidates.iter().enumerate() {
        let Some(text) = text else { continue };
        let target = if index == 0 {
            crate::layout::AddressRole::WaterfallBaseLabel
        } else {
            crate::layout::AddressRole::WaterfallTerminalLabel
        };

        if !projection.contains_data(*value, source_y) {
            continue;
        }
        let appearance = text.appearance(&theme.typography.annotations.waterfall_total);
        let text_plan = producer.plan(text.as_str(), "waterfall total annotation", || {
            Ok(theme.measure_text(
                text.as_str(),
                appearance.theme,
                text.resolved_alignment(crate::TextAlignment::Center).into(),
                "waterfall total annotation",
                appearance.exact_override,
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
                "waterfall: total annotation {:?} requires {:.0}px but only {:.0}px is available; increase chart width or shorten the configured label",
                text.as_str(),
                required_width,
                panel.width()
            )));
            continue;
        }
        let (projected_x, _) = projection.project(*value, source_y);
        let x_quanta = crate::layout::kernel::nearest_spatial_quanta(projected_x)?;
        let half = crate::render::backend::nearest_even_text_half(text_plan.width_quanta)
            .ok_or_else(|| Error::Render("Waterfall text half width overflow".into()))?;
        let (anchor, offset) = match text_plan.style.alignment {
            PhysicalTextAlignment::Start => (TextAnchor::Start, -half),
            PhysicalTextAlignment::Center => (TextAnchor::Middle, 0),
            PhysicalTextAlignment::End => (TextAnchor::End, text_plan.width_quanta - half),
        };
        let x = x_quanta
            .checked_add(offset)
            .ok_or_else(|| Error::Render("Waterfall text anchor overflow".into()))?
            as f64
            / 1024.0;
        let placement =
            PlannedTextPlacement::baseline(Point2D::new(offset as f64 / 1024.0, 0.0), anchor, 0.0);
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
            anchor,
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
