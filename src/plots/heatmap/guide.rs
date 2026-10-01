//! Mandatory Heatmap guide measured by the common continuous-guide owner.

use super::{CellPayload, ColorbarPayload, HeatmapData};
use crate::decorations::legend::{LegendCall, LegendMetrics, MeasuredContinuousGuide};
use crate::layout::fit::{self, FitKind, FitScope, PlanError};
use crate::layout::kernel::{allocate_grid_axis, GridRequirement, TrackSpec};
use crate::layout::{PanelBox, PhysicalTextAlignment};
use crate::theme::ResolvedTheme;
use crate::Error;

pub(super) fn measure(
    marks: &HeatmapData,
    panel: PanelBox,
    minimum: f64,
    theme: &ResolvedTheme,
    scope: &FitScope,
    data: crate::data::PlotData<'_>,
) -> Result<(PanelBox, MeasuredContinuousGuide), PlanError> {
    let semantics = marks.guide_semantics(theme, data)?;
    let indices: &[usize] = if semantics.domain.0.to_bits() == semantics.domain.1.to_bits() {
        &[0]
    } else if semantics
        .center
        .is_some_and(|center| center > semantics.domain.0 && center < semantics.domain.1)
    {
        &[0, 2, 1]
    } else {
        &[0, 1]
    };
    let descriptor = crate::decorations::legend::descriptor::ContinuousDescriptor {
        field: data.copy_source_string(&marks.color_label)?,
        title: data.copy_source_string(&marks.color_label)?,
        stops: data.collect_source(marks.colorbar_stops.iter().copied().map(Ok))?,
        center: semantics.center,
        missing: semantics.missing,
        domain: semantics.domain,
        exact: true,
        labels: data.collect_source(indices.iter().map(|&index| {
            data.claim_source_visits(1)?;
            data.copy_source_string(&marks.colorbar_values[index])
        }))?,
    };
    let metrics = LegendMetrics::from_theme(theme);
    let (hard_title, natural_title) = fit::intrinsic_widths(
        theme,
        &descriptor.title,
        &theme.typography.legend_title,
        "continuous guide title",
    )?;
    let mut hard = hard_title;
    for label in descriptor
        .labels
        .iter()
        .map(String::as_str)
        .chain(descriptor.missing.iter().map(|key| key.label.as_str()))
    {
        data.claim_source_visits(1)?;
        let plan = theme.measure_text(
            label,
            &theme.typography.legend_label,
            PhysicalTextAlignment::Start,
            "continuous guide value",
            false,
        )?;
        hard =
            hard.max(metrics.handle_w + metrics.handle_text + plan.width().max(plan.fit_width()));
    }
    let available = panel.width() - minimum - metrics.outside_gap;
    let width = crate::layout::kernel::required_length(
        natural_title.max(hard) + 2.0 * metrics.pad + 1.0 / 1024.0,
    )?
    .min(available);
    if width < hard + 2.0 * metrics.pad {
        return Err(scope
            .failure(
                FitKind::LegendOrGuide,
                "Heatmap guide cannot preserve its text and the hard data tracks",
            )
            .into());
    }
    let guide = fit::measured(MeasuredContinuousGuide::measure(
        &descriptor,
        (width, panel.height()),
        false,
        false,
        theme,
        &mut false,
        scope,
    ))?;
    let hard = crate::layout::kernel::ceil_spatial_quanta(minimum)?;
    let guide_width = crate::layout::kernel::ceil_spatial_quanta(guide.width)?;
    let tracks = [
        TrackSpec {
            min: hard,
            preferred: hard,
            max: None,
            flexible: true,
        },
        TrackSpec {
            min: guide_width,
            preferred: guide_width,
            max: Some(guide_width),
            flexible: false,
        },
    ];
    let children: [GridRequirement<'_>; 0] = [];
    let assigned = fit::measured(allocate_grid_axis(
        &tracks,
        &children,
        crate::layout::kernel::nearest_spatial_quanta(panel.width())?,
        [0, 0],
        crate::layout::kernel::ceil_spatial_quanta(metrics.outside_gap)?,
        None,
        true,
        data.source_resources()?,
        scope,
    ))?;
    let body = PanelBox::new(
        panel.x0,
        panel.x0 + super::track_pixels(assigned.sizes[0]),
        panel.y0,
        panel.y1,
    );
    Ok((body, guide))
}

pub(super) fn select(
    guide: &MeasuredContinuousGuide,
    body: PanelBox,
    theme: &ResolvedTheme,
    data: crate::data::PlotData<'_>,
) -> Result<ColorbarPayload, Error> {
    let plan = guide.select(
        body.x1 + theme.legend.outside_gap,
        body.y0 + (body.height() - guide.height) / 2.0,
        data,
    )?;
    let mut gradient = None;
    let mut text = Vec::new();
    data.reserve_source(&mut text, plan.calls.len())?;
    let mut missing: Option<CellPayload> = None;
    for call in plan.calls {
        data.claim_source_visits(1)?;
        match call {
            LegendCall::Gradient(bounds, start, end, stops) => {
                gradient = Some((bounds, start, end, stops));
            }
            LegendCall::Text(placed) => {
                if let Some(cell) = &mut missing {
                    cell.text = Some(placed);
                } else {
                    text.push(placed);
                }
            }
            LegendCall::Fill(corners, _) => {
                missing = Some(CellPayload {
                    corners,
                    fill: theme
                        .marks
                        .heatmap_missing_key_outline
                        .backend_style(theme.marks.heatmap_missing_fill),
                    text: None,
                    cross: None,
                });
            }
            LegendCall::Background(_, _) => {}
            _ => {
                return Err(Error::Render(
                    "unexpected Heatmap continuous guide operand".into(),
                ))
            }
        }
    }
    // Keep the title last in the retained producer sequence.
    if !text.is_empty() {
        text.rotate_left(1);
    }
    let (bounds, start, end, stops) =
        gradient.ok_or_else(|| Error::Render("Heatmap guide has no gradient".into()))?;
    Ok(ColorbarPayload {
        bounds,
        start,
        end,
        stops,
        text,
        missing,
    })
}
