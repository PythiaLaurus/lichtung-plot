//! Assigned-width root text, frozen before scene capture and Null validation.

use crate::layout::fit::{self, FitScope, PlanError};
use crate::layout::{AddressRole, PanelBox, ResolvedTextPlan};
use crate::render::backend::{Backend, PlannedTextPlacement, Point2D};
use crate::theme::composition::RootPaddingLengths;
use crate::theme::spacing::ExactLength;
use crate::theme::ResolvedTheme;
use crate::{Error, TextAlignment};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RootAlignments {
    pub(crate) title: TextAlignment,
    pub(crate) subtitle: TextAlignment,
    pub(crate) caption: TextAlignment,
}

impl RootAlignments {
    pub(crate) fn resolve(
        theme: &ResolvedTheme,
        plot: [Option<TextAlignment>; 3],
        panel: [Option<TextAlignment>; 2],
    ) -> Self {
        let title = plot[0]
            .or(panel[0])
            .unwrap_or(theme.composition.get_root_title_alignment());
        Self {
            title,
            subtitle: plot[1]
                .or(panel[1])
                .or(theme.composition.get_root_subtitle_alignment())
                .unwrap_or(title),
            caption: plot[2].unwrap_or(theme.composition.get_root_caption_alignment()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct RootTextRole {
    request: crate::Text,
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
    pub(crate) fn height_with_body(&self, body: f64) -> Result<ExactLength, Error> {
        self.total.add_pixels(body)
    }
    pub(crate) fn body(
        &self,
        outer: PanelBox,
        padding: &RootPaddingLengths,
    ) -> Result<PanelBox, Error> {
        Ok(PanelBox::new(
            padding.left.after_pixels(outer.x0)?,
            padding.right.before_pixels(outer.x1)?,
            self.bottom
                .add_length(&padding.bottom)
                .after_pixels(outer.y0)?,
            self.top.add_length(&padding.top).before_pixels(outer.y1)?,
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

    pub(crate) fn check_width(self, width: f64, scope: &FitScope) -> Result<(), PlanError> {
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
                    .child(role, 0)?
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
    pub(crate) fn appearances<'a>(
        theme: &'a ResolvedTheme,
        sources: [Option<&'a crate::Text>; 3],
    ) -> [crate::text::TextAppearance<'a>; 3] {
        let roles = [
            &theme.typography.root.title,
            &theme.typography.root.subtitle,
            &theme.typography.root.caption,
        ];
        std::array::from_fn(|index| {
            sources[index].map_or_else(|| roles[index].into(), |text| text.appearance(roles[index]))
        })
    }

    pub(crate) fn minimum_height(
        theme: &ResolvedTheme,
        sources: [Option<&crate::Text>; 3],
        body: f64,
    ) -> Result<f64, Error> {
        let appearances = Self::appearances(theme, sources);
        let spacing =
            theme.root_title_lengths(appearances.map(|style| style.theme.get_font_size()))?;
        let present = sources.map(|source| source.is_some());
        let mut required = ExactLength::pixels(body)?;
        for (active, appearance) in present.into_iter().zip(appearances) {
            if active {
                required = required.add_length(&appearance.theme.line_pitch_exact()?);
            }
        }
        if present[0] && present[1] {
            required = required.add_length(&spacing.title_subtitle_gap);
        }
        if present[0] || present[1] {
            required = required.add_length(&spacing.title_block_to_content_clearance);
        }
        if present[2] {
            required = required.add_length(&spacing.caption_to_content_clearance);
        }
        if present.into_iter().any(|active| active) {
            required = required.add_length(&theme.root_padding_lengths()?.vertical());
        }
        crate::theme::composition::hard(&required)
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
        sources: [Option<&crate::Text>; 3],
        scope: Option<&str>,
    ) -> Result<RootTitleBlockIntrinsics, Error> {
        #[cfg(not(test))]
        let _ = scope;

        #[cfg(test)]
        crate::orchestrator::measurements::intrinsic(
            crate::orchestrator::measurements::Counter::RootTitleIntrinsic,
            scope,
        );
        let mut roles = [(0.0_f64, 0.0_f64); 3];
        for ((source, appearance), widths) in sources
            .into_iter()
            .zip(Self::appearances(theme, sources))
            .zip(&mut roles)
        {
            if let Some(source) = source {
                *widths =
                    fit::intrinsic_widths(theme, source.as_str(), appearance, "root title block")?;
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
        sources: [Option<&crate::Text>; 3],
        span: PanelBox,
        padding: &RootPaddingLengths,
        maximum_width: f64,
        alignments: RootAlignments,
        minimum_body_height: f64,
        scope: &FitScope,
    ) -> Result<Self, PlanError> {
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

        let text_span = padding.inset(span)?;
        let (plans, reserves) = Self::measure_content(
            theme,
            sources,
            maximum_width.min(text_span.width()),
            alignments,
            scope,
        )?;
        let spacing = theme.root_title_lengths(
            Self::appearances(theme, sources).map(|style| style.theme.get_font_size()),
        )?;
        let adjacent = plans[0].is_some() && plans[1].is_some();
        if crate::theme::composition::hard(
            &reserves
                .height_with_body(minimum_body_height)?
                .add_length(&padding.vertical()),
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
        )?;
        let mut roles = [None, None, None];
        for (index, plan) in plans.into_iter().enumerate() {
            if let Some(text) = plan {
                let top = if index == 2 {
                    crate::theme::spacing::ExactLength::quanta(
                        crate::layout::kernel::LayoutScalar::from_quanta(text.height_quanta),
                    )?
                    .add_length(&padding.bottom)
                    .after_pixels(span.y0)?
                } else {
                    consumed.add_length(&padding.top).before_pixels(span.y1)?
                };
                let alignment = [alignments.title, alignments.subtitle, alignments.caption][index];
                let (mut x, anchor) = alignment.resolve_physical(text_span.x0, text_span.x1);
                let left = match alignment {
                    TextAlignment::Start => text_span.x0,
                    TextAlignment::Center => {
                        (text_span.x0 * 1024.0 + (text_span.width() - text.width()) * 512.0)
                            .round_ties_even()
                            / 1024.0
                    }
                    TextAlignment::End => text_span.x1 - text.width(),
                };
                if alignment == TextAlignment::Center {
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
                            )?
                            .subtract(&crate::theme::spacing::ExactLength::quanta(
                                crate::layout::kernel::LayoutScalar::from_quanta(first),
                            )?)
                            .ok_or_else(|| {
                                Error::Render("root caption baseline exceeds its extent".into())
                            })?
                            .add_length(&padding.bottom)
                            .after_pixels(span.y0)?
                        } else {
                            consumed
                                .add_quanta(crate::layout::kernel::LayoutScalar::from_quanta(
                                    first,
                                ))?
                                .add_length(&padding.top)
                                .before_pixels(span.y1)?
                        },
                    ),
                    anchor,
                    0.0,
                );
                if index < 2 {
                    consumed = consumed.add_quanta(
                        crate::layout::kernel::LayoutScalar::from_quanta(text.height_quanta),
                    )?;
                    if index == 0 && adjacent {
                        consumed = consumed.add_length(&spacing.title_subtitle_gap);
                    }
                }
                roles[index] = Some(RootTextRole {
                    request: sources[index]
                        .expect("measured present root text")
                        .try_copy()?,
                    text,
                    bounds,
                    placement,
                });
            }
        }

        Ok(Self {
            alignments,
            roles,
            reserves: std::sync::Arc::new(reserves),
        })
    }

    /// Width-dependent text and owned gaps, before the parent assigns height.
    pub(crate) fn content_reserves(
        theme: &ResolvedTheme,
        sources: [Option<&crate::Text>; 3],
        width: f64,
        alignments: RootAlignments,
        scope: &FitScope,
    ) -> Result<RootTextReserves, PlanError> {
        let (_, reserves) = Self::measure_content(theme, sources, width, alignments, scope)?;
        Ok(reserves)
    }

    fn measure_content(
        theme: &ResolvedTheme,
        sources: [Option<&crate::Text>; 3],
        width: f64,
        alignments: RootAlignments,
        scope: &FitScope,
    ) -> Result<(Vec<Option<ResolvedTextPlan>>, RootTextReserves), PlanError> {
        let appearances = Self::appearances(theme, sources);
        let align = [alignments.title, alignments.subtitle, alignments.caption];
        let addresses = [
            AddressRole::RootTitle,
            AddressRole::RootSubtitle,
            AddressRole::RootCaption,
        ];
        let plans = fit::collect(
            "root title block",
            (0..3).map(|index| {
                sources[index]
                    .map(|source| {
                        fit::measured(fit::text(
                            theme,
                            source.as_str(),
                            appearances[index],
                            width,
                            align[index].into(),
                            "root title block",
                            &scope.child(addresses[index], 0)?,
                        ))
                    })
                    .transpose()
            }),
        )?;

        let spacing =
            theme.root_title_lengths(appearances.map(|style| style.theme.get_font_size()))?;
        let mut top_reserve = crate::theme::spacing::ExactLength::quanta(
            crate::layout::kernel::LayoutScalar::from_quanta(
                plans[0].as_ref().map_or(0, |plan| plan.height_quanta),
            ),
        )?
        .add_quanta(crate::layout::kernel::LayoutScalar::from_quanta(
            plans[1].as_ref().map_or(0, |plan| plan.height_quanta),
        ))?;
        if plans[0].is_some() && plans[1].is_some() {
            top_reserve = top_reserve.add_length(&spacing.title_subtitle_gap);
        }
        if plans[0].is_some() || plans[1].is_some() {
            top_reserve = top_reserve.add_length(&spacing.title_block_to_content_clearance);
        }
        let bottom_reserve = if let Some(plan) = &plans[2] {
            spacing.caption_to_content_clearance.add_quanta(
                crate::layout::kernel::LayoutScalar::from_quanta(plan.height_quanta),
            )?
        } else {
            crate::theme::spacing::ExactLength::quanta(
                crate::layout::kernel::LayoutScalar::from_quanta(0),
            )?
        };
        let total = top_reserve.add_length(&bottom_reserve);
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
        sources: [Option<&crate::Text>; 3],
        backend: &mut dyn Backend,
    ) -> Result<(), Error> {
        for (role, source) in self.roles.iter().zip(sources) {
            match (role, source) {
                (None, None) => {}
                (Some(role), Some(source)) if &role.request == source => {
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
        round.observe_number(
            self.reserves
                .top
                .pixels_rounded(crate::layout::kernel::SpatialRounding::NearestEven)?,
        )?;
        round.observe_number(
            self.reserves
                .bottom
                .pixels_rounded(crate::layout::kernel::SpatialRounding::NearestEven)?,
        )?;
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
