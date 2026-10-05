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

fn text_appearance(theme: &ResolvedTheme) -> [&TextTheme; 2] {
    [&theme.typography.text_panel_title, &theme.typography.body]
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
        let inset = theme.composition.text_panel_inset;
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
                title.0.max(body.0) + 2.0 * inset,
                title_style.line_pitch_pixels()
                    + body_style.line_pitch_pixels()
                    + theme.composition.text_panel_gap
                    + 2.0 * inset,
            ),
            preferred_width: title
                .0
                .max(title.1.min(66.0 * title_ch))
                .max(body.0.max(body.1.min(66.0 * body_ch)))
                + 2.0 * inset,
        })
    }
}

pub(super) fn natural_height(
    source: &TextPanel,
    width: f64,
    theme: &ResolvedTheme,
) -> Result<f64, Error> {
    let [title, body] = text_appearance(theme);
    let available = width - 2.0 * theme.composition.text_panel_inset;
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
    Ok(title.height()
        + body.height()
        + theme.composition.text_panel_gap
        + 2.0 * theme.composition.text_panel_inset)
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
        let inset = nearest_spatial_quanta(crate::layout::kernel::required_length(
            theme.composition.text_panel_inset,
        )?)?;
        let gap = nearest_spatial_quanta(crate::layout::kernel::required_length(
            theme.composition.text_panel_gap,
        )?)?;
        let width = x1
            .checked_sub(x0)
            .ok_or_else(|| Error::Render("TextPanel width overflowed".into()))?;
        let height = y1
            .checked_sub(y0)
            .ok_or_else(|| Error::Render("TextPanel height overflowed".into()))?;
        let resources = theme.prepared_font_environment()?.generation_resources();
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
            width,
            [inset, inset],
            0,
            None,
            true,
            &resources,
            &owner,
        )? {
            FitOutcome::Measured(assigned) => assigned.sizes[0],
            FitOutcome::Miss(_) if i128::from(width) - 2 * i128::from(inset) < i128::from(hard) => {
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
        let allocation = match allocate_grid_axis(
            &rows,
            &row_children,
            height,
            [inset, inset],
            gap,
            None,
            false,
            &resources,
            &owner,
        )? {
            FitOutcome::Measured(value) => value,
            FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
        };
        let x = pixels(x0 + inset);
        let title_top = y1 - inset;
        let body_top = title_top - allocation.sizes[0] - gap;
        let trailing_start = y0 + inset + allocation.trailing_slack;
        if body_top - allocation.sizes[1] != trailing_start {
            return Err(Error::Render(
                "TextPanel sequence left an inconsistent trailing slack".into(),
            ));
        }
        let title_baseline = title.plan().first_baseline_quanta.unwrap_or(0);
        let body_baseline = body.plan().first_baseline_quanta.unwrap_or(0);
        resources
            .claim_bytes(40)
            .map_err(|error| Error::Render(format!("TextPanel layer admission: {error:?}")))?;
        Ok(FitOutcome::Measured(Self {
            layer: format!("canvas-text-panel-{index}"),
            cell,
            title_placement: PlannedTextPlacement::baseline(
                Point2D::new(x, pixels(title_top - title_baseline)),
                TextAnchor::Start,
                0.0,
            ),
            body_placement: PlannedTextPlacement::baseline(
                Point2D::new(x, pixels(body_top - body_baseline)),
                TextAnchor::Start,
                0.0,
            ),
            title,
            body,
        }))
    }
}
