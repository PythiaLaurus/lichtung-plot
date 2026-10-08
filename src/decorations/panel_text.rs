//! Caller-owned text anchored inside one standard chart panel.

use crate::layout::{Layout, PanelBox, PhysicalTextAlignment, ResolvedTextPlan};
use crate::render::backend::{PlannedTextPlacement, Point2D, TextAnchor};
use crate::theme::ResolvedTheme;
#[cfg(test)]
use crate::Error;
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
        let (position, _, _) = placement(panel, self.corner, &self.text_plan);
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
        self.text_plan.width() + 2.0 * PANEL_TEXT_PADDING <= panel.width()
            && self.text_plan.height() + 2.0 * PANEL_TEXT_PADDING <= panel.height()
            && !overlap_predicate(
                self.bounds_in(panel),
                PanelBox::new(legend.left, legend.right, legend.bottom, legend.top),
            )
    }

    pub(crate) fn placement_in(&self, panel: PanelBox) -> PlannedTextPlacement {
        let (position, anchor, _) = placement(panel, self.corner, &self.text_plan);
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
            if width > available_width {
                return Err(crate::layout::fit::PlanError::Fit(scope.failure(FitKind::Text, format!(
                    "panel text {:?} requires {width:.1}px but only {available_width:.1}px is available; increase chart width or shorten the annotation",
                    annotation.text()
                ))));
            }
            if text_height + PANEL_TEXT_PADDING * 2.0 > panel.height() {
                return Err(crate::layout::fit::PlanError::Fit(scope.failure(FitKind::Text, "panel text has no usable space inside the resolved chart panel; increase the canvas")));
            }
            let (position, anchor, nominal) = placement(panel, annotation.corner(), &text_plan);
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

#[test]
fn collision_scans_stop_at_the_first_matching_pair() {
    use crate::decorations::legend::{test_assigned_legend, LegendEntry, LegendIdentity};
    let plot = Plot::scatter(
        crate::NumericValues::new("x", [0.0, 1.0]),
        crate::NumericValues::new("y", [0.0, 1.0]),
    )
    .panel_text_annotation(crate::PanelTextAnnotation::new(PanelCorner::TopLeft, "note").unwrap())
    .erase();
    let layout = Layout::envelope(PanelBox::new(0.0, 400.0, 0.0, 400.0));
    let theme = crate::Theme::Publication.resolve().unwrap();
    let original = plans(&plot, &layout, &theme).unwrap().remove(0);
    let mut entries = vec![LegendEntry::point(
        LegendIdentity::literal("group", "entry"),
        crate::render::backend::PointStyle::new((0, 0, 0), 3),
    )];
    crate::decorations::legend::resolve_entries(&plot, &mut entries).unwrap();
    let mut legend = test_assigned_legend("", entries, (400.0, 400.0), &theme)
        .unwrap()
        .box_at(40.0, 0.0);
    legend.right = 50.0;
    legend.top = 10.0;

    let mut separated = vec![original; 4];
    for (index, plan) in separated.iter_mut().enumerate() {
        let x = u32::try_from(index).unwrap() * 20;
        plan.bounds = PanelBox::new(f64::from(x), f64::from(x + 10), 0.0, 10.0);
    }
    for (collision, expected_pairs) in [(None, 6), (Some((0, 1)), 1), (Some((1, 3)), 5)] {
        let mut candidates = separated.clone();
        if let Some((left, right)) = collision {
            candidates[right].bounds = candidates[left].bounds;
        }
        {
            crate::orchestrator::measurements::start();
            let result = check_collisions(&candidates, &crate::layout::fit::FitScope::root());
            let observed = crate::orchestrator::measurements::finish();
            assert_eq!(
                observed.counters
                    [crate::orchestrator::measurements::Counter::AnnotationOverlapComparison
                        as usize],
                expected_pairs
            );
            if collision.is_some() {
                assert!(matches!(result, Err(crate::layout::fit::PlanError::Fit(_))));
            } else {
                result.unwrap();
            }
        }
    }

    assert_eq!(
        conflicting_legend_corner(&separated, legend),
        Some(PanelCorner::TopLeft)
    );

    // The assignment's first width check already fails.

    assert!(!conflict_resolved_by_assignment(
        &separated,
        PanelBox::new(0.0, 1.0, 0.0, 1.0),
        legend
    ));
}

#[test]
fn panel_text_uses_complete_private_role_and_unclipped_canonical_bounds() {
    let plot = Plot::scatter(
        crate::NumericValues::new("x", [0.0, 1.0]),
        crate::NumericValues::new("y", [0.0, 1.0]),
    )
    .panel_text_annotation(
        crate::PanelTextAnnotation::new(PanelCorner::TopRight, "Required note").unwrap(),
    )
    .erase();
    let layout = Layout::envelope(PanelBox::new(80.0, 770.0, 60.0, 570.0));
    let mut theme = crate::Theme::Publication.resolve().unwrap();
    let original = plans(&plot, &layout, &theme).unwrap();
    theme.detail_font_size = 0;
    theme.typography.annotations.panel_text.font_size = 32;
    theme.typography.annotations.panel_text.paint = crate::theme::Color::rgb(18, 34, 56);
    let selected = plans(&plot, &layout, &theme).unwrap();
    assert!(selected[0].bounds.width() > original[0].bounds.width());
    assert_eq!(selected[0].text_plan.style.font_size, 32);
    assert_eq!(
        selected[0].text_plan.style.paint,
        theme.typography.annotations.panel_text.paint
    );
    let mut ink = crate::render::geometry::GeometrySnapshot::new();
    let text = selected[0].retained_placement();
    ink.text(text.placement, &text.plan, None).unwrap();
    let box_ = ink
        .bounds()
        .iter()
        .copied()
        .reduce(crate::render::geometry::GeometryBox::union)
        .unwrap();
    assert_eq!(
        selected[0].bounds.as_tuple(),
        (box_.left, box_.right, box_.bottom, box_.top)
    );
    let panel = layout.projection.panel_box();
    assert!(box_.left >= panel.x0 && box_.right <= panel.x1);
    assert!(box_.bottom >= panel.y0 && box_.top <= panel.y1);
    let shifted = PanelBox::new(
        panel.x0 + 2.25,
        panel.x1 + 2.25,
        panel.y0 + 5.5,
        panel.y1 + 5.5,
    );
    let mut shifted_ink = crate::render::geometry::GeometrySnapshot::new();
    shifted_ink
        .text(
            selected[0].placement_in(shifted),
            &selected[0].text_plan,
            None,
        )
        .unwrap();
    let shifted_box = shifted_ink
        .bounds()
        .iter()
        .copied()
        .reduce(crate::render::geometry::GeometryBox::union)
        .unwrap();
    assert_eq!(
        selected[0].bounds_in(shifted).as_tuple(),
        (
            shifted_box.left,
            shifted_box.right,
            shifted_box.bottom,
            shifted_box.top
        )
    );
}
