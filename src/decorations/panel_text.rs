//! Typed annotation references and measured block anchors inside one actual view.

use crate::layout::{
    fit::{self, FitKind, FitScope, PlanError},
    Layout, PanelBox, ResolvedTextPlan,
};
use crate::render::backend::{PlannedTextPlacement, Point2D};
use crate::text::position::resolve::{self, ReferenceAxis};
use crate::theme::{spacing::ExactLength, ResolvedTheme};
use crate::{
    PanelCorner, Plot, TextAlignment, TextAnchor, TextFailure, TextOverlap, TextPosition,
    TextWrapping,
};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone)]
pub(crate) struct PanelTextPlan {
    pub(crate) text_plan: ResolvedTextPlan,
    placement: PlannedTextPlacement,
    bounds: PanelBox,
    source: TextPosition,
    reference: Reference,
    panel: PanelBox,
    minimum_size: (f64, f64),
    overlap: bool,
}
#[derive(Debug, Clone, Copy)]
enum Reference {
    Corner(PanelCorner, f64),
    Axes([ReferenceAxis; 2]),
}
impl Reference {
    fn point(self, panel: PanelBox) -> Point2D {
        match self {
            Self::Axes(axes) => resolve::point(axes, panel),
            Self::Corner(corner, padding) => Point2D::new(
                match corner {
                    PanelCorner::TopLeft | PanelCorner::BottomLeft => panel.x0 + padding,
                    _ => panel.x1 - padding,
                },
                match corner {
                    PanelCorner::TopLeft | PanelCorner::TopRight => panel.y1 - padding,
                    _ => panel.y0 + padding,
                },
            ),
        }
    }
}
impl PanelTextPlan {
    pub(crate) fn retained_placement(&self) -> crate::render::backend::PlacedText {
        crate::render::backend::PlacedText {
            placement: self.placement,
            plan: self.text_plan.clone(),
        }
    }
    pub(crate) fn placement_in(&self, panel: PanelBox) -> PlannedTextPlacement {
        let old = self.reference.point(self.panel);
        let new = self.reference.point(panel);
        PlannedTextPlacement {
            position: Point2D::new(
                self.placement.position.x + new.x - old.x,
                self.placement.position.y + new.y - old.y,
            ),
            ..self.placement
        }
    }
    pub(crate) fn bounds_in(&self, panel: PanelBox) -> PanelBox {
        let next = self.placement_in(panel).position;
        let old = self.placement.position;
        PanelBox::new(
            self.bounds.x0 + next.x - old.x,
            self.bounds.x1 + next.x - old.x,
            self.bounds.y0 + next.y - old.y,
            self.bounds.y1 + next.y - old.y,
        )
    }
    pub(crate) fn avoids_legend_in(
        &self,
        panel: PanelBox,
        legend: crate::decorations::legend::LegendBox,
    ) -> bool {
        self.text_plan.lines.is_empty()
            || self.overlap
            || (self.minimum_size.0 <= panel.width()
                && self.minimum_size.1 <= panel.height()
                && !overlap_predicate(
                    self.bounds_in(panel),
                    PanelBox::new(legend.left, legend.right, legend.bottom, legend.top),
                ))
    }
}

pub(crate) fn conflicting_legend_annotation(
    plans: &[PanelTextPlan],
    legend: crate::decorations::legend::LegendBox,
) -> Option<&TextPosition> {
    plans
        .iter()
        .find(|plan| {
            !plan.overlap
                && overlap_predicate(
                    plan.bounds,
                    PanelBox::new(legend.left, legend.right, legend.bottom, legend.top),
                )
        })
        .map(|plan| &plan.source)
}
pub(crate) fn conflict_resolved_by_assignment(
    plans: &[PanelTextPlan],
    next_panel: PanelBox,
    next_legend: crate::decorations::legend::LegendBox,
) -> bool {
    plans
        .iter()
        .all(|plan| plan.avoids_legend_in(next_panel, next_legend))
}

fn corner_anchor(corner: PanelCorner) -> TextAnchor {
    match corner {
        PanelCorner::TopLeft => TextAnchor::top_left(),
        PanelCorner::TopRight => TextAnchor::top_right(),
        PanelCorner::BottomLeft => TextAnchor::bottom_left(),
        PanelCorner::BottomRight => TextAnchor::bottom_right(),
    }
}

pub(crate) fn measure_plans(
    plot: &Plot,
    layout: &Layout,
    theme: &ResolvedTheme,
    scope: &FitScope,
) -> Result<Vec<PanelTextPlan>, PlanError> {
    let plans = fit::collect(
        "panel annotations",
        plot.panel_text_annotations()
            .iter()
            .enumerate()
            .map(|(index, annotation)| {
                let scope = scope.child(crate::layout::AddressRole::Annotation, index)?;
                measure(annotation, plot, layout, theme, &scope)
            }),
    )?;
    check_collisions(&plans, scope)?;
    Ok(plans)
}

#[expect(
    clippy::too_many_lines,
    reason = "one annotation transaction resolves its actual reference, complete policy, measured transform and collision footprint"
)]
fn measure(
    annotation: &crate::PanelTextAnnotation,
    plot: &Plot,
    layout: &Layout,
    theme: &ResolvedTheme,
    scope: &FitScope,
) -> Result<PanelTextPlan, PlanError> {
    let panel = layout.projection.panel_box();
    let appearance = annotation
        .content()
        .appearance(&theme.typography.annotations.panel_text);
    let padding =
        theme
            .annotations
            .get_panel_text_padding()
            .product(crate::theme::LocalMetric::css_pixels(
                appearance.theme.get_font_size(),
            )?);
    let inset = padding.pixels_rounded(crate::layout::kernel::SpatialRounding::NearestEven)?;
    let corner = annotation.position().as_corner();
    let reference = if let Some(corner) = corner {
        Reference::Corner(corner, inset)
    } else {
        Reference::Axes(resolve::resolve(
            annotation.position(),
            plot,
            layout,
            theme,
        )?)
    };
    let mut request = appearance.layout.cloned().unwrap_or_default();
    let failure = request
        .instructions()
        .overflow
        .as_ref()
        .map_or(TextFailure::Error, |value| value.failure);
    let overlap = request.instructions().overlap == Some(TextOverlap::Allow);
    request
        .edit_instructions()
        .wrapping
        .get_or_insert(TextWrapping::NoSoftWrap);
    let both = if corner.is_some() {
        padding.times(2)
    } else {
        ExactLength::pixels(0.0)?
    };
    let available = ExactLength::pixels(panel.width())?
        .subtract(&both)
        .map_or(Ok(0.0), |width| {
            width.pixels_rounded(crate::layout::kernel::SpatialRounding::Floor)
        })?;
    let default_alignment = if matches!(
        corner,
        Some(PanelCorner::TopRight | PanelCorner::BottomRight)
    ) {
        TextAlignment::End
    } else {
        TextAlignment::Start
    };
    let mut text_plan = fit::measured(fit::single_text(
        theme,
        annotation.text(),
        crate::text::TextAppearance {
            layout: Some(&request),
            ..appearance
        },
        available,
        annotation
            .content()
            .resolved_alignment(default_alignment)
            .into(),
        "panel text annotation",
        scope,
    ))?;
    let anchor = request
        .instructions()
        .anchor
        .unwrap_or_else(|| corner.map_or_else(TextAnchor::center, corner_anchor));
    let placement = crate::layout::text::placement::at(
        &text_plan,
        reference.point(panel),
        anchor,
        0.0,
        false,
        request.instructions().offset,
    )?;
    let (width, height) = if matches!(reference,Reference::Axes(axes) if axes.iter().any(|axis|matches!(axis,ReferenceAxis::Figure(_))))
    {
        plot.figure_dimensions()?
    } else {
        (0, 0)
    };
    let allowed = match reference {
        Reference::Corner(..) => panel,
        Reference::Axes(axes) => PanelBox::new(
            if matches!(axes[0], ReferenceAxis::Figure(_)) {
                0.0
            } else {
                panel.x0
            },
            if matches!(axes[0], ReferenceAxis::Figure(_)) {
                f64::from(width)
            } else {
                panel.x1
            },
            if matches!(axes[1], ReferenceAxis::Figure(_)) {
                0.0
            } else {
                panel.y0
            },
            if matches!(axes[1], ReferenceAxis::Figure(_)) {
                f64::from(height)
            } else {
                panel.y1
            },
        ),
    };
    let mut bounds = crate::layout::text::placement::logical_bounds(placement, &text_plan)?;
    if let Some(ink) = crate::render::geometry::planned_text_ink_bounds(
        &text_plan,
        placement.position,
        placement.anchor,
        placement.rotation,
        placement.origin,
        placement.reverse_lines,
    ) {
        bounds = PanelBox::new(
            bounds.x0.min(ink.left),
            bounds.x1.max(ink.right),
            bounds.y0.min(ink.bottom),
            bounds.y1.max(ink.top),
        );
    }
    if !text_plan.lines.is_empty()
        && (bounds.x0 < allowed.x0
            || bounds.x1 > allowed.x1
            || bounds.y0 < allowed.y0
            || bounds.y1 > allowed.y1)
    {
        match failure {
            TextFailure::Error => {
                return Err(scope
                    .failure(
                        FitKind::Text,
                        "annotation text exceeds its actual referenced region",
                    )
                    .into())
            }
            TextFailure::Hide => {
                text_plan = text_plan.hide()?;
                bounds = PanelBox::new(
                    placement.position.x,
                    placement.position.x,
                    placement.position.y,
                    placement.position.y,
                );
            }
            TextFailure::Clip => {
                text_plan = text_plan.clip_region(allowed);
                bounds = PanelBox::new(
                    bounds.x0.max(allowed.x0),
                    bounds.x1.min(allowed.x1),
                    bounds.y0.max(allowed.y0),
                    bounds.y1.min(allowed.y1),
                );
            }
            TextFailure::Visible => {}
        }
    }
    let minimum_size = if text_plan.lines.is_empty() {
        (0.0, 0.0)
    } else {
        (
            both.add_pixels(bounds.width().max(0.0))?
                .pixels_rounded(crate::layout::kernel::SpatialRounding::Ceil)?,
            both.add_pixels(bounds.height().max(0.0))?
                .pixels_rounded(crate::layout::kernel::SpatialRounding::Ceil)?,
        )
    };
    Ok(PanelTextPlan {
        text_plan,
        placement,
        bounds,
        source: annotation.position().clone(),
        reference,
        panel,
        minimum_size,
        overlap,
    })
}

fn check_collisions(plans: &[PanelTextPlan], scope: &FitScope) -> Result<(), PlanError> {
    for (index, left) in plans.iter().enumerate() {
        for right in &plans[index + 1..] {
            if !left.overlap && !right.overlap && overlap_predicate(left.bounds, right.bounds) {
                return Err(scope
                    .child(crate::layout::AddressRole::Annotation, index)?
                    .failure(
                        FitKind::OverlayCollision,
                        "required panel annotations overlap",
                    )
                    .into());
            }
        }
    }
    Ok(())
}
fn overlap_predicate(left: PanelBox, right: PanelBox) -> bool {
    #[cfg(test)]
    crate::orchestrator::measurements::count(
        crate::orchestrator::measurements::Counter::AnnotationOverlapComparison,
    );
    left.width() > 0.0
        && left.height() > 0.0
        && right.width() > 0.0
        && right.height() > 0.0
        && left.x0 < right.x1
        && left.x1 > right.x0
        && left.y0 < right.y1
        && left.y1 > right.y0
}
