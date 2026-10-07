//! Canvas-owned TextPanel content uses the active quantized sequence producer.

use super::{TextPanel, TextPanelPlan};
use crate::layout::fit::{self, FitOutcome, FitScope};
use crate::layout::kernel::{
    allocate_grid_axis, nearest_spatial_quanta, FitKind, GridRequirement, TrackSpec,
};
use crate::layout::{AddressRole, PanelBox, PhysicalTextAlignment, TextBlock};
use crate::render::backend::{PlannedTextPlacement, Point2D, TextAnchor};
use crate::theme::{ResolvedTheme, TextTheme};
use crate::Error;

#[expect(
    clippy::cast_precision_loss,
    reason = "the assigned cell is bounded by exact u32 root dimensions"
)]
fn pixels(quanta: i64) -> f64 {
    quanta as f64 / 1024.0
}

#[cfg(test)]
mod tests;

fn text_appearance(theme: &ResolvedTheme) -> [&TextTheme; 2] {
    [
        theme.typography.composition().text_panel_title(),
        theme.typography.composition().body(),
    ]
}

#[derive(Clone, Copy)]
pub(super) struct TextPanelIntrinsics {
    title: (f64, f64),
    body: (f64, f64),
    pub(super) minimum: (f64, f64),
    pub(super) preferred_width: f64,
}

impl TextPanelIntrinsics {
    pub(super) fn prepare(source: &TextPanel, theme: &ResolvedTheme) -> Result<Self, Error> {
        let [title_style, body_style] = text_appearance(theme);
        let title = fit::intrinsic_widths(
            theme,
            source.title(),
            title_style,
            "text panel title intrinsics",
        )?;
        let body = fit::intrinsic_widths(
            theme,
            source.body(),
            body_style,
            "text panel body intrinsics",
        )?;
        let lengths = theme.text_panel_lengths()?;
        let resources = theme.prepared_font_environment()?.generation_resources();
        let edges = lengths.inset.times(2, &resources)?;
        let title_ch = theme
            .measure_text(
                "0",
                title_style,
                PhysicalTextAlignment::Start,
                "TextPanel title 66ch",
                false,
            )?
            .width();
        let body_ch = theme
            .measure_text(
                "0",
                body_style,
                PhysicalTextAlignment::Start,
                "TextPanel body 66ch",
                false,
            )?
            .width();
        Ok(Self {
            title,
            body,
            minimum: (
                crate::theme::composition::hard(
                    &edges.add_pixels(title.0.max(body.0), &resources)?,
                    &resources,
                )?,
                crate::theme::composition::hard(
                    &edges
                        .add(&lengths.gap, &resources)?
                        .add(&title_style.line_pitch_exact(&resources)?, &resources)?
                        .add(&body_style.line_pitch_exact(&resources)?, &resources)?,
                    &resources,
                )?,
            ),
            preferred_width: crate::theme::composition::hard(
                &edges.add_pixels(
                    title
                        .0
                        .max(title.1.min(66.0 * title_ch))
                        .max(body.0.max(body.1.min(66.0 * body_ch))),
                    &resources,
                )?,
                &resources,
            )?,
        })
    }
}

pub(super) fn natural_height(
    source: &TextPanel,
    width: f64,
    theme: &ResolvedTheme,
) -> Result<f64, Error> {
    let [title, body] = text_appearance(theme);
    let lengths = theme.text_panel_lengths()?;
    let resources = theme.prepared_font_environment()?.generation_resources();
    let available = crate::theme::composition::capacity(
        width,
        &lengths.inset.times(2, &resources)?,
        &resources,
    )?;
    let title = fit::measured(fit::text(
        theme,
        source.title(),
        title,
        available,
        PhysicalTextAlignment::Start,
        "natural TextPanel title",
        &FitScope::root(),
    ))
    .map_err(crate::layout::fit::PlanError::into_error)?;
    let body = fit::measured(fit::text(
        theme,
        source.body(),
        body,
        available,
        PhysicalTextAlignment::Start,
        "natural TextPanel body",
        &FitScope::root(),
    ))
    .map_err(crate::layout::fit::PlanError::into_error)?;
    crate::theme::composition::hard(
        &lengths
            .inset
            .times(2, &resources)?
            .add(&lengths.gap, &resources)?
            .add_pixels(title.height(), &resources)?
            .add_pixels(body.height(), &resources)?,
        &resources,
    )
}

impl TextPanelPlan {
    #[expect(
        clippy::too_many_lines,
        reason = "the active TextPanel owns intrinsic text, assigned sequence tracks, and frozen placement"
    )]
    pub(super) fn measure(
        source: &TextPanel,
        index: usize,
        cell: PanelBox,
        theme: &ResolvedTheme,
        intrinsics: TextPanelIntrinsics,
        scope: &FitScope,
    ) -> Result<FitOutcome<Self>, Error> {
        let owner = scope.child(
            AddressRole::TextPanelContent,
            index,
            &theme.prepared_font_environment()?.generation_resources(),
        )?;
        let title_scope = owner.child(
            AddressRole::TextPanelTitle,
            0,
            &theme.prepared_font_environment()?.generation_resources(),
        )?;
        let body_scope = owner.child(
            AddressRole::TextPanelBody,
            0,
            &theme.prepared_font_environment()?.generation_resources(),
        )?;
        let [title_style, body_style] = text_appearance(theme);
        let (title_hard, title_preferred) = intrinsics.title;
        let (body_hard, body_preferred) = intrinsics.body;
        let x0 = nearest_spatial_quanta(cell.x0)?;
        let x1 = nearest_spatial_quanta(cell.x1)?;
        let y0 = nearest_spatial_quanta(cell.y0)?;
        let y1 = nearest_spatial_quanta(cell.y1)?;
        let width = x1
            .checked_sub(x0)
            .ok_or_else(|| Error::Render("TextPanel width overflowed".into()))?;
        let height = y1
            .checked_sub(y0)
            .ok_or_else(|| Error::Render("TextPanel height overflowed".into()))?;
        let resources = theme.prepared_font_environment()?.generation_resources();
        let lengths = theme.text_panel_lengths()?;
        let edges = lengths.inset.times(2, &resources)?;
        let content_width = nearest_spatial_quanta(
            crate::theme::composition::capacity(pixels(width), &edges, &resources)?.max(0.0),
        )?;
        let hard = nearest_spatial_quanta(crate::layout::kernel::required_length(
            title_hard.max(body_hard),
        )?)?;
        let column = TrackSpec {
            min: 0,
            preferred: 0,
            max: None,
            flexible: true,
        };
        let widths = [
            GridRequirement {
                start: 0,
                span: 1,
                hard: nearest_spatial_quanta(crate::layout::kernel::required_length(title_hard)?)?,
                preferred: nearest_spatial_quanta(crate::layout::kernel::required_length(
                    title_preferred.max(title_hard),
                )?)?,
                fill: false,
                address: &title_scope.address,
            },
            GridRequirement {
                start: 0,
                span: 1,
                hard: nearest_spatial_quanta(crate::layout::kernel::required_length(body_hard)?)?,
                preferred: nearest_spatial_quanta(crate::layout::kernel::required_length(
                    body_preferred.max(body_hard),
                )?)?,
                fill: false,
                address: &body_scope.address,
            },
        ];
        let assigned = match allocate_grid_axis(
            &[column],
            &widths,
            content_width,
            [0, 0],
            0,
            None,
            true,
            &resources,
            &owner,
        )? {
            FitOutcome::Measured(assigned) => assigned.sizes[0],
            FitOutcome::Miss(_) if content_width < hard => {
                let role = if title_hard >= body_hard {
                    AddressRole::TextPanelTitle
                } else {
                    AddressRole::TextPanelBody
                };
                let child = if role == AddressRole::TextPanelTitle {
                    &title_scope
                } else {
                    &body_scope
                };
                return Ok(FitOutcome::Miss(child.failure(
                    FitKind::Text,
                    "TextPanel hard unbroken text exceeds its assigned content width",
                )));
            }
            FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
        };
        let maximum_width = pixels(assigned);
        let title = match fit::text(
            theme,
            source.title(),
            title_style,
            maximum_width,
            PhysicalTextAlignment::Start,
            "text panel title",
            &title_scope,
        )? {
            FitOutcome::Measured(plan) => TextBlock::from_resolved(plan),
            FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
        };
        let body = match fit::text(
            theme,
            source.body(),
            body_style,
            maximum_width,
            PhysicalTextAlignment::Start,
            "text panel body",
            &body_scope,
        )? {
            FitOutcome::Measured(plan) => TextBlock::from_resolved(plan),
            FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
        };
        let title_height =
            nearest_spatial_quanta(crate::layout::kernel::required_length(title.height())?)?;
        let body_height =
            nearest_spatial_quanta(crate::layout::kernel::required_length(body.height())?)?;
        let rows = [TrackSpec {
            min: 0,
            preferred: 0,
            max: None,
            flexible: false,
        }; 2];
        let row_children = [
            GridRequirement {
                start: 0,
                span: 1,
                hard: title_height,
                preferred: title_height,
                fill: false,
                address: &title_scope.address,
            },
            GridRequirement {
                start: 1,
                span: 1,
                hard: body_height,
                preferred: body_height,
                fill: false,
                address: &body_scope.address,
            },
        ];
        let vertical_reserve = edges.add(&lengths.gap, &resources)?;
        let content_height = nearest_spatial_quanta(
            crate::theme::composition::capacity(pixels(height), &vertical_reserve, &resources)?
                .max(0.0),
        )?;
        let allocation = match allocate_grid_axis(
            &rows,
            &row_children,
            content_height,
            [0, 0],
            0,
            None,
            false,
            &resources,
            &owner,
        )? {
            FitOutcome::Measured(value) => value,
            FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
        };
        let x = lengths.inset.after_pixels(pixels(x0), &resources)?;
        if i128::from(allocation.sizes[0])
            + i128::from(allocation.sizes[1])
            + i128::from(allocation.trailing_slack)
            != i128::from(content_height)
        {
            return Err(Error::Render(
                "TextPanel sequence left inconsistent content slack".into(),
            ));
        }
        let body_offset = lengths
            .inset
            .add_pixels(pixels(allocation.sizes[0]), &resources)?
            .add(&lengths.gap, &resources)?;
        let title_baseline = title.plan().first_baseline_quanta.unwrap_or(0);
        let body_baseline = body.plan().first_baseline_quanta.unwrap_or(0);
        resources
            .claim_bytes(40)
            .map_err(|error| Error::Render(format!("TextPanel layer admission: {error:?}")))?;
        Ok(FitOutcome::Measured(Self {
            layer: format!("canvas-text-panel-{index}"),
            cell,
            title_placement: PlannedTextPlacement::baseline(
                Point2D::new(
                    x,
                    lengths
                        .inset
                        .add_pixels(pixels(title_baseline), &resources)?
                        .before_pixels(pixels(y1), &resources)?,
                ),
                TextAnchor::Start,
                0.0,
            ),
            body_placement: PlannedTextPlacement::baseline(
                Point2D::new(
                    x,
                    body_offset
                        .add_pixels(pixels(body_baseline), &resources)?
                        .before_pixels(pixels(y1), &resources)?,
                ),
                TextAnchor::Start,
                0.0,
            ),
            title,
            body,
        }))
    }
}
