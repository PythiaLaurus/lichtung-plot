//! Canvas-owned TextPanel content uses the active quantized sequence producer.

use super::{TextPanel, TextPanelPlan};
use crate::layout::fit::{self, FitOutcome, FitScope};
use crate::layout::kernel::{
    allocate_grid_axis, nearest_spatial_quanta, FitKind, GridRequirement, TrackSpec,
};
use crate::layout::{AddressRole, PanelBox, PhysicalTextAlignment, TextBlock};
use crate::render::backend::{PlannedTextPlacement, Point2D};
use crate::text::TextAppearance;
use crate::theme::ResolvedTheme;
use crate::{Error, TextAlignment};

#[cfg(test)]
mod tests;

#[expect(
    clippy::cast_precision_loss,
    reason = "the assigned cell is bounded by exact u32 root dimensions"
)]
fn pixels(quanta: i64) -> f64 {
    quanta as f64 / 1024.0
}

impl TextPanel {
    pub(super) fn appearances<'a>(&'a self, theme: &'a ResolvedTheme) -> [TextAppearance<'a>; 2] {
        [
            self.title
                .appearance(theme.typography.get_composition().get_text_panel_title()),
            self.body
                .appearance(theme.typography.get_composition().get_body()),
        ]
    }

    fn alignments(&self) -> [TextAlignment; 2] {
        [&self.title, &self.body].map(|text| text.resolved_alignment(TextAlignment::Start))
    }
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
        let [title_style, body_style] = source.appearances(theme);
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
        let lengths = theme.text_panel_lengths(
            title_style.theme.get_font_size(),
            body_style.theme.get_font_size(),
        )?;

        let edges = lengths.inset.times(2);
        let title_ch = if title_style.hidden() {
            0.0
        } else {
            theme
                .measure_text(
                    "0",
                    title_style.theme,
                    PhysicalTextAlignment::Start,
                    "TextPanel title 66ch",
                    title_style.exact_override,
                )?
                .width()
        };
        let body_ch = if body_style.hidden() {
            0.0
        } else {
            theme
                .measure_text(
                    "0",
                    body_style.theme,
                    PhysicalTextAlignment::Start,
                    "TextPanel body 66ch",
                    body_style.exact_override,
                )?
                .width()
        };
        Ok(Self {
            title,
            body,
            minimum: (
                crate::theme::composition::hard(&edges.add_pixels(title.0.max(body.0))?)?,
                crate::theme::composition::hard(
                    &edges
                        .add_length(&lengths.gap)
                        .add_length(
                            &title_style
                                .theme
                                .line_pitch_exact()?
                                .times(u32::from(!title_style.hidden())),
                        )
                        .add_length(
                            &body_style
                                .theme
                                .line_pitch_exact()?
                                .times(u32::from(!body_style.hidden())),
                        ),
                )?,
            ),
            preferred_width: crate::theme::composition::hard(
                &edges.add_pixels(
                    title
                        .0
                        .max(title.1.min(66.0 * title_ch))
                        .max(body.0.max(body.1.min(66.0 * body_ch))),
                )?,
            )?,
        })
    }
}

pub(super) fn natural_height(
    source: &TextPanel,
    width: f64,
    theme: &ResolvedTheme,
) -> Result<f64, Error> {
    let [title, body] = source.appearances(theme);
    let [title_alignment, body_alignment] = source.alignments();
    let lengths =
        theme.text_panel_lengths(title.theme.get_font_size(), body.theme.get_font_size())?;

    let available = crate::theme::composition::capacity(width, &lengths.inset.times(2))?;
    let title = fit::measured(fit::single_text(
        theme,
        source.title(),
        title,
        available,
        title_alignment.into(),
        "natural TextPanel title",
        &FitScope::root(),
    ))
    .map_err(crate::layout::fit::PlanError::into_error)?;
    let body = fit::measured(fit::single_text(
        theme,
        source.body(),
        body,
        available,
        body_alignment.into(),
        "natural TextPanel body",
        &FitScope::root(),
    ))
    .map_err(crate::layout::fit::PlanError::into_error)?;
    crate::theme::composition::hard(
        &lengths
            .inset
            .times(2)
            .add_length(&lengths.gap)
            .add_pixels(title.occupied_size().1)?
            .add_pixels(body.occupied_size().1)?,
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
        let owner = scope.child(AddressRole::TextPanelContent, index)?;
        let title_scope = owner.child(AddressRole::TextPanelTitle, 0)?;
        let body_scope = owner.child(AddressRole::TextPanelBody, 0)?;
        let [title_style, body_style] = source.appearances(theme);
        let [title_alignment, body_alignment] = source.alignments();
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

        let lengths = theme.text_panel_lengths(
            title_style.theme.get_font_size(),
            body_style.theme.get_font_size(),
        )?;
        let edges = lengths.inset.times(2);
        let content_width = nearest_spatial_quanta(
            crate::theme::composition::capacity(pixels(width), &edges)?.max(0.0),
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
        let title = match fit::single_text(
            theme,
            source.title(),
            title_style,
            maximum_width,
            title_alignment.into(),
            "text panel title",
            &title_scope,
        )? {
            FitOutcome::Measured(plan) => TextBlock::from_resolved(plan),
            FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
        };
        let body = match fit::single_text(
            theme,
            source.body(),
            body_style,
            maximum_width,
            body_alignment.into(),
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
        let vertical_reserve = edges.add_length(&lengths.gap);
        let content_height = nearest_spatial_quanta(
            crate::theme::composition::capacity(pixels(height), &vertical_reserve)?.max(0.0),
        )?;
        let allocation = match allocate_grid_axis(
            &rows,
            &row_children,
            content_height,
            [0, 0],
            0,
            None,
            false,
            &owner,
        )? {
            FitOutcome::Measured(value) => value,
            FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
        };
        let x = lengths.inset.after_pixels(pixels(x0))?;
        let right = lengths.inset.before_pixels(pixels(x1))?;
        let (title_x, title_anchor) = title_alignment.resolve_physical(x, right);
        let (body_x, body_anchor) = body_alignment.resolve_physical(x, right);
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
            .add_pixels(pixels(allocation.sizes[0]))?
            .add_length(&lengths.gap);
        let title_baseline = title.plan().first_baseline_quanta.unwrap_or(0);
        let body_baseline = body.plan().first_baseline_quanta.unwrap_or(0);

        let title_top = lengths.inset.before_pixels(pixels(y1))?;
        let body_top = body_offset.before_pixels(pixels(y1))?;
        let bottom = lengths.inset.after_pixels(pixels(y0))?;
        let title_placement = crate::layout::text::placement::resolve(
            PlannedTextPlacement::baseline(
                Point2D::new(title_x, title_top - pixels(title_baseline)),
                title_anchor,
                0.0,
            ),
            title.plan(),
            Some(PanelBox::new(
                x,
                right,
                title_top - title.height(),
                title_top,
            )),
        )?;
        let body_placement = crate::layout::text::placement::resolve(
            PlannedTextPlacement::baseline(
                Point2D::new(body_x, body_top - pixels(body_baseline)),
                body_anchor,
                0.0,
            ),
            body.plan(),
            Some(PanelBox::new(x, right, bottom, body_top)),
        )?;
        let title = match crate::layout::fit::outcome(crate::layout::text::placement::contain(
            title.plan().clone(),
            title_placement,
            PanelBox::new(x, right, title_top - title.height(), title_top),
            &title_scope,
        ))? {
            FitOutcome::Measured(plan) => TextBlock::from_resolved(plan),
            FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
        };
        let body = match crate::layout::fit::outcome(crate::layout::text::placement::contain(
            body.plan().clone(),
            body_placement,
            PanelBox::new(x, right, bottom, body_top),
            &body_scope,
        ))? {
            FitOutcome::Measured(plan) => TextBlock::from_resolved(plan),
            FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
        };

        Ok(FitOutcome::Measured(Self {
            layer: format!("canvas-text-panel-{index}"),
            cell,
            title_placement,
            body_placement,
            title,
            body,
        }))
    }
}
