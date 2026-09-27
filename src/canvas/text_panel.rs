//! Canvas-owned TextPanel content uses the active quantized sequence producer.

use super::{TextPanel, TextPanelPlan};
use crate::layout::fit::{self, FitOutcome, FitScope};
use crate::layout::kernel::{allocate_tracks, nearest_spatial_quanta, FitKind, TrackSpec};
use crate::layout::{AddressRole, PanelBox, PhysicalTextAlignment, TextBlock};
use crate::render::backend::{PlannedTextPlacement, Point2D, TextAnchor};
use crate::theme::{ResolvedTheme, TextAppearance};
use crate::Error;

#[expect(
    clippy::cast_precision_loss,
    reason = "the assigned cell is bounded by exact u32 root dimensions"
)]
fn pixels(quanta: i64) -> f64 {
    quanta as f64 / 1024.0
}

fn text_appearance(theme: &ResolvedTheme) -> [TextAppearance; 2] {
    [
        theme.legacy_text_appearance(theme.composition_title_font_size, crate::TextWeight::Normal),
        theme.legacy_text_appearance(theme.label_font_size, crate::TextWeight::Normal),
    ]
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
        scope: &FitScope,
    ) -> Result<FitOutcome<Self>, Error> {
        let owner = scope.child(AddressRole::TextPanelContent, index)?;
        let [title_style, body_style] = text_appearance(theme);
        let (title_hard, title_preferred) = fit::intrinsic_widths(
            theme,
            source.title(),
            &title_style,
            "text panel title intrinsics",
        )?;
        let (body_hard, body_preferred) = fit::intrinsic_widths(
            theme,
            source.body(),
            &body_style,
            "text panel body intrinsics",
        )?;
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
        let hard = title_hard.max(body_hard);
        let width_spec = TrackSpec {
            min: nearest_spatial_quanta(crate::layout::kernel::required_length(hard)?)?,
            preferred: nearest_spatial_quanta(crate::layout::kernel::required_length(
                title_preferred.max(body_preferred).max(hard),
            )?)?,
            max: None,
            flexible: true,
        };
        let assigned = match allocate_tracks(
            &[width_spec],
            width,
            [inset, inset],
            0,
            true,
            &resources,
            &owner,
        )? {
            FitOutcome::Measured(assigned) => assigned.sizes[0],
            FitOutcome::Miss(_)
                if i128::from(width) - 2 * i128::from(inset) < i128::from(width_spec.min) =>
            {
                let role = if title_hard >= body_hard {
                    AddressRole::TextPanelTitle
                } else {
                    AddressRole::TextPanelBody
                };
                return Ok(FitOutcome::Miss(owner.child(role, 0)?.failure(
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
            &title_style,
            maximum_width,
            PhysicalTextAlignment::Start,
            "text panel title",
            &owner.child(AddressRole::TextPanelTitle, 0)?,
        )? {
            FitOutcome::Measured(plan) => TextBlock::from_resolved(plan),
            FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
        };
        let body = match fit::text(
            theme,
            source.body(),
            &body_style,
            maximum_width,
            PhysicalTextAlignment::Start,
            "text panel body",
            &owner.child(AddressRole::TextPanelBody, 0)?,
        )? {
            FitOutcome::Measured(plan) => TextBlock::from_resolved(plan),
            FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
        };
        let title_height =
            nearest_spatial_quanta(crate::layout::kernel::required_length(title.height())?)?;
        let body_height =
            nearest_spatial_quanta(crate::layout::kernel::required_length(body.height())?)?;
        let rows = [title_height, body_height].map(|size| TrackSpec {
            min: size,
            preferred: size,
            max: None,
            flexible: false,
        });
        let allocation = match allocate_tracks(
            &rows,
            height,
            [inset, inset],
            gap,
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
