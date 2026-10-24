//! Measured common text, in ordinary role typography and containing geometry.

use crate::layout::fit::{self, FitKind, FitScope, PlanError};
use crate::render::backend::{PlacedText, PlannedTextPlacement, Point2D, TextAnchor};
use crate::{Error, Text, TextAlignment};

#[derive(Debug)]
pub(crate) struct ExplanationPlan {
    pub(crate) text: PlacedText,
    pub(crate) height: f64,
}

impl ExplanationPlan {
    /// Relative to the top-left of the allocated slot, with downward-negative y.
    pub(crate) fn measure<'a>(
        text: &'a Text,
        role: impl Into<crate::text::TextAppearance<'a>>,
        width: f64,
        default: TextAlignment,
        theme: &crate::theme::ResolvedTheme,
        scope: &FitScope,
    ) -> Result<Self, PlanError> {
        let alignment = text.resolved_alignment(default);
        let plan = fit::measured(fit::single_text(
            theme,
            text.as_str(),
            text.appearance(role),
            width,
            alignment.into(),
            "numeric explanation",
            scope,
        ))?;
        let placement =
            PlannedTextPlacement::middle(Point2D::new(0.0, 0.0), TextAnchor::Start, 0.0);
        let ink = crate::render::geometry::planned_text_ink_bounds(
            &plan,
            placement.position,
            placement.anchor,
            0.0,
            placement.origin,
            placement.reverse_lines,
        );
        let left = ink.map_or(0.0, |ink| ink.left.min(0.0));
        let right = ink.map_or(plan.width(), |ink| ink.right.max(plan.width()));
        let top = ink.map_or(plan.height() / 2.0, |ink| ink.top.max(plan.height() / 2.0));
        let bottom = ink.map_or(-plan.height() / 2.0, |ink| {
            ink.bottom.min(-plan.height() / 2.0)
        });
        let occupied = right - left;
        if occupied > width {
            return Err(scope
                .failure(
                    FitKind::Text,
                    "required numeric explanation exceeds its assigned width",
                )
                .into());
        }
        let offset = match alignment {
            TextAlignment::Start => 0.0,
            TextAlignment::Center => (width - occupied) / 2.0,
            TextAlignment::End => width - occupied,
        };
        let height = crate::layout::kernel::required_length(top - bottom)?;
        let placement = crate::layout::text::placement::resolve(
            PlannedTextPlacement {
                position: Point2D::new(offset - left, -top),
                ..placement
            },
            &plan,
            Some(crate::layout::PanelBox::new(0.0, width, -height, 0.0)),
        )?;
        let plan = crate::layout::text::placement::contain(
            plan,
            placement,
            crate::layout::PanelBox::new(0.0, width, -height, 0.0),
            scope,
        )?;
        Ok(Self {
            text: PlacedText { plan, placement },
            height,
        })
    }

    pub(crate) fn at(mut self, x: f64, top: f64) -> PlacedText {
        self.text.placement.position.x += x;
        self.text.placement.position.y += top;
        self.text.plan = self.text.plan.translate_paint_region(x, top);
        self.text
    }
}

pub(crate) fn preferred_width<'a>(
    text: &'a Text,
    role: impl Into<crate::text::TextAppearance<'a>>,
    theme: &crate::theme::ResolvedTheme,
) -> Result<f64, Error> {
    crate::layout::interior::sizing::text_width(theme, text.as_str(), text.appearance(role))
}
