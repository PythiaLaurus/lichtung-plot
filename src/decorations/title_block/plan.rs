//! Assigned-width root text, frozen before scene capture and Null validation.

use crate::font::canonical::OperationGenerationResources;
use crate::layout::fit::{self, FitScope, PlanError};
use crate::layout::{AddressRole, PanelBox, PhysicalTextAlignment, ResolvedTextPlan};
use crate::render::backend::{Backend, PlannedTextPlacement, Point2D};
use crate::theme::composition::RootPaddingLengths;
use crate::theme::spacing::ExactLength;
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
            .unwrap_or(theme.composition.root_title_alignment());
        Self {
            title,
            subtitle: plot[1]
                .or(panel[1])
                .or(theme.composition.root_subtitle_alignment())
                .unwrap_or(title),
            caption: theme.composition.root_caption_alignment(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct RootTextRole {
    text: ResolvedTextPlan,
    bounds: PanelBox,
    placement: PlannedTextPlacement,
}

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub(crate) struct RootTitleBlockPlan {
    alignments: RootAlignments,
    roles: [Option<RootTextRole>; 3],
    pub(crate) reserves: std::sync::Arc<RootTextReserves>,
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub(crate) struct RootTextReserves {
    pub(crate) top: ExactLength,
    pub(crate) bottom: ExactLength,
    total: ExactLength,
}

impl RootTextReserves {
    pub(crate) fn height_with_body(
        &self,
        body: f64,
        resources: &OperationGenerationResources,
    ) -> Result<ExactLength, Error> {
        self.total.add_pixels(body, resources)
    }
    pub(crate) fn body(
        &self,
        outer: PanelBox,
        padding: &RootPaddingLengths,
        resources: &OperationGenerationResources,
    ) -> Result<PanelBox, Error> {
        Ok(PanelBox::new(
            padding.left.after_pixels(outer.x0, resources)?,
            padding.right.before_pixels(outer.x1, resources)?,
            self.bottom
                .add(&padding.bottom, resources)?
                .after_pixels(outer.y0, resources)?,
            self.top
                .add(&padding.top, resources)?
                .before_pixels(outer.y1, resources)?,
        ))
    }
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
    pub(crate) fn retained_handle_count(&self) -> usize {
        1 + self
            .roles
            .iter()
            .flatten()
            .map(|role| role.text.retained_handle_count())
            .sum::<usize>()
    }
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
                &theme.typography.root.title,
                &theme.typography.root.subtitle,
                &theme.typography.root.caption,
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
        clippy::too_many_lines,
        clippy::too_many_arguments,
        reason = "one assigned-width pass freezes bounded text and adjacent role geometry"
    )]
    pub(crate) fn measure(
        theme: &ResolvedTheme,
        sources: [Option<&str>; 3],
        span: PanelBox,
        padding: &RootPaddingLengths,
        maximum_width: f64,
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
        if [
            span.x0,
            span.x1,
            span.y0,
            span.y1,
            maximum_width,
            minimum_body_height,
        ]
        .into_iter()
        .any(|value| !value.is_finite())
            || minimum_body_height < 0.0
        {
            return Err(
                Error::Render("root title block has invalid assigned geometry".into()).into(),
            );
        }
        if span.width() < 0.0 || span.height() < 0.0 || maximum_width < 0.0 {
            return Err(scope
                .failure(
                    fit::FitKind::Constraint,
                    "root title block has a negative assigned extent",
                )
                .into());
        }
        let resources = theme.prepared_font_environment()?.generation_resources();
        let text_span = padding.inset(span, &resources)?;
        let (plans, reserves) = Self::measure_content(
            theme,
            sources,
            maximum_width.min(text_span.width()),
            alignments,
            scope,
        )?;
        let spacing = theme.root_title_lengths()?;
        let adjacent = plans[0].is_some() && plans[1].is_some();
        if crate::theme::composition::hard(
            &reserves
                .height_with_body(minimum_body_height, &resources)?
                .add(&padding.vertical(&resources)?, &resources)?,
            &resources,
        )? > span.height()
        {
            return Err(scope
                .failure(
                    fit::FitKind::TrackOrResidual,
                    "root title block exceeds its exact allocated height",
                )
                .into());
        }
        let mut consumed = crate::theme::spacing::ExactLength::quanta(
            crate::layout::kernel::LayoutScalar::from_quanta(0),
            &resources,
        )?;
        let mut roles = [None, None, None];
        for (index, plan) in plans.into_iter().enumerate() {
            if let Some(text) = plan {
                let top = if index == 2 {
                    crate::theme::spacing::ExactLength::quanta(
                        crate::layout::kernel::LayoutScalar::from_quanta(text.height_quanta),
                        &resources,
                    )?
                    .add(&padding.bottom, &resources)?
                    .after_pixels(span.y0, &resources)?
                } else {
                    consumed
                        .add(&padding.top, &resources)?
                        .before_pixels(span.y1, &resources)?
                };
                let alignment = [alignments.title, alignments.subtitle, alignments.caption][index];
                let (mut x, anchor) = alignment.resolve_physical(text_span.x0, text_span.x1);
                let left = match alignment {
                    TitleAlignment::Start => text_span.x0,
                    TitleAlignment::Center => {
                        (text_span.x0 * 1024.0 + (text_span.width() - text.width()) * 512.0)
                            .round_ties_even()
                            / 1024.0
                    }
                    TitleAlignment::End => text_span.x1 - text.width(),
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
                    Point2D::new(
                        x,
                        if index == 2 {
                            crate::theme::spacing::ExactLength::quanta(
                                crate::layout::kernel::LayoutScalar::from_quanta(
                                    text.height_quanta,
                                ),
                                &resources,
                            )?
                            .subtract(
                                &crate::theme::spacing::ExactLength::quanta(
                                    crate::layout::kernel::LayoutScalar::from_quanta(first),
                                    &resources,
                                )?,
                                &resources,
                            )?
                            .ok_or_else(|| {
                                Error::Render("root caption baseline exceeds its extent".into())
                            })?
                            .add(&padding.bottom, &resources)?
                            .after_pixels(span.y0, &resources)?
                        } else {
                            consumed
                                .add_quanta(
                                    crate::layout::kernel::LayoutScalar::from_quanta(first),
                                    &resources,
                                )?
                                .add(&padding.top, &resources)?
                                .before_pixels(span.y1, &resources)?
                        },
                    ),
                    anchor,
                    0.0,
                );
                if index < 2 {
                    consumed = consumed.add_quanta(
                        crate::layout::kernel::LayoutScalar::from_quanta(text.height_quanta),
                        &resources,
                    )?;
                    if index == 0 && adjacent {
                        consumed = consumed.add(&spacing.title_subtitle_gap, &resources)?;
                    }
                }
                roles[index] = Some(RootTextRole {
                    text,
                    bounds,
                    placement,
                });
            }
        }
        resources
            .claim_slots(1)
            .map_err(|error| Error::Render(format!("root exact reserve retention: {error:?}")))?;
        resources
            .claim_bytes((size_of::<RootTextReserves>() + 2 * size_of::<usize>()) as u64)
            .map_err(|error| Error::Render(format!("root exact reserve storage: {error:?}")))?;
        Ok(Self {
            alignments,
            roles,
            reserves: std::sync::Arc::new(reserves),
        })
    }

    /// Width-dependent text and owned gaps, before the parent assigns height.
    pub(crate) fn content_reserves(
        theme: &ResolvedTheme,
        sources: [Option<&str>; 3],
        width: f64,
        alignments: RootAlignments,
        scope: &FitScope,
    ) -> Result<RootTextReserves, PlanError> {
        let (_, reserves) = Self::measure_content(theme, sources, width, alignments, scope)?;
        Ok(reserves)
    }

    fn measure_content(
        theme: &ResolvedTheme,
        sources: [Option<&str>; 3],
        width: f64,
        alignments: RootAlignments,
        scope: &FitScope,
    ) -> Result<(Vec<Option<ResolvedTextPlan>>, RootTextReserves), PlanError> {
        let appearances = [
            &theme.typography.root.title,
            &theme.typography.root.subtitle,
            &theme.typography.root.caption,
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
        let resources = theme.prepared_font_environment()?.generation_resources();
        let spacing = theme.root_title_lengths()?;
        let mut top_reserve = crate::theme::spacing::ExactLength::quanta(
            crate::layout::kernel::LayoutScalar::from_quanta(
                plans[0].as_ref().map_or(0, |plan| plan.height_quanta),
            ),
            &resources,
        )?
        .add_quanta(
            crate::layout::kernel::LayoutScalar::from_quanta(
                plans[1].as_ref().map_or(0, |plan| plan.height_quanta),
            ),
            &resources,
        )?;
        if plans[0].is_some() && plans[1].is_some() {
            top_reserve = top_reserve.add(&spacing.title_subtitle_gap, &resources)?;
        }
        if plans[0].is_some() || plans[1].is_some() {
            top_reserve = top_reserve.add(&spacing.title_block_to_content_clearance, &resources)?;
        }
        let bottom_reserve = if let Some(plan) = &plans[2] {
            spacing.caption_to_content_clearance.add_quanta(
                crate::layout::kernel::LayoutScalar::from_quanta(plan.height_quanta),
                &resources,
            )?
        } else {
            crate::theme::spacing::ExactLength::quanta(
                crate::layout::kernel::LayoutScalar::from_quanta(0),
                &resources,
            )?
        };
        let total = top_reserve.add(&bottom_reserve, &resources)?;
        Ok((
            plans,
            RootTextReserves {
                top: top_reserve,
                bottom: bottom_reserve,
                total,
            },
        ))
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
        round.observe_number(self.reserves.top.pixels_rounded(
            crate::layout::kernel::SpatialRounding::NearestEven,
            round.resources(),
        )?)?;
        round.observe_number(self.reserves.bottom.pixels_rounded(
            crate::layout::kernel::SpatialRounding::NearestEven,
            round.resources(),
        )?)?;
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
