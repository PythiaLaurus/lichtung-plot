//! Assigned-width root text, frozen before scene capture and Null validation.

use crate::layout::fit::{self, FitScope, PlanError};
use crate::layout::{AddressRole, PanelBox, PhysicalTextAlignment, ResolvedTextPlan};
use crate::render::backend::{Backend, PlannedTextPlacement, Point2D};
use crate::theme::{ResolvedTheme, TitleAlignment};
use crate::Error;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RootAlignments {
    pub(crate) title: TitleAlignment,
    pub(crate) subtitle: TitleAlignment,
    pub(crate) caption: TitleAlignment,
}

impl RootAlignments {
    pub(crate) fn resolve(
        theme: &ResolvedTheme,
        plot: [Option<TitleAlignment>; 2],
        panel: [Option<TitleAlignment>; 2],
    ) -> Self {
        let title = plot[0]
            .or(panel[0])
            .unwrap_or(theme.composition.root_title_alignment);
        Self {
            title,
            subtitle: plot[1]
                .or(panel[1])
                .or(theme.composition.root_subtitle_alignment)
                .unwrap_or(title),
            caption: theme.composition.root_caption_alignment,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct RootTextRole {
    text: ResolvedTextPlan,
    bounds: PanelBox,
    placement: PlannedTextPlacement,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RootTitleBlockPlan {
    alignments: RootAlignments,
    roles: [Option<RootTextRole>; 3],
    pub(crate) top_reserve: f64,
    pub(crate) bottom_reserve: f64,
}

/// Alignment- and allocation-independent role requirements, retained by the root.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RootTitleBlockIntrinsics {
    roles: [(f64, f64); 3],
}

impl RootTitleBlockIntrinsics {
    pub(crate) fn widths(self) -> (f64, f64) {
        self.roles
            .into_iter()
            .fold((0.0_f64, 0.0_f64), |a, b| (a.0.max(b.0), a.1.max(b.1)))
    }

    pub(crate) fn check_width(
        self,
        width: f64,
        scope: &FitScope,
        resources: &crate::font::canonical::OperationGenerationResources,
    ) -> Result<(), PlanError> {
        if width >= self.widths().0 {
            return Ok(());
        }
        for ((hard, _), role) in self.roles.into_iter().zip([
            AddressRole::RootTitle,
            AddressRole::RootSubtitle,
            AddressRole::RootCaption,
        ]) {
            if hard > 0.0 && hard > width {
                return Err(scope
                    .child(role, 0, resources)?
                    .failure(
                        fit::FitKind::Text,
                        "root title block is narrower than its hard text intrinsic",
                    )
                    .into());
            }
        }
        Ok(())
    }
}

impl RootTitleBlockPlan {
    #[cfg(test)]
    pub(crate) fn placed_text(
        &self,
    ) -> impl Iterator<Item = (PlannedTextPlacement, &ResolvedTextPlan)> {
        self.roles
            .iter()
            .flatten()
            .map(|role| (role.placement, &role.text))
    }

    pub(crate) fn draw_retained(&self, backend: &mut dyn Backend) -> Result<(), Error> {
        for role in self.roles.iter().flatten() {
            backend.draw_planned_text(role.placement, &role.text)?;
        }
        Ok(())
    }

    pub(crate) fn geometry(
        &self,
        geometry: &mut crate::render::geometry::GeometrySnapshot,
    ) -> Result<(), Error> {
        for role in self.roles.iter().flatten() {
            geometry.text(role.placement, &role.text, None)?;
        }
        Ok(())
    }

    /// Intrinsics do not depend on alignment or on an allocated height.
    pub(crate) fn intrinsic_widths(
        theme: &ResolvedTheme,
        sources: [Option<&str>; 3],
        scope: Option<&str>,
    ) -> Result<RootTitleBlockIntrinsics, Error> {
        #[cfg(not(test))]
        let _ = scope;
        theme
            .prepared_font_environment()?
            .claim_measurement_call()
            .map_err(|error| {
                Error::Render(format!("root title intrinsic call admission: {error:?}"))
            })?;
        #[cfg(test)]
        crate::orchestrator::measurements::intrinsic(
            crate::orchestrator::measurements::Counter::RootTitleIntrinsic,
            scope,
        );
        let mut roles = [(0.0_f64, 0.0_f64); 3];
        for ((source, appearance), widths) in sources
            .into_iter()
            .zip([
                &theme.typography.title,
                &theme.typography.subtitle,
                &theme.typography.caption,
            ])
            .zip(&mut roles)
        {
            if let Some(source) = source.filter(|source| !source.is_empty()) {
                *widths = fit::intrinsic_widths(theme, source, appearance, "root title block")?;
            }
        }
        Ok(RootTitleBlockIntrinsics { roles })
    }

    #[expect(
        clippy::cast_precision_loss,
        reason = "one assigned-width pass freezes bounded text and adjacent role geometry"
    )]
    pub(crate) fn measure(
        theme: &ResolvedTheme,
        sources: [Option<&str>; 3],
        span: PanelBox,
        alignments: RootAlignments,
        minimum_body_height: f64,
        scope: &FitScope,
    ) -> Result<Self, PlanError> {
        theme
            .prepared_font_environment()?
            .claim_measurement_call()
            .map_err(|error| {
                Error::Render(format!("root title measurement admission: {error:?}"))
            })?;
        if [span.x0, span.x1, span.y0, span.y1, minimum_body_height]
            .into_iter()
            .any(|value| !value.is_finite())
            || minimum_body_height < 0.0
        {
            return Err(
                Error::Render("root title block has invalid assigned geometry".into()).into(),
            );
        }
        if span.width() < 0.0 || span.height() < 0.0 {
            return Err(scope
                .failure(
                    fit::FitKind::Constraint,
                    "root title block has a negative assigned extent",
                )
                .into());
        }
        let (plans, top_reserve, bottom_reserve) =
            Self::measure_content(theme, sources, span.width(), alignments, scope)?;
        let spacing = &theme.composition.root_title_block;
        let gap = if plans[0].is_some() && plans[1].is_some() {
            spacing.title_subtitle_gap
        } else {
            0.0
        };
        if top_reserve + bottom_reserve + minimum_body_height > span.height() {
            return Err(scope
                .failure(
                    fit::FitKind::TrackOrResidual,
                    "root title block exceeds its exact allocated height",
                )
                .into());
        }
        let mut cursor = span.y1;
        let mut roles = [None, None, None];
        for (index, plan) in plans.into_iter().enumerate() {
            if let Some(text) = plan {
                let top = if index == 2 {
                    span.y0 + text.height()
                } else {
                    cursor
                };
                let alignment = [alignments.title, alignments.subtitle, alignments.caption][index];
                let (mut x, anchor) = alignment.resolve_physical(span.x0, span.x1);
                let left = match alignment {
                    TitleAlignment::Start => span.x0,
                    TitleAlignment::Center => {
                        span.x0 + ((span.width() - text.width()) * 512.0).round_ties_even() / 1024.0
                    }
                    TitleAlignment::End => span.x1 - text.width(),
                };
                if alignment == TitleAlignment::Center {
                    x = left
                        + crate::render::backend::nearest_even_text_half(text.width_quanta)
                            .ok_or_else(|| Error::Render("root text anchor overflowed".into()))?
                            as f64
                            / 1024.0;
                }
                let bounds = PanelBox::new(left, left + text.width(), top - text.height(), top);
                let first = text
                    .first_baseline_quanta
                    .ok_or_else(|| Error::Render("present root text has no baseline".into()))?;
                let placement = PlannedTextPlacement::baseline(
                    Point2D::new(x, top - first as f64 / 1024.0),
                    anchor,
                    0.0,
                );
                cursor = bounds.y0 - if index == 0 { gap } else { 0.0 };
                roles[index] = Some(RootTextRole {
                    text,
                    bounds,
                    placement,
                });
            }
        }
        Ok(Self {
            alignments,
            roles,
            top_reserve,
            bottom_reserve,
        })
    }

    /// Width-dependent text and owned gaps, before the parent assigns height.
    pub(crate) fn content_reserves(
        theme: &ResolvedTheme,
        sources: [Option<&str>; 3],
        width: f64,
        alignments: RootAlignments,
        scope: &FitScope,
    ) -> Result<(f64, f64), PlanError> {
        let (_, top, bottom) = Self::measure_content(theme, sources, width, alignments, scope)?;
        Ok((top, bottom))
    }

    fn measure_content(
        theme: &ResolvedTheme,
        sources: [Option<&str>; 3],
        width: f64,
        alignments: RootAlignments,
        scope: &FitScope,
    ) -> Result<(Vec<Option<ResolvedTextPlan>>, f64, f64), PlanError> {
        let appearances = [
            &theme.typography.title,
            &theme.typography.subtitle,
            &theme.typography.caption,
        ];
        let align = [alignments.title, alignments.subtitle, alignments.caption];
        let addresses = [
            AddressRole::RootTitle,
            AddressRole::RootSubtitle,
            AddressRole::RootCaption,
        ];
        let plans = fit::collect(
            theme,
            "root title block",
            (0..3).map(|index| {
                sources[index]
                    .filter(|source| !source.is_empty())
                    .map(|source| {
                        fit::measured(fit::text(
                            theme,
                            source,
                            appearances[index],
                            width,
                            match align[index] {
                                TitleAlignment::Start => PhysicalTextAlignment::Start,
                                TitleAlignment::Center => PhysicalTextAlignment::Center,
                                TitleAlignment::End => PhysicalTextAlignment::End,
                            },
                            "root title block",
                            &scope.child(
                                addresses[index],
                                0,
                                &theme.prepared_font_environment()?.generation_resources(),
                            )?,
                        ))
                    })
                    .transpose()
            }),
        )?;
        let spacing = &theme.composition.root_title_block;
        let gap = if plans[0].is_some() && plans[1].is_some() {
            spacing.title_subtitle_gap
        } else {
            0.0
        };
        let top_reserve = plans[0].as_ref().map_or(0.0, ResolvedTextPlan::height)
            + plans[1].as_ref().map_or(0.0, ResolvedTextPlan::height)
            + gap
            + if plans[0].is_some() || plans[1].is_some() {
                spacing.title_block_to_content_clearance
            } else {
                0.0
            };
        let bottom_reserve = plans[2].as_ref().map_or(0.0, |plan| {
            plan.height() + spacing.caption_to_content_clearance
        });
        Ok((plans, top_reserve, bottom_reserve))
    }

    pub(crate) fn draw(
        &self,
        sources: [Option<&str>; 3],
        backend: &mut dyn Backend,
    ) -> Result<(), Error> {
        for (role, source) in self.roles.iter().zip(sources) {
            match (role, source.filter(|source| !source.is_empty())) {
                (None, None) => {}
                (Some(role), Some(source)) if role.text.source == source => {
                    // Baselines live in the retained lines, never in a parallel vector.
                    backend.draw_planned_text(role.placement, &role.text)?;
                }
                _ => {
                    return Err(Error::Render(
                        "planned root title block does not match its configured source".into(),
                    ))
                }
            }
        }
        Ok(())
    }

    pub(crate) fn observe(
        &self,
        round: &mut crate::layout::kernel::FeedbackRound,
    ) -> Result<(), Error> {
        for alignment in [
            self.alignments.title,
            self.alignments.subtitle,
            self.alignments.caption,
        ] {
            round.observe_count(alignment as usize)?;
        }
        round.observe_number(self.top_reserve)?;
        round.observe_number(self.bottom_reserve)?;
        for role in &self.roles {
            round.observe_count(usize::from(role.is_some()))?;
            if let Some(role) = role {
                round.observe_text(&role.text)?;
                round.observe_box(role.bounds)?;
                round.observe_number(role.placement.position.x)?;
                round.observe_number(role.placement.position.y)?;
            }
        }
        Ok(())
    }
}
