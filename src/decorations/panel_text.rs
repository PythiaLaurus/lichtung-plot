//! Caller-owned text anchored inside one standard chart panel.

use crate::layout::kernel::SpatialRounding;
use crate::layout::{Layout, PanelBox, PhysicalTextAlignment, ResolvedTextPlan};
use crate::render::backend::{PlannedTextPlacement, Point2D, TextAnchor};
use crate::theme::spacing::ExactLength;
use crate::theme::ResolvedTheme;
use crate::{PanelCorner, Plot};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone)]
pub(crate) struct PanelTextPlan {
    pub(crate) text_plan: ResolvedTextPlan,
    position: Point2D,
    anchor: TextAnchor,
    bounds: PanelBox,
    corner: PanelCorner,
    padding: f64,
    minimum_size: (f64, f64),
}

impl PanelTextPlan {
    pub(crate) fn retained_placement(&self) -> crate::render::backend::PlacedText {
        crate::render::backend::PlacedText {
            placement: PlannedTextPlacement::middle(self.position, self.anchor, 0.0),
            plan: self.text_plan.clone(),
        }
    }
    pub(crate) fn bounds_in(&self, panel: PanelBox) -> PanelBox {
        let (position, _, _) = placement(panel, self.corner, &self.text_plan, self.padding);
        let dx = position.x - self.position.x;
        let dy = position.y - self.position.y;
        PanelBox::new(
            self.bounds.x0 + dx,
            self.bounds.x1 + dx,
            self.bounds.y0 + dy,
            self.bounds.y1 + dy,
        )
    }
    pub(crate) fn avoids_legend_in(
        &self,
        panel: PanelBox,
        legend: crate::decorations::legend::LegendBox,
    ) -> bool {
        self.minimum_size.0 <= panel.width()
            && self.minimum_size.1 <= panel.height()
            && !overlap_predicate(
                self.bounds_in(panel),
                PanelBox::new(legend.left, legend.right, legend.bottom, legend.top),
            )
    }

    pub(crate) fn placement_in(&self, panel: PanelBox) -> PlannedTextPlacement {
        let (position, anchor, _) = placement(panel, self.corner, &self.text_plan, self.padding);
        PlannedTextPlacement::middle(position, anchor, 0.0)
    }
}

pub(crate) fn conflicting_legend_corner(
    plans: &[PanelTextPlan],
    legend: crate::decorations::legend::LegendBox,
) -> Option<PanelCorner> {
    for plan in plans {
        if rectangles_overlap(
            plan.bounds,
            PanelBox::new(legend.left, legend.right, legend.bottom, legend.top),
        ) {
            return Some(plan.corner);
        }
    }
    None
}

/// Panel annotations are screen anchored. Domain-only changes cannot resolve
/// their conflict with a legend; a changed assignment must actually separate it.
pub(crate) fn conflict_resolved_by_assignment(
    plans: &[PanelTextPlan],
    next_panel: PanelBox,
    next_legend: crate::decorations::legend::LegendBox,
) -> bool {
    for plan in plans {
        if !plan.avoids_legend_in(next_panel, next_legend) {
            return false;
        }
    }
    true
}

fn placement(
    panel: PanelBox,
    corner: PanelCorner,
    text: &ResolvedTextPlan,
    padding: f64,
) -> (Point2D, TextAnchor, PanelBox) {
    let (x, anchor) = match corner {
        PanelCorner::TopLeft | PanelCorner::BottomLeft => (panel.x0 + padding, TextAnchor::Start),
        PanelCorner::TopRight | PanelCorner::BottomRight => (panel.x1 - padding, TextAnchor::End),
    };
    let y = match corner {
        PanelCorner::TopLeft | PanelCorner::TopRight => panel.y1 - padding - text.height() / 2.0,
        PanelCorner::BottomLeft | PanelCorner::BottomRight => {
            panel.y0 + padding + text.height() / 2.0
        }
    };
    let x = x.round();
    let y = y.round();
    let (left, right) = match anchor {
        TextAnchor::Start => (x, x + text.width()),
        TextAnchor::End => (x - text.width(), x),
        TextAnchor::Middle => (x - text.width() / 2.0, x + text.width() / 2.0),
    };
    (
        Point2D::new(x, y),
        anchor,
        PanelBox::new(
            left,
            right,
            y - text.height() / 2.0,
            y + text.height() / 2.0,
        ),
    )
}

pub(crate) fn measure_plans(
    plot: &Plot,
    layout: &Layout,
    theme: &ResolvedTheme,
    scope: &crate::layout::fit::FitScope,
) -> Result<Vec<PanelTextPlan>, crate::layout::fit::PlanError> {
    use crate::layout::{kernel::FitKind, AddressRole};
    if plot.panel_text_annotations().is_empty() {
        return Ok(Vec::new());
    }
    let panel = layout.projection.panel_box();
    let padding =
        theme
            .annotations
            .panel_text_padding()
            .product(crate::theme::LocalMetric::css_pixels(
                theme.typography.annotations.panel_text.get_font_size(),
            )?);
    let inset = padding.pixels_rounded(SpatialRounding::NearestEven)?;
    let both_edges = padding.times(2);
    let available_width = ExactLength::pixels(panel.width())?
        .subtract(&both_edges)
        .map_or(Ok(0.0), |width| {
            width.pixels_rounded(SpatialRounding::Floor)
        })?;
    let plans = crate::layout::fit::collect( "panel annotations", plot
        .panel_text_annotations()
        .iter()
        .enumerate()
        .map(|(index, annotation)| {
            let scope = scope.child(AddressRole::Annotation, index)?;
            if available_width <= 0.0 {
                return Err(scope.failure(FitKind::Constraint, "panel text has no usable space inside the resolved chart panel; increase the canvas").into());
            }
            let alignment = match annotation.corner() {
                PanelCorner::TopLeft | PanelCorner::BottomLeft => PhysicalTextAlignment::Start,
                PanelCorner::TopRight | PanelCorner::BottomRight => PhysicalTextAlignment::End,
            };
            let text_plan = theme.measure_legacy_appearance_text(
                annotation.text(),
                &theme.typography.annotations.panel_text,
                alignment,
                "panel text annotation",
                false,
            )?;
            let width = text_plan.width();
            let text_height = text_plan.height();
            let minimum_size = (
                both_edges.add_pixels(width)?.pixels_rounded(SpatialRounding::Ceil)?,
                both_edges.add_pixels(text_height)?.pixels_rounded(SpatialRounding::Ceil)?,
            );
            if minimum_size.0 > panel.width() {
                return Err(crate::layout::fit::PlanError::Fit(scope.failure(FitKind::Text, format!(
                    "panel text {:?} requires {width:.1}px but only {available_width:.1}px is available; increase chart width or shorten the annotation",
                    annotation.text()
                ))));
            }
            if minimum_size.1 > panel.height() {
                return Err(crate::layout::fit::PlanError::Fit(scope.failure(FitKind::Text, "panel text has no usable space inside the resolved chart panel; increase the canvas")));
            }
            let (position, anchor, nominal) = placement(panel, annotation.corner(), &text_plan, inset);
            let mut ink = crate::render::geometry::GeometrySnapshot::new();
            ink.text(
                PlannedTextPlacement::middle(position, anchor, 0.0),
                &text_plan,
                None,
            )?;
            let bounds = ink.bounds().iter().copied()
                .reduce(crate::render::geometry::GeometryBox::union)
                .map_or(nominal, |box_| PanelBox::new(box_.left, box_.right, box_.bottom, box_.top));
            if bounds.x0 < panel.x0 || bounds.x1 > panel.x1
                || bounds.y0 < panel.y0 || bounds.y1 > panel.y1 {
                return Err(scope.failure(FitKind::Text,
                    "required panel text ink exceeds its assigned chart panel").into());
            }
            Ok(PanelTextPlan {
                text_plan,
                position,
                anchor,
                bounds,
                corner: annotation.corner(),
                padding: inset,
                minimum_size,
            })
        })
        )?;
    check_collisions(&plans, scope)?;
    Ok(plans)
}

fn check_collisions(
    plans: &[PanelTextPlan],

    scope: &crate::layout::fit::FitScope,
) -> Result<(), crate::layout::fit::PlanError> {
    use crate::layout::{kernel::FitKind, AddressRole};
    for (index, left) in plans.iter().enumerate() {
        for right in &plans[index + 1..] {
            if !rectangles_overlap(left.bounds, right.bounds) {
                continue;
            }
            return Err(scope
                .child(AddressRole::Annotation, index)?
                .failure(
                    FitKind::OverlayCollision,
                    format!(
                "panel text annotations at {:?} and {:?} overlap; shorten or remove one annotation",
                left.corner, right.corner
            ),
                )
                .into());
        }
    }
    Ok(())
}

fn rectangles_overlap(left: PanelBox, right: PanelBox) -> bool {
    overlap_predicate(left, right)
}

fn overlap_predicate(left: PanelBox, right: PanelBox) -> bool {
    #[cfg(test)]
    crate::orchestrator::measurements::count(
        crate::orchestrator::measurements::Counter::AnnotationOverlapComparison,
    );
    left.x0 < right.x1 && left.x1 > right.x0 && left.y0 < right.y1 && left.y1 > right.y0
}
