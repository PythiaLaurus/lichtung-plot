//! Caller-owned text anchored inside one standard chart panel.

#[cfg(test)]
use crate::Error;

use crate::layout::{Layout, PanelBox, PhysicalTextAlignment, ResolvedTextPlan};
use crate::render::backend::{
    with_clip, with_layer, Backend, PlannedTextPlacement, Point2D, TextAnchor,
};
use crate::theme::ResolvedTheme;
use crate::{PanelCorner, Plot};

const PANEL_TEXT_PADDING: f64 = 8.0;

#[derive(Debug, Clone)]
pub(crate) struct PanelTextPlan {
    pub(crate) text_plan: ResolvedTextPlan,
    position: Point2D,
    anchor: TextAnchor,
    bounds: PanelBox,
    corner: PanelCorner,
}

impl PanelTextPlan {
    pub(crate) fn retained_placement(&self) -> crate::render::backend::PlacedText {
        crate::render::backend::PlacedText {
            placement: PlannedTextPlacement::middle(self.position, self.anchor, 0.0),
            plan: self.text_plan.clone(),
        }
    }
    pub(crate) fn bounds_in(&self, panel: PanelBox) -> PanelBox {
        placement(panel, self.corner, &self.text_plan).2
    }
    pub(crate) fn avoids_legend_in(
        &self,
        panel: PanelBox,
        legend: crate::decorations::legend::LegendBox,
    ) -> bool {
        self.text_plan.width() + 2.0 * PANEL_TEXT_PADDING <= panel.width()
            && self.text_plan.height() + 2.0 * PANEL_TEXT_PADDING <= panel.height()
            && !rectangles_overlap(
                placement(panel, self.corner, &self.text_plan).2,
                PanelBox::new(legend.left, legend.right, legend.bottom, legend.top),
            )
    }

    pub(crate) fn placement_in(&self, panel: PanelBox) -> PlannedTextPlacement {
        let (position, anchor, _) = placement(panel, self.corner, &self.text_plan);
        PlannedTextPlacement::middle(position, anchor, 0.0)
    }
}

pub(crate) fn draw(
    plot: &Plot,
    layout: &Layout,
    plans: &[PanelTextPlan],
    backend: &mut dyn Backend,
) -> Result<(), crate::layout::fit::PlanError> {
    if plot.panel_text_annotations().is_empty() {
        return Ok(());
    }
    with_layer(backend, &plot.layer_name("panel-text"), |backend| {
        with_clip(backend, layout.projection.panel_box(), |backend| {
            for plan in plans {
                backend.draw_planned_text(
                    PlannedTextPlacement::middle(plan.position, plan.anchor, 0.0),
                    &plan.text_plan,
                )?;
            }
            Ok(())
        })
    })
    .map_err(Into::into)
}

pub(crate) fn conflicting_legend_corner(
    plot: &Plot,
    layout: &Layout,
    legend: crate::decorations::legend::LegendBox,
    theme: &ResolvedTheme,
    scope: &crate::layout::fit::FitScope,
) -> Result<Option<PanelCorner>, crate::layout::fit::PlanError> {
    Ok(measure_plans(plot, layout, theme, scope)?
        .into_iter()
        .find_map(|plan| {
            rectangles_overlap(
                plan.bounds,
                PanelBox::new(legend.left, legend.right, legend.bottom, legend.top),
            )
            .then_some(plan.corner)
        }))
}

/// Panel annotations are screen anchored. Domain-only changes cannot resolve
/// their conflict with a legend; a changed assignment must actually separate it.
pub(crate) fn conflict_resolved_by_assignment(
    plot: &Plot,
    layout: &Layout,
    next_panel: PanelBox,
    next_legend: crate::decorations::legend::LegendBox,
    theme: &ResolvedTheme,
    scope: &crate::layout::fit::FitScope,
) -> Result<bool, crate::layout::fit::PlanError> {
    Ok(measure_plans(plot, layout, theme, scope)?
        .iter()
        .all(|plan| plan.avoids_legend_in(next_panel, next_legend)))
}

fn placement(
    panel: PanelBox,
    corner: PanelCorner,
    text: &ResolvedTextPlan,
) -> (Point2D, TextAnchor, PanelBox) {
    let (x, anchor) = match corner {
        PanelCorner::TopLeft | PanelCorner::BottomLeft => {
            (panel.x0 + PANEL_TEXT_PADDING, TextAnchor::Start)
        }
        PanelCorner::TopRight | PanelCorner::BottomRight => {
            (panel.x1 - PANEL_TEXT_PADDING, TextAnchor::End)
        }
    };
    let y = match corner {
        PanelCorner::TopLeft | PanelCorner::TopRight => {
            panel.y1 - PANEL_TEXT_PADDING - text.height() / 2.0
        }
        PanelCorner::BottomLeft | PanelCorner::BottomRight => {
            panel.y0 + PANEL_TEXT_PADDING + text.height() / 2.0
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

#[cfg(test)]
pub(crate) fn plans(
    plot: &Plot,
    layout: &Layout,
    theme: &ResolvedTheme,
) -> Result<Vec<PanelTextPlan>, Error> {
    measure_plans(plot, layout, theme, &crate::layout::fit::FitScope::root())
        .map_err(crate::layout::fit::PlanError::into_error)
}

pub(crate) fn measure_plans(
    plot: &Plot,
    layout: &Layout,
    theme: &ResolvedTheme,
    scope: &crate::layout::fit::FitScope,
) -> Result<Vec<PanelTextPlan>, crate::layout::fit::PlanError> {
    use crate::layout::{kernel::FitKind, AddressRole};
    let panel = layout.projection.panel_box();
    let available_width = panel.width() - PANEL_TEXT_PADDING * 2.0;
    let plans = crate::layout::fit::collect(theme, "panel annotations", plot
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
            let text_plan = theme.measure_legacy_text(
                annotation.text(),
                theme.detail_font_size,
                alignment,
                "panel text annotation",
            )?;
            let width = text_plan.width();
            let text_height = text_plan.height();
            if width > available_width {
                return Err(crate::layout::fit::PlanError::Fit(scope.failure(FitKind::Text, format!(
                    "panel text {:?} requires {width:.1}px but only {available_width:.1}px is available; increase chart width or shorten the annotation",
                    annotation.text()
                ))));
            }
            if text_height + PANEL_TEXT_PADDING * 2.0 > panel.height() {
                return Err(crate::layout::fit::PlanError::Fit(scope.failure(FitKind::Text, "panel text has no usable space inside the resolved chart panel; increase the canvas")));
            }
            let (position, anchor, bounds) = placement(panel, annotation.corner(), &text_plan);
            Ok(PanelTextPlan {
                text_plan,
                position,
                anchor,
                bounds,
                corner: annotation.corner(),
            })
        })
        )?;
    for (index, left) in plans.iter().enumerate() {
        if let Some(right) = plans[index + 1..]
            .iter()
            .find(|right| rectangles_overlap(left.bounds, right.bounds))
        {
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
    Ok(plans)
}

fn rectangles_overlap(left: PanelBox, right: PanelBox) -> bool {
    #[cfg(test)]
    crate::orchestrator::measurements::count(
        crate::orchestrator::measurements::Counter::AnnotationOverlapComparison,
    );
    left.x0 < right.x1 && left.x1 > right.x0 && left.y0 < right.y1 && left.y1 > right.y0
}
